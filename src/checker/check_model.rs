use super::*;
use crate::checker::type_utils;
use crate::parser::AstNode;

impl Checker {
    pub(crate) fn check_model(&mut self, ctx: &CheckContext, node_id: NodeId) -> TypeId {
        let ast = require_ast_or!(self, self.error_type);

        // Circular reference detection
        if let Some(type_id) = self.check_circular_ref(node_id) {
            return type_id;
        }

        let node = match ast.id_to_node(node_id) {
            Some(n) => n.clone(),
            None => return self.error_type,
        };

        let (
            name,
            properties,
            extends,
            is_node,
            decorators,
            has_template_params,
            template_param_ids,
        ) = match &node {
            AstNode::ModelDeclaration(decl) => {
                let name = Self::get_identifier_name(&ast, decl.name);
                let has_tmpl = !decl.template_parameters.is_empty();
                (
                    name,
                    decl.properties.clone(),
                    decl.extends,
                    decl.is,
                    decl.decorators.clone(),
                    has_tmpl,
                    decl.template_parameters.clone(),
                )
            }
            AstNode::ModelExpression(expr) => (
                String::new(),
                expr.properties.clone(),
                None,
                None,
                vec![],
                false,
                vec![],
            ),
            _ => return self.error_type,
        };

        // Mark this node as currently being type-checked (for circular detection)
        // Insert AFTER all early returns to avoid leaking entries in pending_type_checks.
        self.pending_type_checks.insert(node_id);

        // Check template parameter declarations (two-pass for circular constraint detection)
        // TS: checkTemplateDeclaration(ctx, node)
        self.check_template_declaration(ctx, &ast, &template_param_ids);

        // Check if already fully type-checked
        // TS: if (links.declaredType && ctx.mapper === undefined) return declaredType
        if ctx.mapper.is_none()
            && let Some(&type_id) = self.node_type_map.get(&node_id)
            && let Some(t) = self.get_type(type_id)
            && t.is_finished()
        {
            self.pending_type_checks.remove(&node_id);
            return type_id;
        }
        // Pre-registered shell - reuse the type_id and fill in details below

        // Get or create the model type
        let type_name = name.clone();
        let type_id = if ctx.mapper.is_none() && self.node_type_map.contains_key(&node_id) {
            // Reuse the pre-registered type (only when not instantiating)
            self.node_type_map[&node_id]
        } else {
            // When instantiating (mapper is Some), this is a template instance.
            // Set template_node to point to the original AST node so is_template_instance()
            // can identify it. TS: linkMapper sets templateNode = templateDeclaration.node.
            let template_node = if ctx.mapper.is_some() {
                Some(node_id)
            } else {
                None
            };
            let new_id = {
                let mut m = ModelType::new(
                    self.next_type_id(),
                    name,
                    Some(node_id),
                    self.current_namespace,
                );
                m.template_node = template_node;
                self.create_type(Type::Model(m))
            };
            // Only update node_type_map and declared_types for template declarations (no mapper).
            // For instantiations (with mapper), the instance is cached in symbol_links.instantiations
            // and should NOT overwrite the template declaration's registration.
            self.register_type(node_id, new_id, &type_name, ctx.mapper.as_ref());
            new_id
        };

        // Process directives (e.g., #deprecated) BEFORE checking properties,
        // so that the type is marked as deprecated when checking property type references.
        // This allows emit_deprecated_warning_if_needed to skip warnings when
        // the type reference is used within a deprecated context.
        self.process_and_mark_directives(node_id, type_id);

        // TS: linkMapper(type, ctx.mapper) — Propagate the mapper to inner types
        // created during template instantiation. In TS, checkModelExpression calls
        // linkMapper(type, ctx.mapper) which sets templateMapper on the inner model
        // expression, making isTemplateInstance() return true for it.
        if let Some(ref mapper) = ctx.mapper {
            self.link_template_mapper(type_id, mapper.clone());
        }

        // TS: linkType(ctx, links, type) — Pre-cache the instantiation result
        // BEFORE checking properties/heritage, so recursive references to this
        // template instantiation will find it in the cache instead of recursing.
        // This is how TS avoids infinite recursion for template self-references
        // like: model Templated<T> { parent: Templated<T>; }
        if let Some(ref mapper) = ctx.mapper {
            let links = self.symbol_links.entry(node_id).or_default();
            if links.instantiations.is_none() {
                links.instantiations = Some(HashMap::new());
            }
            if let Some(ref mut instantiations) = links.instantiations
                && !instantiations.contains_key(&mapper.args)
            {
                instantiations.insert(mapper.args.clone(), type_id);
            }
        }

        // Check extends heritage
        // TS: pendingResolutions.start(modelSymId, ResolutionKind.BaseType)
        if let Some(extends_id) = extends {
            // Mark this model as currently resolving its base type (for circular detection)
            if !type_name.is_empty() {
                self.pending_base_type_names.insert(type_name.clone());
            }

            let base_type = self.check_node(ctx, extends_id);

            // Remove from pending
            if !type_name.is_empty() {
                self.pending_base_type_names.remove(&type_name);
            }

            // If base_type is error_type, a diagnostic was already reported (e.g., circular-base-type)
            if base_type != self.error_type {
                let resolved_base_type = self.resolve_alias_chain(base_type);

                if let Some(Type::Model(base_model)) = self.get_type(resolved_base_type) {
                    // Check for circular base type chain
                    let is_circular = self.is_circular_base(type_id, resolved_base_type);

                    // Model expression (anonymous model) cannot be extended
                    let base_is_anonymous = base_model.name.is_empty();
                    let base_indexer = base_model.indexer;
                    if base_is_anonymous {
                        self.error("extend-model", "Models cannot extend model expressions.");
                    } else if is_circular {
                        self.error(
                            "circular-base-type",
                            &format!(
                                "Type '{}' recursively references itself as a base type.",
                                type_name
                            ),
                        );
                    } else {
                        if let Some(t) = self.get_type_mut(type_id)
                            && let Type::Model(m) = t
                        {
                            m.base_model = Some(resolved_base_type);
                            if base_indexer.is_some() {
                                m.indexer = base_indexer;
                            }
                        }
                        // Register as derived model
                        if let Some(t) = self.get_type_mut(resolved_base_type)
                            && let Type::Model(base) = t
                        {
                            base.derived_models.push(type_id);
                        }
                        // TS: copyDeprecation(type.baseModel, type)
                        self.copy_deprecation(resolved_base_type, type_id);
                    }
                } else if self.get_type(resolved_base_type).is_some() {
                    // Extending a non-model type
                    // TS: "Models must extend other models."
                    self.error("extend-model", "Models must extend other models.");
                }
            }
        }

        // Check 'is' heritage
        // TS: pendingResolutions.start(modelSymId, ResolutionKind.BaseType)
        if let Some(is_id) = is_node {
            // Check if the 'is' target is currently being type-checked (direct circular)
            if self.pending_type_checks.contains(&is_id) {
                self.error(
                    "circular-base-type",
                    &format!(
                        "Type '{}' recursively references itself as a base type.",
                        type_name
                    ),
                );
            } else {
                // Mark this model as currently resolving its base type (for circular detection)
                if !type_name.is_empty() {
                    self.pending_base_type_names.insert(type_name.clone());
                }

                let is_type = self.check_node(ctx, is_id);

                // Remove from pending
                if !type_name.is_empty() {
                    self.pending_base_type_names.remove(&type_name);
                }

                // If is_type is error_type, a diagnostic was already reported
                if is_type != self.error_type {
                    let resolved_is_type = self.resolve_alias_chain(is_type);

                    if let Some(Type::Model(is_model)) = self.get_type(resolved_is_type) {
                        // Check for circular base type chain
                        let is_circular = self.is_circular_base(type_id, resolved_is_type);

                        // Model expression (anonymous model) cannot be used with 'is'
                        let is_anonymous = is_model.name.is_empty();
                        // Extract data before mutable borrows
                        let is_properties: Vec<(String, TypeId)> = is_model
                            .property_names
                            .iter()
                            .filter_map(|n| is_model.properties.get(n).map(|&id| (n.clone(), id)))
                            .collect();
                        let is_base_model = is_model.base_model;
                        let is_indexer = is_model.indexer;
                        if is_anonymous {
                            self.error("is-model", "Model `is` cannot specify a model expression.");
                        } else if is_circular {
                            self.error(
                                "circular-base-type",
                                &format!(
                                    "Type '{}' recursively references itself as a base type.",
                                    type_name
                                ),
                            );
                        } else {
                            // Clone properties from 'is' model with proper re-parenting
                            let mut copied_props: Vec<(String, TypeId)> = Vec::new();
                            for (prop_name, prop_type_id) in &is_properties {
                                let cloned_prop_id = self.clone_type(*prop_type_id);
                                if let Some(prop) = self.get_type_mut(cloned_prop_id)
                                    && let Type::ModelProperty(p) = prop
                                {
                                    p.model = Some(type_id);
                                    p.source_property = Some(*prop_type_id);
                                }
                                copied_props.push((prop_name.clone(), cloned_prop_id));
                            }

                            if let Some(t) = self.get_type_mut(type_id)
                                && let Type::Model(m) = t
                            {
                                m.source_model = Some(resolved_is_type);
                                m.base_model = is_base_model;
                                if is_indexer.is_some() {
                                    m.indexer = is_indexer;
                                }
                                for (name, prop_id) in copied_props {
                                    m.properties.insert(name.clone(), prop_id);
                                    m.property_names.push(name);
                                }
                            }
                        }
                    } else if self.get_type(resolved_is_type).is_some() {
                        // 'is' targets a non-model type
                        self.error("is-model", "Model `is` must specify another model.");
                    }
                }
            }
        }

        // Check properties (including spread properties)
        for prop_id in &properties {
            let prop_node = ast.id_to_node(*prop_id).cloned();
            match prop_node {
                Some(AstNode::ModelProperty(prop_node)) => {
                    let prop_name = Self::get_identifier_name(&ast, prop_node.name);

                    // Check for duplicate property before inserting
                    // TS: checkModelProperties → if properties.has(newProp.name) → report "duplicate-property"
                    if let Some(t) = self.get_type(type_id)
                        && let Type::Model(m) = t
                        && m.properties.contains_key(&prop_name)
                    {
                        let model_name = type_name.clone();
                        self.error(
                            "duplicate-property",
                            &Self::format_duplicate_property_msg(&model_name, &prop_name),
                        );
                        continue; // Skip adding the duplicate
                    }

                    // Pre-register a placeholder property in the model's properties map
                    // so that member access (e.g., A.a) can find it during circular-prop
                    // detection. The property type will be updated after check_model_property.
                    // This mirrors TS's approach where properties are registered before
                    // their types are fully resolved.
                    let placeholder_id = self.create_type(Type::ModelProperty(ModelPropertyType {
                        id: self.next_type_id(),
                        name: prop_name.clone(),
                        node: Some(*prop_id),
                        r#type: self.error_type,
                        optional: prop_node.optional,
                        default_value: None,
                        model: Some(type_id),
                        source_property: None,
                        decorators: Vec::new(),
                        is_finished: false,
                    }));

                    // Insert placeholder into model's properties
                    if let Some(t) = self.get_type_mut(type_id)
                        && let Type::Model(m) = t
                    {
                        m.properties.insert(prop_name.clone(), placeholder_id);
                        m.property_names.push(prop_name.clone());
                    }

                    // Now fully check the property (which will update the placeholder)
                    let prop_type_id =
                        self.check_model_property_with_placeholder(ctx, *prop_id, placeholder_id);
                    // Check for override-property-mismatch
                    // Ported from TS checker.ts defineProperty → getOverriddenProperty → isTypeAssignableTo
                    if let Some(overridden_prop_id) =
                        self.get_overridden_property(type_id, &prop_name)
                    {
                        self.check_override_property_mismatch(
                            prop_type_id,
                            overridden_prop_id,
                            &prop_name,
                        );
                    }
                    // Check property compatibility with model indexer
                    // Ported from TS checker.ts checkPropertyCompatibleWithModelIndexer
                    self.check_property_compatible_with_model_indexer(type_id, prop_type_id);
                }
                Some(AstNode::ModelSpreadProperty(spread_node)) => {
                    // Ported from TS checker.ts checkSpreadProperty / checkSpreadTarget
                    let target_type_id = self.check_node(ctx, spread_node.target);

                    // Resolve through alias chains (e.g., alias Spread = { ...Source })
                    // so spreading an alias that points to a model works correctly.
                    let resolved_target_id = self.resolve_alias_chain(target_type_id);
                    let target_type = self.get_type(resolved_target_id).cloned();

                    let is_error_type = matches!(&target_type, Some(Type::Intrinsic(i)) if i.name == IntrinsicTypeName::ErrorType);

                    match target_type {
                        Some(_) if is_error_type => {
                            // Error type - skip silently
                        }
                        None => {
                            // Unresolved - skip silently
                        }
                        Some(Type::TemplateParameter(_)) => {
                            // Template parameter - skip silently
                        }
                        Some(Type::Model(ref target_model)) if !target_model.is_finished => {
                            // Check if spreading itself first (even if not finished)
                            if resolved_target_id == type_id {
                                self.error(
                                    "spread-model",
                                    "Cannot spread type within its own declaration.",
                                );
                                continue;
                            }
                            // Target model is still being created (forward reference).
                            // TS: ensureResolved() defers property copying until the target
                            // is finished. We record this as a pending spread to be resolved
                            // when the target type is finished.
                            self.pending_spreads
                                .entry(resolved_target_id)
                                .or_default()
                                .push(type_id);
                        }
                        Some(Type::Model(target_model)) => {
                            // Check if spreading itself
                            if resolved_target_id == type_id {
                                self.error(
                                    "spread-model",
                                    "Cannot spread type within its own declaration.",
                                );
                                continue;
                            }

                            // Check if target is an array model (has indexer with key named "integer")
                            let is_array =
                                type_utils::is_array_model_type(&self.type_store, &target_model);
                            if is_array {
                                self.error(
                                    "spread-model",
                                    "Cannot spread properties of non-model type.",
                                );
                                continue;
                            }

                            // Copy properties from the spread target (including inherited)
                            // TS: walkPropertiesInherited(targetType) + cloneTypeForSymbol
                            let prop_names_to_copy: Vec<(String, TypeId)> = {
                                let mut result = Vec::new();
                                // Walk inherited properties (self + base chain)
                                let mut current: Option<TypeId> = Some(resolved_target_id);
                                while let Some(mid) = current {
                                    if let Some(Type::Model(cur_model)) = self.get_type(mid) {
                                        for (name, &prop_id) in &cur_model.properties {
                                            result.push((name.clone(), prop_id));
                                        }
                                        current = cur_model.base_model;
                                    } else {
                                        break;
                                    }
                                }
                                result
                            };

                            for (prop_name, src_prop_id) in prop_names_to_copy {
                                // Check for duplicate
                                if let Some(t) = self.get_type(type_id)
                                    && let Type::Model(m) = t
                                    && m.properties.contains_key(&prop_name)
                                {
                                    self.error(
                                        "duplicate-property",
                                        &Self::format_duplicate_property_msg(
                                            &type_name, &prop_name,
                                        ),
                                    );
                                    continue;
                                }
                                // Clone the property and set source_property + model references
                                // TS: cloneTypeForSymbol(memberSym, prop, { sourceProperty: prop, model: parentModel })
                                let cloned_id = self.clone_type(src_prop_id);
                                if let Some(prop) = self.get_type_mut(cloned_id)
                                    && let Type::ModelProperty(p) = prop
                                {
                                    p.source_property = Some(src_prop_id);
                                    p.model = Some(type_id);
                                }
                                if let Some(t) = self.get_type_mut(type_id)
                                    && let Type::Model(m) = t
                                {
                                    m.properties.insert(prop_name.clone(), cloned_id);
                                    m.property_names.push(prop_name);
                                }
                            }

                            // If the spread target has an indexer (e.g., Record), copy it
                            if let Some((key_id, value_id)) = target_model.indexer
                                && let Some(t) = self.get_type_mut(type_id)
                                && let Type::Model(m) = t
                            {
                                m.indexer = Some((key_id, value_id));
                            }

                            // Record spread source for cascade propagation
                            self.spread_sources
                                .entry(resolved_target_id)
                                .or_default()
                                .push(type_id);
                        }
                        _ => {
                            // Non-model type - report spread-model diagnostic
                            self.error(
                                "spread-model",
                                "Cannot spread properties of non-model type.",
                            );
                        }
                    }
                }
                _ => {
                    // Unknown property type - skip
                }
            }
        }

        // TS: checkPropertyCompatibleWithIndexer - if model has integer indexer (is array)
        // and has properties, report no-array-properties
        if let Some(t) = self.get_type(type_id)
            && let Type::Model(m) = t
            && type_utils::is_array_model_type(&self.type_store, m)
            && !m.properties.is_empty()
        {
            self.error(
                "no-array-properties",
                "Array models cannot have any properties.",
            );
        }

        // Set template_node:
        // Both template declarations and instances set template_node = Some(node_id).
        // - For declarations (mapper is None): node_id IS the declaration node, so template_node points to itself.
        // - For instances (mapper is Some): node_id is also the declaration node, because
        //   instantiate_template() calls check_node(inst_ctx, template_node_id) where
        //   template_node_id is the original declaration node.
        // In TS, template_node is set on both declarations and instances via createType().
        if has_template_params
            && let Some(t) = self.get_type_mut(type_id)
            && let Type::Model(m) = t
        {
            m.template_node = Some(node_id);
        }

        // Finalize the type - processes decorators, finishes template handling,
        // and removes from pending checks.
        // For template declarations (mapper is None and has template params),
        // finish_template_or_type skips decorators so isFinished stays false,
        // meaning the semantic walker skips template declarations by default.
        // Template INSTANCES (mapper is Some) should still be finished.
        self.finalize_type_check(
            ctx,
            type_id,
            node_id,
            &template_param_ids,
            &decorators,
            ctx.mapper.as_ref(),
        );

        type_id
    }

