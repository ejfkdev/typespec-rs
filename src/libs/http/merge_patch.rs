//! JSON Merge Patch transform.
//!
//! Ported from `packages/http/src/merge-patch.ts`. The upstream implementation
//! uses the compiler mutator engine; this port builds the transformed model
//! graph directly. Visibility filtering and the full mutator pipeline are not
//! implemented yet, but the public shape (optional/nullable properties,
//! recursive keyed types, media-type hint, source links, and diagnostics)
//! matches the upstream transform.

use std::collections::{HashMap, HashSet};

use crate::checker::CheckContext;
use crate::checker::Checker;
use crate::checker::types::{
    ModelPropertyType, ModelType, Type, TypeId, UnionType, UnionVariantType,
};
use crate::state_accessors::StateAccessors;

use super::types::{
    DIAG_MERGE_PATCH_CONTAINS_METADATA, DIAG_MERGE_PATCH_CONTAINS_NULL, STATE_MERGE_PATCH_MODEL,
    STATE_MERGE_PATCH_PROPERTY,
};

const MERGE_PATCH_MEDIA_TYPE: &str = "application/merge-patch+json";

/// Get the source model of a merge-patch model.
pub fn get_merge_patch_source(state: &StateAccessors, model_id: TypeId) -> Option<TypeId> {
    state
        .get_state(STATE_MERGE_PATCH_MODEL, model_id)
        .and_then(|value| value.parse::<TypeId>().ok())
}

/// Whether a model is the result of a merge-patch transform.
pub fn is_merge_patch(state: &StateAccessors, model_id: TypeId) -> bool {
    get_merge_patch_source(state, model_id).is_some()
}

/// Get the source property of a merge-patch property.
pub fn get_merge_patch_property_source(
    state: &StateAccessors,
    property_id: TypeId,
) -> Option<TypeId> {
    state
        .get_state(STATE_MERGE_PATCH_PROPERTY, property_id)
        .and_then(|value| value.parse::<TypeId>().ok())
}

