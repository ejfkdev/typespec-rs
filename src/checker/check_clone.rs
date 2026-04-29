//! Clone type operations
//!
//! Ported from TypeSpec compiler clone_type methods

use super::*;

impl Checker {
    /// Clone a type, creating a new TypeId with deep cloning of child types.
    /// Recursively clones ModelProperty, EnumMember, UnionVariant, Operation
    /// and re-parents them to the new clone.
    /// Ported from TS compiler/src/core/checker.ts cloneType
    pub fn clone_type(&mut self, type_id: TypeId) -> TypeId {
        // Safety: prevent infinite recursion with a depth limit
        self.check_depth += 1;
        if self.check_depth > 200 {
            self.check_depth -= 1;
            return self.error_type;
        }
        let result = self.clone_type_impl(type_id);
        self.check_depth -= 1;
        result
    }

    pub(crate) fn clone_type_impl(&mut self, type_id: TypeId) -> TypeId {
        let t = self.get_type(type_id).cloned();
        match t {
            Some(t) => {
                match t {
                    Type::Model(mut m) => {
                        // Deep clone properties and re-parent
                        let (new_properties, new_property_names, orig_prop_ids) =
                            self.clone_named_map(&m.property_names, &m.properties);
                        m.properties = new_properties;
                        m.property_names = new_property_names;
                        m.derived_models = Vec::new();
                        let new_id = self.create_type(Type::Model(m));
                        // Re-parent all cloned properties to the new model
                        let cloned_prop_ids: Vec<(String, TypeId)> = self
                            .get_type(new_id)
                            .map(|t| {
                                if let Type::Model(new_m) = t {
                                    new_m
                                        .property_names
                                        .iter()
                                        .filter_map(|n| {
                                            new_m.properties.get(n).map(|&id| (n.clone(), id))
                                        })
                                        .collect()
                                } else {
                                    vec![]
                                }
                            })
                            .unwrap_or_default();
                        for (i, (_name, cloned_prop_id)) in cloned_prop_ids.iter().enumerate() {
                            if let Some(prop) = self.get_type_mut(*cloned_prop_id)
                                && let Type::ModelProperty(p) = prop
                            {
                                p.model = Some(new_id);
                                if i < orig_prop_ids.len() {
                                    p.source_property = Some(orig_prop_ids[i]);
                                }
                            }
                        }
                        new_id
                    }
                    Type::ModelProperty(p) => self.create_type(Type::ModelProperty(p)),
                    Type::Enum(mut e) => {
                        let (new_members, new_member_names, _) =
                            self.clone_named_map(&e.member_names, &e.members);
                        e.members = new_members;
                        e.member_names = new_member_names;
                        let new_id = self.create_type(Type::Enum(e));
                        self.reparent_children(
                            new_id,
                            |t| {
                                if let Type::Enum(en) = t {
                                    en.member_names
                                        .iter()
                                        .filter_map(|n| en.members.get(n).copied())
                                        .collect()
                                } else {
                                    vec![]
                                }
                            },
                            |child, parent| {
                                if let Type::EnumMember(em) = child {
                                    em.r#enum = Some(parent);
                                }
                            },
                        );
                        new_id
                    }
                    Type::EnumMember(m) => self.create_type(Type::EnumMember(m)),
                    Type::Union(mut u) => {
                        let (new_variants, new_variant_names, _) =
                            self.clone_named_map(&u.variant_names, &u.variants);
                        u.variants = new_variants;
                        u.variant_names = new_variant_names;
                        let new_id = self.create_type(Type::Union(u));
                        self.reparent_children(
                            new_id,
                            |t| {
                                if let Type::Union(un) = t {
                                    un.variant_names
                                        .iter()
                                        .filter_map(|n| un.variants.get(n).copied())
                                        .collect()
                                } else {
                                    vec![]
                                }
                            },
                            |child, parent| {
                                if let Type::UnionVariant(uv) = child {
                                    uv.union = Some(parent);
                                }
                            },
                        );
                        new_id
                    }
                    Type::UnionVariant(v) => self.create_type(Type::UnionVariant(v)),
                    Type::Interface(mut i) => {
                        let (new_operations, new_operation_names, _) =
                            self.clone_named_map(&i.operation_names, &i.operations);
                        i.operations = new_operations;
                        i.operation_names = new_operation_names;
                        let new_id = self.create_type(Type::Interface(i));
                        self.reparent_children(
                            new_id,
                            |t| {
                                if let Type::Interface(new_i) = t {
                                    new_i
                                        .operation_names
                                        .iter()
                                        .filter_map(|n| new_i.operations.get(n).copied())
                                        .collect()
                                } else {
                                    vec![]
                                }
                            },
                            |child, parent| {
                                if let Type::Operation(op) = child {
                                    op.interface_ = Some(parent);
                                }
                            },
                        );
                        new_id
                    }
                    Type::Operation(o) => self.create_type(Type::Operation(o)),
                    Type::Scalar(mut s) => {
                        s.derived_scalars = Vec::new();
                        self.create_type(Type::Scalar(s))
                    }
                    Type::Namespace(n) => self.create_type(Type::Namespace(n)),
                    Type::Tuple(t) => self.create_type(Type::Tuple(t)),
                    // Simple value types and others - just clone directly
                    _ => self.create_type(t),
                }
            }
            None => self.error_type,
        }
    }

    /// Deep-clone a named map (HashMap<String, TypeId> + Vec<String> order).
    /// Returns (new_map, new_names, original_ids_in_order).
    fn clone_named_map(
        &mut self,
        names: &[String],
        map: &HashMap<String, TypeId>,
    ) -> (HashMap<String, TypeId>, Vec<String>, Vec<TypeId>) {
        let mut new_map = HashMap::new();
        let mut new_names = Vec::new();
        let mut orig_ids = Vec::new();
        for name in names {
            if let Some(&child_id) = map.get(name) {
                orig_ids.push(child_id);
                let cloned_id = self.clone_type(child_id);
                new_map.insert(name.clone(), cloned_id);
                new_names.push(name.clone());
            }
        }
        (new_map, new_names, orig_ids)
    }

    /// Collect child TypeIds from a newly created parent, then re-parent each child.
    fn reparent_children(
        &mut self,
        parent_id: TypeId,
        collect: impl Fn(&Type) -> Vec<TypeId>,
        reparent: impl Fn(&mut Type, TypeId),
    ) {
        let child_ids: Vec<TypeId> = self.get_type(parent_id).map(collect).unwrap_or_default();
        for child_id in child_ids {
            if let Some(child) = self.get_type_mut(child_id) {
                reparent(child, parent_id);
            }
        }
    }
}