    /// Get the overridden property from the base model chain.
    /// Ported from TS checker.ts getOverriddenProperty()
    pub fn get_overridden_property(
        &self,
        model_type_id: TypeId,
        prop_name: &str,
    ) -> Option<TypeId> {
        let base_model_id = match self.get_type(model_type_id) {
            Some(Type::Model(m)) => m.base_model,
            _ => return None,
        };

        let mut current = base_model_id;
        while let Some(cur_id) = current {
            match self.get_type(cur_id) {
                Some(Type::Model(m)) => {
                    if let Some(&overridden_id) = m.properties.get(prop_name) {
                        return Some(overridden_id);
                    }
                    current = m.base_model;
                }
                _ => break,
            }
        }
        None
    }

    /// Check if a property override is compatible with the base property.
    /// Ported from TS checker.ts defineProperty() → override-property-mismatch checks
    pub(crate) fn check_override_property_mismatch(
        &mut self,
        new_prop_id: TypeId,
        overridden_prop_id: TypeId,
        prop_name: &str,
    ) {
        let (new_prop_type, new_prop_optional) = match self.get_type(new_prop_id) {
            Some(Type::ModelProperty(p)) => (p.r#type, p.optional),
            _ => return,
        };

        let (overridden_prop_type, overridden_optional) = match self.get_type(overridden_prop_id) {
            Some(Type::ModelProperty(p)) => (p.r#type, p.optional),
            _ => return,
        };

        // Check type assignability
        // TS: relation.isTypeAssignableTo(newProp.type, overriddenProp.type, newProp)
        if new_prop_type != overridden_prop_type {
            let is_assignable = self
                .type_relation
                .is_related_with_store(&self.type_store, new_prop_type, overridden_prop_type)
                .is_true();

            if !is_assignable {
                let parent_type_name = self.get_type_name_for_diagnostic(overridden_prop_type);
                let new_type_name = self.get_type_name_for_diagnostic(new_prop_type);
                self.error("override-property-mismatch", &format!("Property '{}' has type {} which is not assignable to base property type {}", prop_name, new_type_name, parent_type_name));
            }
        }

        // Check optional override: parent is required but override is optional
        // TS: if !overriddenProp.optional && newProp.optional
        if !overridden_optional && new_prop_optional {
            self.error(
                "override-property-mismatch",
                &format!(
                    "Property '{}' cannot be optional because the base property is required",
                    prop_name
                ),
            );
        }
    }

    /// Get a human-readable type name for diagnostic messages
    pub(crate) fn get_type_name_for_diagnostic(&self, type_id: TypeId) -> String {
        type_utils::get_fully_qualified_name(&self.type_store, type_id)
    }

    /// Check a model property using an existing placeholder type.
    /// The placeholder was already created and inserted into the model's properties map
    /// before calling this method, so that member access (e.g., A.a) can find it during
    /// circular-prop detection.
    pub(crate) fn check_model_property_with_placeholder(
        &mut self,
        ctx: &CheckContext,
        node_id: NodeId,
        placeholder_id: TypeId,
    ) -> TypeId {
        let (ast, node) = require_ast_node!(self, node_id, ModelProperty, placeholder_id);

        // Check if already type-checked (fully finished)
        if ctx.mapper.is_none()
            && let Some(t) = self.get_type(placeholder_id)
            && t.is_finished()
        {
            return placeholder_id;
        }

        let name = Self::get_identifier_name(&ast, node.name);

        // Circular reference detection: if we're already checking this property node,
        // the property type recursively references itself.
        if self.pending_type_checks.contains(&node_id) {
            self.error(
                "circular-prop",
                &format!("Property '{}' recursively references itself.", name),
            );
            return self.error_type;
        }

        // Map the node to the existing placeholder type
        self.node_type_map.insert(node_id, placeholder_id);

        // Mark this property as being checked (for circular-prop detection)
        self.pending_type_checks.insert(node_id);

        // Check the value type expression
        let value_type = self.check_node(ctx, node.value);

        // Remove from pending checks
        self.pending_type_checks.remove(&node_id);

        // Update the placeholder property's type
        if let Some(t) = self.get_type_mut(placeholder_id)
            && let Type::ModelProperty(p) = t
        {
            p.r#type = value_type;
        }

        // Check default value
        if let Some(default_id) = node.default
            && value_type != self.error_type
        {
            let default_entity = self.check_node_entity(ctx, default_id);
            let default_type_id = self.entity_to_type_id(&default_entity);

            let (is_assignable, _) =
                self.is_type_assignable_to(default_type_id, value_type, default_id);
            if !is_assignable {
                let default_type_name = self.get_type_name_for_diagnostic(default_type_id);
                let prop_type_name = self.get_type_name_for_diagnostic(value_type);
                self.error(
                    "unassignable",
                    &format!(
                        "Default value of type '{}' is not assignable to type '{}'",
                        default_type_name, prop_type_name
                    ),
                );
            }

            if let Some(t) = self.get_type_mut(placeholder_id)
                && let Type::ModelProperty(p) = t
            {
                p.default_value = Some(default_type_id);
            }
        }

        // Check decorators
        self.check_and_store_decorators(ctx, placeholder_id, &node.decorators);

        self.finish_type(placeholder_id);
        placeholder_id
    }

    /// Recursively collect template parameter references from AST nodes
    pub(crate) fn collect_template_param_refs(
        &self,
        ast: &AstBuilder,
        node_id: NodeId,
        param_names: &[String],
        used: &mut HashSet<String>,
    ) {
        match ast.id_to_node(node_id) {
            Some(AstNode::Identifier(ident)) if param_names.contains(&ident.value) => {
                used.insert(ident.value.clone());
            }
            Some(AstNode::TypeReference(tr)) => {
                self.collect_template_param_refs(ast, tr.name, param_names, used);
                for &arg_id in &tr.arguments {
                    if let Some(AstNode::TemplateArgument(ta)) = ast.id_to_node(arg_id) {
                        if let Some(name_id) = ta.name {
                            self.collect_template_param_refs(ast, name_id, param_names, used);
                        }
                        self.collect_template_param_refs(ast, ta.argument, param_names, used);
                    }
                }
            }
            Some(AstNode::MemberExpression(me)) => {
                self.collect_template_param_refs(ast, me.object, param_names, used);
                self.collect_template_param_refs(ast, me.property, param_names, used);
            }
            Some(AstNode::ModelProperty(mp)) => {
                self.collect_template_param_refs(ast, mp.value, param_names, used);
            }
            Some(AstNode::ModelSpreadProperty(ms)) => {
                self.collect_template_param_refs(ast, ms.target, param_names, used);
            }
            Some(AstNode::UnionExpression(ue)) => {
                for &opt_id in &ue.options {
                    self.collect_template_param_refs(ast, opt_id, param_names, used);
                }
            }
            Some(AstNode::IntersectionExpression(ie)) => {
                for &opt_id in &ie.options {
                    self.collect_template_param_refs(ast, opt_id, param_names, used);
                }
            }
            Some(AstNode::ArrayExpression(ae)) => {
                self.collect_template_param_refs(ast, ae.element_type, param_names, used);
            }
            Some(AstNode::TupleExpression(te)) => {
                for &val_id in &te.values {
                    self.collect_template_param_refs(ast, val_id, param_names, used);
                }
            }
            Some(AstNode::ValueOfExpression(ve)) => {
                self.collect_template_param_refs(ast, ve.target, param_names, used);
            }
            Some(AstNode::TypeOfExpression(te)) => {
                self.collect_template_param_refs(ast, te.target, param_names, used);
            }
            Some(AstNode::DecoratorExpression(de)) => {
                for &arg_id in &de.arguments {
                    self.collect_template_param_refs(ast, arg_id, param_names, used);
                }
            }
            Some(AstNode::OperationDeclaration(decl)) => {
                self.collect_template_param_refs(ast, decl.signature, param_names, used);
                for &dec_id in &decl.decorators {
                    self.collect_template_param_refs(ast, dec_id, param_names, used);
                }
            }
            Some(AstNode::OperationSignatureDeclaration(sig)) => {
                self.collect_template_param_refs(ast, sig.parameters, param_names, used);
                self.collect_template_param_refs(ast, sig.return_type, param_names, used);
            }
            Some(AstNode::OperationSignatureReference(sig)) => {
                self.collect_template_param_refs(ast, sig.base_operation, param_names, used);
            }
            Some(AstNode::InterfaceDeclaration(decl)) => {
                for &op_id in &decl.operations {
                    self.collect_template_param_refs(ast, op_id, param_names, used);
                }
                for &ext_id in &decl.extends {
                    self.collect_template_param_refs(ast, ext_id, param_names, used);
                }
                for &dec_id in &decl.decorators {
                    self.collect_template_param_refs(ast, dec_id, param_names, used);
                }
            }
            Some(AstNode::ModelExpression(me)) => {
                for &prop_id in &me.properties {
                    self.collect_template_param_refs(ast, prop_id, param_names, used);
                }
            }
            Some(AstNode::UnionVariant(uv)) => {
                self.collect_template_param_refs(ast, uv.value, param_names, used);
            }
            Some(AstNode::EnumMember(em)) => {
                if let Some(val) = em.value {
                    self.collect_template_param_refs(ast, val, param_names, used);
                }
            }
            _ => {}
        }
    }
}
