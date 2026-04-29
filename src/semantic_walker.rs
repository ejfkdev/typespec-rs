//! Semantic walker for TypeSpec-Rust
//! Ported from TypeSpec compiler/src/core/semantic-walker.ts
//!
//! Traverses the TypeSpec type graph, visiting types in a structured way.

use crate::checker::Checker;
use crate::checker::types::{Type, TypeId};
use std::collections::HashSet;

/// Control flow for listener callbacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListenerFlow {
    /// Continue visiting child types
    #[default]
    Continue,
    /// Skip visiting child types (no recursion into this type)
    NoRecursion,
}

/// Navigation options for controlling traversal behavior
#[derive(Debug, Clone, Default)]
pub struct NavigationOptions {
    /// Skip non-instantiated templates (default: false)
    pub include_template_declaration: bool,
    /// Visit derived types (default: false)
    pub visit_derived_types: bool,
}

/// Semantic node listener - callbacks for each type kind
/// Ported from TS SemanticNodeListener interface
pub trait SemanticNodeListener {
    fn on_root(&mut self, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_namespace(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_namespace(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_model(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_model(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_model_property(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_model_property(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_scalar(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_scalar(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_interface(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_interface(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_enum(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_enum(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_enum_member(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_enum_member(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_operation(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_operation(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_union(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_union(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_union_variant(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_union_variant(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_tuple(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_exit_tuple(&mut self, _type_id: TypeId, _checker: &Checker) {}
    fn on_string_template(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_string_template_span(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_template_parameter(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_decorator(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_scalar_constructor(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_intrinsic(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_string(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_number(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
    fn on_boolean(&mut self, _type_id: TypeId, _checker: &Checker) -> ListenerFlow {
        ListenerFlow::Continue
    }
}

/// Navigation context for tracking visited types and options
struct NavigationContext<'a, L: SemanticNodeListener> {
    listener: &'a mut L,
    options: NavigationOptions,
    visited: HashSet<TypeId>,
}

impl<'a, L: SemanticNodeListener> NavigationContext<'a, L> {
    fn should_navigate_templatable_type(&self, type_id: TypeId, checker: &Checker) -> bool {
        let is_finished = checker
            .get_type(type_id)
            .map(|t| match t {
                Type::Model(m) => m.is_finished,
                Type::Interface(i) => i.is_finished,
                Type::Union(u) => u.is_finished,
                Type::Operation(o) => o.is_finished,
                _ => true,
            })
            .unwrap_or(true);

        if self.options.include_template_declaration {
            // TS: isFinished || isTemplateDeclaration(type)
            // isTemplateDeclaration = template_node.is_some() && template_mapper.is_none()
            let is_template_decl = checker
                .get_type(type_id)
                .map(|t| match t {
                    Type::Model(m) => m.template_node.is_some() && m.template_mapper.is_none(),
                    Type::Interface(i) => i.template_node.is_some() && i.template_mapper.is_none(),
                    Type::Union(u) => u.template_node.is_some() && u.template_mapper.is_none(),
                    Type::Scalar(s) => s.template_node.is_some() && s.template_mapper.is_none(),
                    Type::Operation(o) => o.template_node.is_some() && o.template_mapper.is_none(),
                    _ => false,
                })
                .unwrap_or(false);
            is_finished || is_template_decl
        } else {
            is_finished
        }
    }
}

/// Navigate all types in the program
/// Ported from TS navigateProgram()
pub fn navigate_program<L: SemanticNodeListener>(
    checker: &Checker,
    listener: &mut L,
    options: NavigationOptions,
) {
    let mut context = NavigationContext {
        listener,
        options,
        visited: HashSet::new(),
    };

    context.listener.on_root(checker);

    if let Some(global_ns_id) = checker.get_global_namespace_type() {
        navigate_namespace_type(checker, global_ns_id, &mut context);
    }
}

/// Navigate the given type and all the types used in it
/// Ported from TS navigateType()
pub fn navigate_type<L: SemanticNodeListener>(
    checker: &Checker,
    type_id: TypeId,
    listener: &mut L,
    options: NavigationOptions,
) {
    let mut context = NavigationContext {
        listener,
        options,
        visited: HashSet::new(),
    };
    navigate_type_internal(checker, type_id, &mut context);
}

/// Navigate types in a namespace (scoped to not leave the namespace)
/// Ported from TS navigateTypesInNamespace()
pub fn navigate_types_in_namespace<L: SemanticNodeListener>(
    checker: &Checker,
    namespace_id: TypeId,
    listener: &mut L,
    options: NavigationOptions,
) {
    // For now, just navigate the namespace type
    // TS version wraps listeners to scope navigation; we simplify
    let mut context = NavigationContext {
        listener,
        options,
        visited: HashSet::new(),
    };
    navigate_namespace_type(checker, namespace_id, &mut context);
}

fn check_visited(visited: &mut HashSet<TypeId>, type_id: TypeId) -> bool {
    if visited.contains(&type_id) {
        return true;
    }
    visited.insert(type_id);
    false
}

fn navigate_namespace_type<L: SemanticNodeListener>(
    checker: &Checker,
    namespace_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, namespace_id) {
        return;
    }

    if context.listener.on_namespace(namespace_id, checker) == ListenerFlow::NoRecursion {
        return;
    }

    // Navigate namespace contents
    if let Some(Type::Namespace(ns)) = checker.get_type(namespace_id) {
        // Models
        for name in &ns.model_names {
            if let Some(&model_id) = ns.models.get(name) {
                navigate_model_type(checker, model_id, context);
            }
        }

        // Scalars
        for name in &ns.scalar_names {
            if let Some(&scalar_id) = ns.scalars.get(name) {
                navigate_scalar_type(checker, scalar_id, context);
            }
        }

        // Operations
        for name in &ns.operation_names {
            if let Some(&op_id) = ns.operations.get(name) {
                navigate_operation_type(checker, op_id, context);
            }
        }

        // Sub-namespaces
        for name in &ns.namespace_names {
            if let Some(&sub_ns_id) = ns.namespaces.get(name) {
                // Skip TypeSpec.Prototypes (as in TS)
                if ns.name == "TypeSpec"
                    && let Some(Type::Namespace(sub_ns)) = checker.get_type(sub_ns_id)
                    && sub_ns.name == "Prototypes"
                {
                    continue;
                }
                navigate_namespace_type(checker, sub_ns_id, context);
            }
        }

        // Unions
        for name in &ns.union_names {
            if let Some(&union_id) = ns.unions.get(name) {
                navigate_union_type(checker, union_id, context);
            }
        }

        // Interfaces
        for name in &ns.interface_names {
            if let Some(&iface_id) = ns.interfaces.get(name) {
                navigate_interface_type(checker, iface_id, context);
            }
        }

        // Enums
        for name in &ns.enum_names {
            if let Some(&enum_id) = ns.enums.get(name) {
                navigate_enum_type(checker, enum_id, context);
            }
        }

        // Decorator declarations (NamespaceType doesn't have decorator_declarations field yet)
        // Skip for now - will be added when decorator declarations are fully implemented
    }

    context.listener.on_exit_namespace(namespace_id, checker);
}

fn navigate_model_type<L: SemanticNodeListener>(
    checker: &Checker,
    model_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, model_id) {
        return;
    }
    if !context.should_navigate_templatable_type(model_id, checker) {
        return;
    }
    if context.listener.on_model(model_id, checker) == ListenerFlow::NoRecursion {
        return;
    }

    if let Some(Type::Model(model)) = checker.get_type(model_id) {
        // Properties
        for name in &model.property_names {
            if let Some(&prop_id) = model.properties.get(name) {
                navigate_model_type_property(checker, prop_id, context);
            }
        }

        // Base model
        if let Some(base_model_id) = model.base_model {
            navigate_model_type(checker, base_model_id, context);
        }

        // Indexer (stored as (key_type_id, value_type_id) tuple)
        if let Some(indexer) = &model.indexer {
            navigate_type_internal(checker, indexer.1, context);
        }

        // Derived types
        if context.options.visit_derived_types {
            for &derived_id in &model.derived_models {
                navigate_model_type(checker, derived_id, context);
            }
        }
    }

    context.listener.on_exit_model(model_id, checker);
}

fn navigate_model_type_property<L: SemanticNodeListener>(
    checker: &Checker,
    prop_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, prop_id) {
        return;
    }
    if context.listener.on_model_property(prop_id, checker) == ListenerFlow::NoRecursion {
        return;
    }

    if let Some(Type::ModelProperty(prop)) = checker.get_type(prop_id) {
        navigate_type_internal(checker, prop.r#type, context);
    }

    context.listener.on_exit_model_property(prop_id, checker);
}

fn navigate_scalar_type<L: SemanticNodeListener>(
    checker: &Checker,
    scalar_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, scalar_id) {
        return;
    }
    if context.listener.on_scalar(scalar_id, checker) == ListenerFlow::NoRecursion {
        return;
    }

    if let Some(Type::Scalar(scalar)) = checker.get_type(scalar_id) {
        // Base scalar
        if let Some(base_scalar_id) = scalar.base_scalar {
            navigate_scalar_type(checker, base_scalar_id, context);
        }

        // Constructors
        for &ctor_id in &scalar.constructors {
            navigate_scalar_constructor(checker, ctor_id, context);
        }
    }

    context.listener.on_exit_scalar(scalar_id, checker);
}

fn navigate_interface_type<L: SemanticNodeListener>(
    checker: &Checker,
    iface_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, iface_id) {
        return;
    }
    if !context.should_navigate_templatable_type(iface_id, checker) {
        return;
    }

    if context.listener.on_interface(iface_id, checker) == ListenerFlow::NoRecursion {
        context.listener.on_exit_interface(iface_id, checker);
        return;
    }

    if let Some(Type::Interface(iface)) = checker.get_type(iface_id) {
        for name in &iface.operation_names {
            if let Some(&op_id) = iface.operations.get(name) {
                navigate_operation_type(checker, op_id, context);
            }
        }
    }

    context.listener.on_exit_interface(iface_id, checker);
}

fn navigate_enum_type<L: SemanticNodeListener>(
    checker: &Checker,
    enum_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, enum_id) {
        return;
    }

    if context.listener.on_enum(enum_id, checker) == ListenerFlow::NoRecursion {
        context.listener.on_exit_enum(enum_id, checker);
        return;
    }

    if let Some(Type::Enum(enum_type)) = checker.get_type(enum_id) {
        for name in &enum_type.member_names {
            if let Some(&member_id) = enum_type.members.get(name) {
                navigate_type_internal(checker, member_id, context);
            }
        }
    }

    context.listener.on_exit_enum(enum_id, checker);
}

fn navigate_union_type<L: SemanticNodeListener>(
    checker: &Checker,
    union_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, union_id) {
        return;
    }
    if !context.should_navigate_templatable_type(union_id, checker) {
        return;
    }
    if context.listener.on_union(union_id, checker) == ListenerFlow::NoRecursion {
        return;
    }

    if let Some(Type::Union(union)) = checker.get_type(union_id) {
        for name in &union.variant_names {
            if let Some(&variant_id) = union.variants.get(name) {
                navigate_union_type_variant(checker, variant_id, context);
            }
        }
    }

    context.listener.on_exit_union(union_id, checker);
}

fn navigate_union_type_variant<L: SemanticNodeListener>(
    checker: &Checker,
    variant_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, variant_id) {
        return;
    }
    if context.listener.on_union_variant(variant_id, checker) == ListenerFlow::NoRecursion {
        return;
    }

    if let Some(Type::UnionVariant(variant)) = checker.get_type(variant_id) {
        navigate_type_internal(checker, variant.r#type, context);
    }

    context.listener.on_exit_union_variant(variant_id, checker);
}

fn navigate_operation_type<L: SemanticNodeListener>(
    checker: &Checker,
    op_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, op_id) {
        return;
    }
    if !context.should_navigate_templatable_type(op_id, checker) {
        return;
    }
    if context.listener.on_operation(op_id, checker) == ListenerFlow::NoRecursion {
        return;
    }

    if let Some(Type::Operation(op)) = checker.get_type(op_id) {
        // Parameters
        if let Some(params_model_id) = op.parameters
            && let Some(Type::Model(params_model)) = checker.get_type(params_model_id)
        {
            for name in &params_model.property_names {
                if let Some(&prop_id) = params_model.properties.get(name) {
                    navigate_type_internal(checker, prop_id, context);
                }
            }
        }

        // Return type
        if let Some(return_type_id) = op.return_type {
            navigate_type_internal(checker, return_type_id, context);
        }
    }

    context.listener.on_exit_operation(op_id, checker);
}

fn navigate_decorator_declaration<L: SemanticNodeListener>(
    checker: &Checker,
    dec_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, dec_id) {
        return;
    }
    context.listener.on_decorator(dec_id, checker);
}

fn navigate_scalar_constructor<L: SemanticNodeListener>(
    checker: &Checker,
    ctor_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    if check_visited(&mut context.visited, ctor_id) {
        return;
    }
    context.listener.on_scalar_constructor(ctor_id, checker);
}

fn navigate_type_internal<L: SemanticNodeListener>(
    checker: &Checker,
    type_id: TypeId,
    context: &mut NavigationContext<L>,
) {
    let Some(ty) = checker.get_type(type_id) else {
        return;
    };

    match ty {
        Type::Model(_) => navigate_model_type(checker, type_id, context),
        Type::Scalar(_) => navigate_scalar_type(checker, type_id, context),
        Type::ModelProperty(_) => navigate_model_type_property(checker, type_id, context),
        Type::Namespace(_) => navigate_namespace_type(checker, type_id, context),
        Type::Interface(_) => navigate_interface_type(checker, type_id, context),
        Type::Enum(_) => navigate_enum_type(checker, type_id, context),
        Type::Operation(_) => navigate_operation_type(checker, type_id, context),
        Type::Union(_) => navigate_union_type(checker, type_id, context),
        Type::UnionVariant(_) => navigate_union_type_variant(checker, type_id, context),
        Type::Tuple(_) => {
            if !check_visited(&mut context.visited, type_id) {
                if context.listener.on_tuple(type_id, checker) != ListenerFlow::NoRecursion
                    && let Some(Type::Tuple(tuple)) = checker.get_type(type_id)
                {
                    for &val_id in tuple.values.iter() {
                        navigate_type_internal(checker, val_id, context);
                    }
                }
                context.listener.on_exit_tuple(type_id, checker);
            }
        }
        Type::StringTemplate(_) => {
            if !check_visited(&mut context.visited, type_id)
                && context.listener.on_string_template(type_id, checker)
                    != ListenerFlow::NoRecursion
                && let Some(Type::StringTemplate(st)) = checker.get_type(type_id)
            {
                for &span_id in &st.spans {
                    navigate_type_internal(checker, span_id, context);
                }
            }
        }
        Type::StringTemplateSpan(_) => {
            if !check_visited(&mut context.visited, type_id)
                && context.listener.on_string_template_span(type_id, checker)
                    != ListenerFlow::NoRecursion
                && let Some(Type::StringTemplateSpan(span)) = checker.get_type(type_id)
                && let Some(ty) = span.r#type
            {
                navigate_type_internal(checker, ty, context);
            }
        }
        Type::TemplateParameter(_) => {
            if !check_visited(&mut context.visited, type_id) {
                context.listener.on_template_parameter(type_id, checker);
            }
        }
        Type::Decorator(_) => {
            navigate_decorator_declaration(checker, type_id, context);
        }
        Type::ScalarConstructor(_) => {
            navigate_scalar_constructor(checker, type_id, context);
        }
        // Leaf types - just emit events
        Type::Intrinsic(_) => {
            if !check_visited(&mut context.visited, type_id) {
                context.listener.on_intrinsic(type_id, checker);
            }
        }
        Type::String(_) => {
            if !check_visited(&mut context.visited, type_id) {
                context.listener.on_string(type_id, checker);
            }
        }
        Type::Number(_) => {
            if !check_visited(&mut context.visited, type_id) {
                context.listener.on_number(type_id, checker);
            }
        }
        Type::Boolean(_) => {
            if !check_visited(&mut context.visited, type_id) {
                context.listener.on_boolean(type_id, checker);
            }
        }
        Type::EnumMember(_) => {
            if !check_visited(&mut context.visited, type_id) {
                context.listener.on_enum_member(type_id, checker);
            }
        }
        // Leaf types with no child navigation
        Type::FunctionType(_) | Type::FunctionParameter(_) => {}
    }
}

/// Get a property from a model, looking up the base type chain
/// Ported from TS getProperty()
pub fn get_property(checker: &Checker, model_id: TypeId, property_name: &str) -> Option<TypeId> {
    let mut current_id = model_id;
    loop {
        if let Some(Type::Model(model)) = checker.get_type(current_id) {
            if let Some(&prop_id) = model.properties.get(property_name) {
                return Some(prop_id);
            }
            match model.base_model {
                Some(base_id) => current_id = base_id,
                None => return None,
            }
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::test_utils::check;
    use crate::checker::types::Type;

    /// Detailed collector matching TS createCollector pattern
    /// Records TypeIds for each type kind with entry/exit tracking
    struct DetailCollector {
        models: Vec<TypeId>,
        exit_models: Vec<TypeId>,
        model_properties: Vec<TypeId>,
        exit_model_properties: Vec<TypeId>,
        operations: Vec<TypeId>,
        exit_operations: Vec<TypeId>,
        namespaces: Vec<TypeId>,
        exit_namespaces: Vec<TypeId>,
        enums: Vec<TypeId>,
        exit_enums: Vec<TypeId>,
        interfaces: Vec<TypeId>,
        exit_interfaces: Vec<TypeId>,
        unions: Vec<TypeId>,
        exit_unions: Vec<TypeId>,
        union_variants: Vec<TypeId>,
        exit_union_variants: Vec<TypeId>,
        tuples: Vec<TypeId>,
        exit_tuples: Vec<TypeId>,
        scalars: Vec<TypeId>,
        exit_scalars: Vec<TypeId>,
    }

    impl DetailCollector {
        fn new() -> Self {
            Self {
                models: Vec::new(),
                exit_models: Vec::new(),
                model_properties: Vec::new(),
                exit_model_properties: Vec::new(),
                operations: Vec::new(),
                exit_operations: Vec::new(),
                namespaces: Vec::new(),
                exit_namespaces: Vec::new(),
                enums: Vec::new(),
                exit_enums: Vec::new(),
                interfaces: Vec::new(),
                exit_interfaces: Vec::new(),
                unions: Vec::new(),
                exit_unions: Vec::new(),
                union_variants: Vec::new(),
                exit_union_variants: Vec::new(),
                tuples: Vec::new(),
                exit_tuples: Vec::new(),
                scalars: Vec::new(),
                exit_scalars: Vec::new(),
            }
        }
    }

    impl SemanticNodeListener for DetailCollector {
        fn on_model(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.models.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_model(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_models.push(type_id);
        }
        fn on_model_property(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.model_properties.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_model_property(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_model_properties.push(type_id);
        }
        fn on_operation(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.operations.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_operation(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_operations.push(type_id);
        }
        fn on_namespace(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.namespaces.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_namespace(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_namespaces.push(type_id);
        }
        fn on_enum(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.enums.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_enum(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_enums.push(type_id);
        }
        fn on_interface(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.interfaces.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_interface(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_interfaces.push(type_id);
        }
        fn on_union(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.unions.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_union(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_unions.push(type_id);
        }
        fn on_union_variant(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.union_variants.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_union_variant(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_union_variants.push(type_id);
        }
        fn on_tuple(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.tuples.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_tuple(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_tuples.push(type_id);
        }
        fn on_scalar(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.scalars.push(type_id);
            ListenerFlow::Continue
        }
        fn on_exit_scalar(&mut self, type_id: TypeId, _checker: &Checker) {
            self.exit_scalars.push(type_id);
        }
    }

    /// Collector that allows a custom model callback for NoRecursion testing
    struct NoRecursionCollector {
        models: Vec<TypeId>,
        model_properties: Vec<TypeId>,
        no_recursion_names: Vec<String>,
    }

    impl NoRecursionCollector {
        fn new(no_recursion_names: Vec<String>) -> Self {
            Self {
                models: Vec::new(),
                model_properties: Vec::new(),
                no_recursion_names,
            }
        }
    }

    impl SemanticNodeListener for NoRecursionCollector {
        fn on_model(&mut self, type_id: TypeId, checker: &Checker) -> ListenerFlow {
            self.models.push(type_id);
            let name = checker
                .get_type(type_id)
                .and_then(|t| t.name())
                .unwrap_or("")
                .to_string();
            if self.no_recursion_names.contains(&name) {
                ListenerFlow::NoRecursion
            } else {
                ListenerFlow::Continue
            }
        }
        fn on_model_property(&mut self, type_id: TypeId, _checker: &Checker) -> ListenerFlow {
            self.model_properties.push(type_id);
            ListenerFlow::Continue
        }
    }

    /// Helper: get the name of a type by TypeId
    fn type_name(checker: &Checker, type_id: TypeId) -> String {
        checker
            .get_type(type_id)
            .and_then(|t| t.name())
            .unwrap_or("")
            .to_string()
    }

    // =========================================================================
    // Tests ported from TS semantic-walker.test.ts
    // =========================================================================

    #[test]
    fn test_finds_models() {
        let checker = check(
            "
            model Foo {
                nested: {
                    inline: true
                }
            }

            model Bar {
                name: Foo;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let model_names: Vec<String> = collector
            .models
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        // HashMap iteration order is non-deterministic, so check by content
        assert_eq!(
            model_names.len(),
            3,
            "Should find 3 models: Foo, inline, Bar (got {:?})",
            model_names
        );
        assert!(model_names.contains(&"Foo".to_string()), "Should find Foo");
        assert!(model_names.contains(&"Bar".to_string()), "Should find Bar");
        assert!(
            model_names.contains(&"".to_string()),
            "Should find inline model (empty name)"
        );

        // Verify Foo contains inline model: Foo comes before its inline child in DFS
        let foo_idx = model_names.iter().position(|n| n == "Foo").unwrap();
        let inline_idx = model_names.iter().position(|n| n.is_empty()).unwrap();
        assert!(foo_idx < inline_idx, "Foo should come before inline model");
    }

    #[test]
    fn test_finds_exit_models() {
        let checker = check(
            "
            model Foo {
                nested: {
                    inline: true
                }
            }

            model Bar {
                name: Foo;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let exit_model_names: Vec<String> = collector
            .exit_models
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        assert_eq!(
            exit_model_names.len(),
            3,
            "Should find 3 exit models (got {:?})",
            exit_model_names
        );
        // Exit order: DFS post-order - inline before Foo
        let inline_idx = exit_model_names.iter().position(|n| n.is_empty()).unwrap();
        let foo_idx = exit_model_names.iter().position(|n| n == "Foo").unwrap();
        assert!(inline_idx < foo_idx, "Inline model should exit before Foo");
    }

    #[test]
    fn test_finds_operations() {
        let checker = check(
            "
            op foo(): true;

            namespace Nested {
                op bar(): true;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.operations.len(), 2);
        assert_eq!(type_name(&checker, collector.operations[0]), "foo");
        assert_eq!(type_name(&checker, collector.operations[1]), "bar");
    }

    #[test]
    fn test_finds_exit_operations() {
        let checker = check(
            "
            op foo(): true;

            namespace Nested {
                op bar(): true;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.exit_operations.len(), 2);
        assert_eq!(type_name(&checker, collector.exit_operations[0]), "foo");
        assert_eq!(type_name(&checker, collector.exit_operations[1]), "bar");
    }

    #[test]
    fn test_finds_namespaces() {
        // TS test uses: namespace Global.My; namespace Simple {} namespace Parent { namespace Child {} }
        // With nostdlib, there are no TypeSpec library namespaces
        // Without nostdlib, our checker creates a "TypeSpec" namespace for std types
        let checker = check(
            "
            namespace Simple {
            }
            namespace Parent {
                namespace Child {
                }
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        // Should find at least: global (""), Simple, Parent, Child
        // May also include TypeSpec namespace from std types
        let ns_names: Vec<String> = collector
            .namespaces
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        // Check that the key namespaces are present in the expected order
        assert!(
            ns_names.iter().any(|n| n.is_empty()),
            "Should have global namespace"
        );
        assert!(ns_names.iter().any(|n| n == "Simple"), "Should find Simple");
        assert!(ns_names.iter().any(|n| n == "Parent"), "Should find Parent");
        assert!(ns_names.iter().any(|n| n == "Child"), "Should find Child");

        // Verify ordering: Parent comes before Child
        let parent_idx = ns_names.iter().position(|n| n == "Parent").unwrap();
        let child_idx = ns_names.iter().position(|n| n == "Child").unwrap();
        assert!(parent_idx < child_idx, "Parent should come before Child");
    }

    #[test]
    fn test_finds_exit_namespaces() {
        let checker = check(
            "
            namespace Simple {
            }
            namespace Parent {
                namespace Child {
                }
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let exit_ns_names: Vec<String> = collector
            .exit_namespaces
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        // Exit order: DFS post-order (children before parents)
        // Child before Parent
        let parent_idx = exit_ns_names.iter().position(|n| n == "Parent").unwrap();
        let child_idx = exit_ns_names.iter().position(|n| n == "Child").unwrap();
        assert!(child_idx < parent_idx, "Child should exit before Parent");
    }

    #[test]
    fn test_finds_model_properties() {
        let checker = check(
            "
            model Foo {
                nested: {
                    inline: true
                }
            }

            model Bar {
                name: Foo;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let prop_names: Vec<String> = collector
            .model_properties
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        assert_eq!(
            prop_names.len(),
            3,
            "Should find 3 properties (got {:?})",
            prop_names
        );
        assert!(
            prop_names.contains(&"nested".to_string()),
            "Should find nested"
        );
        assert!(
            prop_names.contains(&"inline".to_string()),
            "Should find inline"
        );
        assert!(prop_names.contains(&"name".to_string()), "Should find name");
    }

    #[test]
    fn test_finds_exit_model_properties() {
        let checker = check(
            "
            model Foo {
                nested: {
                    inline: true
                }
            }

            model Bar {
                name: Foo;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let exit_prop_names: Vec<String> = collector
            .exit_model_properties
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        assert_eq!(
            exit_prop_names.len(),
            3,
            "Should find 3 exit properties (got {:?})",
            exit_prop_names
        );
        assert!(
            exit_prop_names.contains(&"inline".to_string()),
            "Should find inline"
        );
        assert!(
            exit_prop_names.contains(&"nested".to_string()),
            "Should find nested"
        );
        assert!(
            exit_prop_names.contains(&"name".to_string()),
            "Should find name"
        );
    }

    #[test]
    fn test_finds_enums() {
        let checker = check(
            "
            enum Direction {
                North: \"north\",
                East: \"east\",
                South: \"south\",
                West: \"west\",
            }

            enum Metric {
                One: 1,
                Ten: 10,
                Hundred: 100,
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let enum_names: Vec<String> = collector
            .enums
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        assert_eq!(
            enum_names.len(),
            2,
            "Should find 2 enums (got {:?})",
            enum_names
        );
        assert!(
            enum_names.contains(&"Direction".to_string()),
            "Should find Direction"
        );
        assert!(
            enum_names.contains(&"Metric".to_string()),
            "Should find Metric"
        );
    }

    #[test]
    fn test_finds_exit_enums() {
        let checker = check(
            "
            enum Direction {
                North: \"north\",
                East: \"east\",
                South: \"south\",
                West: \"west\",
            }

            enum Metric {
                One: 1,
                Ten: 10,
                Hundred: 100,
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let exit_enum_names: Vec<String> = collector
            .exit_enums
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        assert_eq!(
            exit_enum_names.len(),
            2,
            "Should find 2 exit enums (got {:?})",
            exit_enum_names
        );
        assert!(
            exit_enum_names.contains(&"Direction".to_string()),
            "Should find Direction"
        );
        assert!(
            exit_enum_names.contains(&"Metric".to_string()),
            "Should find Metric"
        );
    }

    #[test]
    fn test_finds_tuples_with_model() {
        let checker = check(
            "
            model Foo {
                bar: [Direction, Color]
            }

            enum Direction {
                North,
                East,
                South,
                West,
            }

            model Color {
                value: string;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.tuples.len(), 1);
        assert_eq!(type_name(&checker, collector.enums[0]), "Direction");
        // models[0] = Foo, models[1] = Color (inline tuple model not named)
        assert!(
            collector
                .models
                .iter()
                .any(|&id| type_name(&checker, id) == "Color"),
            "Should find Color model"
        );
    }

    #[test]
    fn test_finds_exit_tuples_with_model() {
        let checker = check(
            "
            model Foo {
                bar: [Direction, Color]
            }

            enum Direction {
                North,
                East,
                South,
                West,
            }

            model Color {
                value: string;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.exit_tuples.len(), 1);
        assert_eq!(type_name(&checker, collector.exit_enums[0]), "Direction");
        assert!(
            collector
                .exit_models
                .iter()
                .any(|&id| type_name(&checker, id) == "Color"),
            "Should find Color exit model"
        );
    }

    #[test]
    fn test_finds_unions() {
        let checker = check(
            "
            union A {
                x: true;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.unions.len(), 1);
        assert_eq!(type_name(&checker, collector.unions[0]), "A");
        assert_eq!(collector.union_variants.len(), 1);
        assert_eq!(type_name(&checker, collector.union_variants[0]), "x");
    }

    #[test]
    fn test_finds_tuples() {
        let checker = check(
            "
            model ContainsTuple {
                tuple: [string];
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.tuples.len(), 1);
        // Check tuple has 1 value
        if let Some(Type::Tuple(tuple)) = checker.get_type(collector.tuples[0]) {
            assert_eq!(tuple.values.len(), 1);
        } else {
            panic!("Expected Tuple type");
        }
    }

    #[test]
    fn test_finds_exit_tuples() {
        let checker = check(
            "
            model ContainsTuple {
                tuple: [string];
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.exit_tuples.len(), 1);
        if let Some(Type::Tuple(tuple)) = checker.get_type(collector.exit_tuples[0]) {
            assert_eq!(tuple.values.len(), 1);
        } else {
            panic!("Expected Tuple type");
        }
    }

    #[test]
    fn test_finds_exit_unions() {
        let checker = check(
            "
            union A {
                x: true;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.exit_unions.len(), 1);
        assert_eq!(type_name(&checker, collector.exit_unions[0]), "A");
        assert_eq!(collector.exit_union_variants.len(), 1);
        assert_eq!(type_name(&checker, collector.exit_union_variants[0]), "x");
    }

    #[test]
    fn test_finds_interfaces() {
        let checker = check(
            "
            model B { };
            interface A {
                a(): true;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.interfaces.len(), 1, "finds interfaces");
        assert_eq!(type_name(&checker, collector.interfaces[0]), "A");
        assert_eq!(collector.operations.len(), 1, "finds operations");
        assert_eq!(type_name(&checker, collector.operations[0]), "a");
    }

    #[test]
    fn test_finds_exit_interfaces() {
        let checker = check(
            "
            model B { };
            interface A {
                a(): true;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.exit_interfaces.len(), 1, "finds exit interfaces");
        assert_eq!(type_name(&checker, collector.exit_interfaces[0]), "A");
        assert_eq!(collector.exit_operations.len(), 1, "finds exit operations");
        assert_eq!(type_name(&checker, collector.exit_operations[0]), "a");
    }

    #[test]
    fn test_finds_owned_or_inherited_properties() {
        let checker = check(
            "
            model Pet {
                name: true;
            }

            model Cat extends Pet {
                meow: true;
            }
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let model_names: Vec<String> = collector
            .models
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        assert_eq!(
            model_names.len(),
            2,
            "Should find 2 models: Pet and Cat (got {:?})",
            model_names
        );
        assert!(model_names.contains(&"Pet".to_string()), "Should find Pet");
        assert!(model_names.contains(&"Cat".to_string()), "Should find Cat");

        let cat_id = checker.declared_types.get("Cat").copied().unwrap();
        assert!(get_property(&checker, cat_id, "meow").is_some());
        assert!(get_property(&checker, cat_id, "name").is_some());
        assert!(get_property(&checker, cat_id, "bark").is_none());
    }

    #[test]
    fn test_no_recursion_stops_navigation() {
        let checker = check(
            "
            model A {
                shouldNotNavigate: true;
            }

            model B {
                shouldNavigate: true;
            }
        ",
        );
        let mut collector = NoRecursionCollector::new(vec!["A".to_string()]);
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        assert_eq!(collector.model_properties.len(), 1);
        assert_eq!(
            type_name(&checker, collector.model_properties[0]),
            "shouldNavigate"
        );
    }

    #[test]
    fn test_derived_models_with_option() {
        let checker = check(
            "
            model Bird {
                kind: string;
                wingspan: int32;
            }

            model SeaGull extends Bird {
                kind: \"seagull\";
            }

            model Sparrow extends Bird {
                kind: \"sparrow\";
            }

            model Goose extends Bird {
                kind: \"goose\";
            }

            model Eagle extends Bird {
                kind: \"eagle\";
            }
        ",
        );
        let bird_id = checker.declared_types.get("Bird").copied().unwrap();
        let mut collector = DetailCollector::new();
        navigate_type(
            &checker,
            bird_id,
            &mut collector,
            NavigationOptions {
                visit_derived_types: true,
                ..NavigationOptions::default()
            },
        );

        let visited_names: Vec<String> = collector
            .models
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();
        for expected in &["Bird", "SeaGull", "Sparrow", "Goose", "Eagle"] {
            assert!(
                visited_names.contains(&expected.to_string()),
                "Should find model {} in visited: {:?}",
                expected,
                visited_names
            );
        }
    }

    #[test]
    fn test_no_derived_models_without_option() {
        let checker = check(
            "
            model Bird {
                kind: string;
                wingspan: int32;
            }

            model SeaGull extends Bird {
                kind: \"seagull\";
            }

            model Sparrow extends Bird {
                kind: \"sparrow\";
            }
        ",
        );
        let bird_id = checker.declared_types.get("Bird").copied().unwrap();
        let mut collector = DetailCollector::new();
        navigate_type(
            &checker,
            bird_id,
            &mut collector,
            NavigationOptions {
                visit_derived_types: false,
                ..NavigationOptions::default()
            },
        );

        assert_eq!(
            collector.models.len(),
            1,
            "Only Bird itself, no derived models"
        );
    }

    #[test]
    fn test_find_in_namespace_models_only() {
        let checker = check(
            "
            namespace TargetNs {
                model A {}
            }

            model B {}

            namespace Other {
                model C {}
            }
        ",
        );
        let global_ns_id = checker.get_global_namespace_type().unwrap();
        let target_ns_id = if let Some(Type::Namespace(ns)) = checker.get_type(global_ns_id) {
            *ns.namespaces.get("TargetNs").unwrap()
        } else {
            panic!("Expected global namespace");
        };

        let mut collector = DetailCollector::new();
        navigate_types_in_namespace(
            &checker,
            target_ns_id,
            &mut collector,
            NavigationOptions::default(),
        );

        assert_eq!(collector.models.len(), 1);
        assert_eq!(type_name(&checker, collector.models[0]), "A");
    }

    #[test]
    fn test_find_in_namespace_sub_namespace() {
        let checker = check(
            "
            namespace TargetNs {
                model A {}

                namespace Sub {
                    model B {}
                }
            }
        ",
        );
        let global_ns_id = checker.get_global_namespace_type().unwrap();
        let target_ns_id = if let Some(Type::Namespace(ns)) = checker.get_type(global_ns_id) {
            *ns.namespaces.get("TargetNs").unwrap()
        } else {
            panic!("Expected global namespace");
        };

        let mut collector = DetailCollector::new();
        navigate_types_in_namespace(
            &checker,
            target_ns_id,
            &mut collector,
            NavigationOptions::default(),
        );

        assert_eq!(collector.models.len(), 2);
        assert_eq!(type_name(&checker, collector.models[0]), "A");
        assert_eq!(type_name(&checker, collector.models[1]), "B");
    }

    #[test]
    fn test_template_declarations_not_included_by_default() {
        let checker = check(
            "
            model Foo<T> {}
            model Bar {}
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(&checker, &mut collector, NavigationOptions::default());

        let model_names: Vec<String> = collector
            .models
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        // By default, template declarations (is_finished = false) are excluded
        // Only Bar should appear (Foo is a template declaration, so is_finished=false)
        // Note: std types (Array, Record) may also appear if they exist
        assert!(
            !model_names.contains(&"Foo".to_string()),
            "Template declaration Foo should NOT appear: {:?}",
            model_names
        );
        assert!(
            model_names.contains(&"Bar".to_string()),
            "Bar should appear: {:?}",
            model_names
        );
    }

    #[test]
    fn test_template_declarations_included_with_option() {
        let checker = check(
            "
            model Foo<T> {}
            model Bar {}
        ",
        );
        let mut collector = DetailCollector::new();
        navigate_program(
            &checker,
            &mut collector,
            NavigationOptions {
                include_template_declaration: true,
                ..NavigationOptions::default()
            },
        );

        // With include_template_declaration, Foo should also be included
        // Plus Array and Record from std types
        let model_names: Vec<String> = collector
            .models
            .iter()
            .map(|&id| type_name(&checker, id))
            .collect();

        assert!(
            model_names.contains(&"Foo".to_string()),
            "Should include Foo template declaration: {:?}",
            model_names
        );
        assert!(
            model_names.contains(&"Bar".to_string()),
            "Should include Bar: {:?}",
            model_names
        );
    }

    // NOTE: The "include functions" test requires extern fn support which needs
    // JS decorator execution, so it's skipped (same as TS NavigatorTester setup).
    // The "by default only include the template instantiations" test requires
    // full template instantiation (op return type resolves to Bar<Qux>), which
    // depends on deeper checker support - skipped.
}