/// Whether a type is part of a merge-patch body (model, union variant, tuple
/// element, array/record element, or a model containing transformed
/// properties).
pub fn is_merge_patch_body(checker: &Checker, state: &StateAccessors, type_id: TypeId) -> bool {
    fn check(
        checker: &Checker,
        state: &StateAccessors,
        type_id: TypeId,
        visited: &mut HashSet<TypeId>,
    ) -> bool {
        let resolved = checker.resolve_alias_chain(type_id);
        if !visited.insert(resolved) {
            return false;
        }
        match checker.get_type(resolved) {
            Some(Type::Model(model)) => {
                if is_merge_patch(state, resolved) {
                    return true;
                }
                if let Some((_, value)) = model.indexer
                    && check(checker, state, value, visited)
                {
                    return true;
                }
                if let Some(source) = model.source_model
                    && check(checker, state, source, visited)
                {
                    return true;
                }
                for name in &model.property_names {
                    if let Some(&prop_id) = model.properties.get(name) {
                        if get_merge_patch_property_source(state, prop_id).is_some() {
                            return true;
                        }
                        if let Some(Type::ModelProperty(prop)) = checker.get_type(prop_id)
                            && check(checker, state, prop.r#type, visited)
                        {
                            return true;
                        }
                    }
                }
                false
            }
            Some(Type::Union(union)) => union.variant_names.iter().any(|name| {
                union.variants.get(name).is_some_and(|&variant_id| {
                    matches!(checker.get_type(variant_id), Some(Type::UnionVariant(v)) if check(checker, state, v.r#type, visited))
                })
            }),
            Some(Type::ModelProperty(prop)) => check(checker, state, prop.r#type, visited),
            Some(Type::UnionVariant(variant)) => check(checker, state, variant.r#type, visited),
            Some(Type::Tuple(tuple)) => tuple
                .values
                .iter()
                .any(|&value| check(checker, state, value, visited)),
            _ => false,
        }
    }

    check(checker, state, type_id, &mut HashSet::new())
}

fn render_name(template: &str, source_name: &str) -> String {
    template.replace("{name}", source_name)
}

fn diagnostic_metadata_kind(state: &StateAccessors, prop_id: TypeId) -> Option<&'static str> {
    if super::is_path(state, prop_id) {
        Some("@path")
    } else if super::is_header(state, prop_id) {
        Some("@header")
    } else if super::is_cookie(state, prop_id) {
        Some("@cookie")
    } else if super::is_query(state, prop_id) {
        Some("@query")
    } else if super::is_status_code(state, prop_id) {
        Some("@statusCode")
    } else {
        None
    }
}

fn contains_null(checker: &Checker, type_id: TypeId, visited: &mut HashSet<TypeId>) -> bool {
    let resolved = checker.resolve_alias_chain(type_id);
    if !visited.insert(resolved) {
        return false;
    }
    match checker.get_type(resolved) {
        Some(Type::Intrinsic(intrinsic)) => intrinsic.name == crate::checker::IntrinsicTypeName::Null,
        Some(Type::Model(model)) => {
            if let Some((_, value)) = model.indexer
                && contains_null(checker, value, visited)
            {
                return true;
            }
            model.property_names.iter().any(|name| {
                model.properties.get(name).is_some_and(|&prop_id| {
                    matches!(checker.get_type(prop_id), Some(Type::ModelProperty(prop)) if contains_null(checker, prop.r#type, visited))
                })
            })
        }
        Some(Type::ModelProperty(prop)) => contains_null(checker, prop.r#type, visited),
        Some(Type::Union(union)) => union.variant_names.iter().any(|name| {
            union.variants.get(name).is_some_and(|&variant_id| {
                matches!(checker.get_type(variant_id), Some(Type::UnionVariant(v)) if contains_null(checker, v.r#type, visited))
            })
        }),
        Some(Type::UnionVariant(variant)) => contains_null(checker, variant.r#type, visited),
        Some(Type::Tuple(tuple)) => tuple
            .values
            .iter()
            .any(|&value| contains_null(checker, value, visited)),
        _ => false,
    }
}

struct Transform<'a> {
    checker: &'a mut Checker,
    name_template: String,
    cache: HashMap<(TypeId, bool), TypeId>,
}

impl Transform<'_> {
    fn nullable(&mut self, base: TypeId) -> TypeId {
        let id = self.checker.next_type_id();
        let mut union = UnionType::new(id, String::new(), None, None, true);
        let base_variant_id = self.checker.next_type_id();
        let base_variant = UnionVariantType {
            id: base_variant_id,
            name: String::new(),
            node: None,
            r#type: base,
            union: Some(id),
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: true,
        };
        let base_variant = self.checker.create_type(Type::UnionVariant(base_variant));
        let null_variant_id = self.checker.next_type_id();
        let null_variant = UnionVariantType {
            id: null_variant_id,
            name: String::new(),
            node: None,
            r#type: self.checker.null_type,
            union: Some(id),
            decorators: Vec::new(),
            doc: None,
            summary: None,
            is_finished: true,
        };
        let null_variant = self.checker.create_type(Type::UnionVariant(null_variant));
        union
            .variants
            .insert("\u{0}value".to_string(), base_variant);
        union.variants.insert("\u{0}null".to_string(), null_variant);
        union.variant_names.push("\u{0}value".to_string());
        union.variant_names.push("\u{0}null".to_string());
        union.is_finished = true;
        self.checker.create_type(Type::Union(union))
    }

    fn transform(&mut self, type_id: TypeId, optionalize: bool) -> TypeId {
        let resolved = self.checker.resolve_alias_chain(type_id);
        if let Some(&cached) = self.cache.get(&(resolved, optionalize)) {
            return cached;
        }
        match self.checker.get_type(resolved).cloned() {
            Some(Type::Model(model)) => {
                if model.indexer.is_some() {
                    self.transform_array_or_record(resolved, &model, optionalize)
                } else {
                    self.transform_model(resolved, &model, optionalize)
                }
            }
            Some(Type::Union(union)) => self.transform_union(resolved, &union),
            Some(Type::Tuple(tuple)) => {
                let mut values = Vec::with_capacity(tuple.values.len());
                for value in &tuple.values {
                    values.push(self.transform(*value, false));
                }
                let id = self.checker.next_type_id();
                let mut clone = tuple.clone();
                clone.id = id;
                clone.values = values;
                clone.is_finished = true;
                let clone_id = self.checker.create_type(Type::Tuple(clone));
                self.cache.insert((resolved, optionalize), clone_id);
                clone_id
            }
            Some(Type::ModelProperty(prop)) => {
                let new_type = self.transform(prop.r#type, optionalize);
                let id = self.checker.next_type_id();
                let mut clone = prop.clone();
                clone.id = id;
                clone.r#type = new_type;
                clone.is_finished = true;
                let clone_id = self.checker.create_type(Type::ModelProperty(clone));
                self.checker.state_accessors.set_state(
                    STATE_MERGE_PATCH_PROPERTY,
                    clone_id,
                    resolved.to_string(),
                );
                self.cache.insert((resolved, optionalize), clone_id);
                clone_id
            }
            _ => resolved,
        }
    }

    fn transform_array_or_record(
        &mut self,
        source_id: TypeId,
        model: &ModelType,
        optionalize: bool,
    ) -> TypeId {
        let (key, value) = model.indexer.unwrap_or((source_id, source_id));
        let is_record = self
            .checker
            .get_type(self.checker.resolve_alias_chain(key))
            .is_some_and(|t| matches!(t, Type::Scalar(s) if s.name == "string"));
        let _ = optionalize;
        let new_value = self.transform(value, is_record);
        let id = self.checker.next_type_id();
        let mut clone = model.clone();
        clone.id = id;
        clone.indexer = Some((key, new_value));
        clone.is_finished = true;
        let clone_id = self.checker.create_type(Type::Model(clone));
        self.cache.insert((source_id, optionalize), clone_id);
        clone_id
    }

    fn transform_model(
        &mut self,
        source_id: TypeId,
        model: &ModelType,
        optionalize: bool,
    ) -> TypeId {
        let name = render_name(&self.name_template, &model.name);
        let id = self.checker.next_type_id();
        let mut clone = ModelType::new(id, name.clone(), None, model.namespace);
        clone.doc = model.doc.clone();
        clone.summary = model.summary.clone();
        clone.decorators = model.decorators.clone();
        clone.is_finished = true;
        let clone_id = self.checker.create_type(Type::Model(clone));
        self.cache.insert((source_id, optionalize), clone_id);
        self.checker.type_display_names.insert(clone_id, name);

        let base = model
            .base_model
            .map(|base_id| self.transform(base_id, optionalize));

        let mut properties = HashMap::new();
        let mut property_names = Vec::new();
        for property_name in &model.property_names {
            let Some(&source_prop_id) = model.properties.get(property_name) else {
                continue;
            };
            let Some(Type::ModelProperty(source_prop)) =
                self.checker.get_type(source_prop_id).cloned()
            else {
                continue;
            };
            if let Some(metadata) =
                diagnostic_metadata_kind(&self.checker.state_accessors, source_prop_id)
            {
                let target = source_prop.node.unwrap_or(0);
                self.checker.error_at(
                    target,
                    DIAG_MERGE_PATCH_CONTAINS_METADATA,
                    &format!(
                        "The MergePatch transform does not operate on http envelope metadata.  Remove any http metadata decorators ('@query', '@header', '@path', '@cookie', '@statusCode') from the model passed to the MergePatch template. Found '{metadata}' decorating property '{property_name}'"
                    ),
                );
                continue;
            }
            let source_type = source_prop.r#type;
            let new_type = self.transform(source_type, true);
            let new_type =
                if optionalize && (source_prop.optional || source_prop.default_value.is_some()) {
                    self.nullable(new_type)
                } else {
                    new_type
                };
            let prop_id = self.checker.next_type_id();
            let mut prop = ModelPropertyType {
                id: prop_id,
                name: source_prop.name.clone(),
                node: source_prop.node,
                r#type: new_type,
                optional: if optionalize {
                    true
                } else {
                    source_prop.optional
                },
                default_value: if optionalize {
                    None
                } else {
                    source_prop.default_value
                },
                model: Some(clone_id),
                source_property: Some(source_prop_id),
                decorators: source_prop.decorators.clone(),
                doc: source_prop.doc.clone(),
                summary: source_prop.summary.clone(),
                is_finished: true,
            };
            prop.source_property = Some(source_prop_id);
            let new_prop_id = self.checker.create_type(Type::ModelProperty(prop));
            self.checker
                .state_accessors
                .copy_type_state(source_prop_id, new_prop_id);
            self.checker.state_accessors.set_state(
                STATE_MERGE_PATCH_PROPERTY,
                new_prop_id,
                source_prop_id.to_string(),
            );
            properties.insert(property_name.clone(), new_prop_id);
            property_names.push(property_name.clone());
        }

        if let Some(Type::Model(target)) = self.checker.get_type_mut(clone_id) {
            target.base_model = base;
            target.properties = properties;
            target.property_names = property_names;
        }
        if let Some(base_id) = base
            && let Some(Type::Model(base_model)) = self.checker.get_type_mut(base_id)
        {
            base_model.derived_models.push(clone_id);
        }
        self.checker.state_accessors.set_state(
            STATE_MERGE_PATCH_MODEL,
            clone_id,
            source_id.to_string(),
        );
        self.checker.state_accessors.set_state(
            crate::libs::compiler::STATE_MEDIA_TYPE_HINT,
            clone_id,
            MERGE_PATCH_MEDIA_TYPE.to_string(),
        );
        clone_id
    }

    fn transform_union(&mut self, source_id: TypeId, union: &UnionType) -> TypeId {
        let name = if union.name.is_empty() {
            String::new()
        } else {
            render_name(&self.name_template, &union.name)
        };
        let id = self.checker.next_type_id();
        let mut clone = UnionType::new(id, name.clone(), None, union.namespace, union.expression);
        clone.doc = union.doc.clone();
        clone.summary = union.summary.clone();
        clone.decorators = union.decorators.clone();
        clone.is_finished = true;
        let clone_id = self.checker.create_type(Type::Union(clone));
        self.cache.insert((source_id, true), clone_id);

        let mut variants = HashMap::new();
        let mut variant_names = Vec::new();
        for variant_name in &union.variant_names {
            let Some(&source_variant_id) = union.variants.get(variant_name) else {
                continue;
            };
            let Some(Type::UnionVariant(variant)) =
                self.checker.get_type(source_variant_id).cloned()
            else {
                continue;
            };
            let new_type = self.transform(variant.r#type, true);
            let variant_id = self.checker.next_type_id();
            let mut new_variant = variant;
            new_variant.id = variant_id;
            new_variant.r#type = new_type;
            new_variant.union = Some(clone_id);
            new_variant.is_finished = true;
            let new_variant_id = self.checker.create_type(Type::UnionVariant(new_variant));
            variants.insert(variant_name.clone(), new_variant_id);
            variant_names.push(variant_name.clone());
        }
        if let Some(Type::Union(target)) = self.checker.get_type_mut(clone_id) {
            target.variants = variants;
            target.variant_names = variant_names;
        }
        if !name.is_empty() {
            self.checker.state_accessors.set_state(
                crate::libs::compiler::STATE_MEDIA_TYPE_HINT,
                clone_id,
                MERGE_PATCH_MEDIA_TYPE.to_string(),
            );
            self.checker.type_display_names.insert(clone_id, name);
        }
        clone_id
    }
}

/// Apply the merge-patch transform to a model.
pub fn apply_merge_patch_transform(
    checker: &mut Checker,
    source_id: TypeId,
    name_template: &str,
) -> TypeId {
    if contains_null(checker, source_id, &mut HashSet::new()) {
        let target = checker
            .get_type(source_id)
            .and_then(|t| t.node_id_from_type())
            .unwrap_or(0);
        checker.error_at(
            target,
            DIAG_MERGE_PATCH_CONTAINS_NULL,
            "Cannot convert model to a merge-patch compatible shape because it contains the 'null' intrinsic type.",
        );
    }
    let resolved = checker.resolve_alias_chain(source_id);
    let mut transform = Transform {
        checker,
        name_template: name_template.to_string(),
        cache: HashMap::new(),
    };
    transform.transform(resolved, true)
}

/// Move a transformed model's properties onto the decorator target model.
pub fn adopt_transformed_model(
    checker: &mut Checker,
    target_id: TypeId,
    source_id: TypeId,
    name_template: &str,
) {
    let transformed_id = apply_merge_patch_transform(checker, source_id, name_template);
    let transformed = match checker.get_type(transformed_id) {
        Some(Type::Model(model)) => model.clone(),
        _ => return,
    };
    let mut properties = HashMap::new();
    let mut property_names = Vec::new();
    for name in &transformed.property_names {
        if let Some(&prop_id) = transformed.properties.get(name) {
            if let Some(Type::ModelProperty(prop)) = checker.get_type_mut(prop_id) {
                prop.model = Some(target_id);
            }
            properties.insert(name.clone(), prop_id);
            property_names.push(name.clone());
        }
    }
    if let Some(Type::Model(target)) = checker.get_type_mut(target_id) {
        target.properties = properties;
        target.property_names = property_names;
    }
    checker
        .state_accessors
        .set_state(STATE_MERGE_PATCH_MODEL, target_id, source_id.to_string());
    checker.state_accessors.set_state(
        crate::libs::compiler::STATE_MEDIA_TYPE_HINT,
        target_id,
        MERGE_PATCH_MEDIA_TYPE.to_string(),
    );
}

/// Check a call to `applyMergePatchTransform(T, NameTemplate, mode)` used by
/// the `MergePatchUpdate` / `MergePatchCreateOrUpdate` aliases.
pub fn check_merge_patch_call(
    checker: &mut Checker,
    ctx: &CheckContext,
    node_id: crate::ast::NodeId,
    arguments: &[crate::ast::NodeId],
) -> TypeId {
    let Some(&source_arg) = arguments.first() else {
        checker.error_at(
            node_id,
            "invalid-argument-count",
            "Expected at least 1 arguments, but got 0.",
        );
        return checker.error_type;
    };
    let source_type = checker.check_node(ctx, source_arg);
    let resolved = checker.resolve_alias_chain(source_type);
    if !matches!(checker.get_type(resolved), Some(Type::Model(_))) {
        // The alias body is also checked against its template parameters;
        // defer until the parameter is mapped to a concrete model.
        if matches!(checker.get_type(resolved), Some(Type::TemplateParameter(_))) {
            return resolved;
        }
        checker.error_at(
            node_id,
            "invalid-argument",
            "Argument of type is not assignable to parameter of type 'Reflection.Model'",
        );
        return checker.error_type;
    }
    let name_template = if let Some(&arg) = arguments.get(1) {
        let name_type = checker.check_node(ctx, arg);
        match checker.get_type(checker.resolve_alias_chain(name_type)) {
            Some(Type::String(value)) => value.value.clone(),
            _ => "{name}MergePatchUpdate".to_string(),
        }
    } else {
        "{name}MergePatchUpdate".to_string()
    };
    apply_merge_patch_transform(checker, resolved, &name_template)
}
