//! Checker Enum Tests
//!
//! Ported from TypeSpec compiler/test/checker/enum.test.ts
//!
//! Skipped (needs diagnostics system):
//!
//! Skipped (needs decorator execution):
//! - Enums referenced from decorator on namespace
//! - Decorate spread member independently

use crate::checker::Type;
use crate::checker::test_utils::check;

#[test]
fn test_valueless_enum() {
    // Ported from: "can be valueless"
    let checker = check("enum E { A, B, C }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.member_names.len(), 3);
            assert!(e.members.contains_key("A"));
            assert!(e.members.contains_key("B"));
            assert!(e.members.contains_key("C"));

            // Valueless members should have None value
            let a_id = e.members.get("A").copied().unwrap();
            let a_member = checker.get_type(a_id).cloned().unwrap();
            match a_member {
                Type::EnumMember(m) => {
                    assert!(m.value.is_none(), "Valueless member should have None value");
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type, got {:?}", t.kind_name()),
    }
}

#[test]
fn test_enum_with_string_values() {
    // Ported from: "can have values"
    let checker = check(r#"enum E { A: "a"; B: "b"; C: "c"; }"#);
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.member_names.len(), 3);

            // Check A value
            let a_id = e.members.get("A").copied().unwrap();
            let a_member = checker.get_type(a_id).cloned().unwrap();
            match a_member {
                Type::EnumMember(m) => {
                    assert!(m.value.is_some(), "Member A should have a value");
                    let val_type = checker.get_type(m.value.unwrap()).cloned().unwrap();
                    match val_type {
                        Type::String(s) => assert_eq!(s.value, "a"),
                        other => panic!("Expected String value for A, got {:?}", other.kind_name()),
                    }
                }
                _ => panic!("Expected EnumMember"),
            }

            // Check B value
            let b_id = e.members.get("B").copied().unwrap();
            let b_member = checker.get_type(b_id).cloned().unwrap();
            match b_member {
                Type::EnumMember(m) => {
                    let val_type = checker.get_type(m.value.unwrap()).cloned().unwrap();
                    match val_type {
                        Type::String(s) => assert_eq!(s.value, "b"),
                        other => panic!("Expected String value for B, got {:?}", other.kind_name()),
                    }
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_with_numeric_values() {
    let checker = check("enum E { A: 1; B: 2; C: 3; }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            let a_id = e.members.get("A").copied().unwrap();
            let a_member = checker.get_type(a_id).cloned().unwrap();
            match a_member {
                Type::EnumMember(m) => {
                    assert!(m.value.is_some());
                    let val_type = checker.get_type(m.value.unwrap()).cloned().unwrap();
                    match val_type {
                        Type::Number(n) => assert_eq!(n.value, 1.0),
                        other => panic!("Expected Number value, got {:?}", other.kind_name()),
                    }
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_as_model_property() {
    // Ported from: "can be a model property"
    let checker = check("enum E { A, B, C } model Foo { prop: E; }");
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Model(m) => {
            let prop_id = m.properties.get("prop").copied().unwrap();
            let prop = checker.get_type(prop_id).cloned().unwrap();
            match prop {
                Type::ModelProperty(p) => {
                    let resolved = checker.get_type(p.r#type).cloned().unwrap();
                    assert!(
                        matches!(resolved, Type::Enum(_)),
                        "Expected Enum type for property, got {:?}",
                        resolved.kind_name()
                    );
                }
                _ => panic!("Expected ModelProperty"),
            }
        }
        _ => panic!("Expected Model type"),
    }
}

#[test]
fn test_enum_is_finished() {
    let checker = check("enum E { A, B }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    assert!(t.is_finished(), "Enum type should be finished");
}

#[test]
fn test_enum_member_name() {
    let checker = check("enum Direction { North, South, East, West }");
    let e_type = checker.declared_types.get("Direction").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.member_names, vec!["North", "South", "East", "West"]);
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_with_decorator() {
    let checker = check("@doc enum E { A, B }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.decorators.len(), 1, "Enum should have 1 decorator");
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_member_with_decorator() {
    let checker = check("enum E { @doc A, B }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            let a_id = e.members.get("A").copied().unwrap();
            let a_member = checker.get_type(a_id).cloned().unwrap();
            match a_member {
                Type::EnumMember(m) => {
                    assert_eq!(m.decorators.len(), 1, "Member A should have 1 decorator");
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type"),
    }
}

// ============================================================================
// Enum Diagnostic Tests
// ============================================================================

#[test]
fn test_enum_duplicate_member_detected() {
    // Ported from: "can't have duplicate variants"
    let checker = check("enum A { A, A }");
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "enum-member-duplicate"),
        "Should report enum-member-duplicate: {:?}",
        diags
    );
}

#[test]
fn test_enum_no_duplicate_member_no_error() {
    // No diagnostic when members are unique
    let checker = check("enum E { A, B, C }");
    let diags = checker.diagnostics();
    assert!(diags.is_empty(), "Should have no diagnostics: {:?}", diags);
}

#[test]
fn test_enum_duplicate_member_message() {
    // Ported from: "can't have duplicate variants" - verify message content
    let checker = check("enum A { A, A }");
    let diags = checker.diagnostics();
    let dup_diag = diags
        .iter()
        .find(|d| d.code == "enum-member-duplicate")
        .unwrap();
    assert!(
        dup_diag.message.contains("A"),
        "Message should mention member 'A': {}",
        dup_diag.message
    );
}

#[test]
fn test_enum_member_parent_reference() {
    // Verify enum members reference their parent enum
    let checker = check("enum E { A, B }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            let a_id = e.members.get("A").copied().unwrap();
            let a_member = checker.get_type(a_id).cloned().unwrap();
            match a_member {
                Type::EnumMember(m) => {
                    assert_eq!(m.r#enum, Some(e_type), "A.enum should be E");
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_in_namespace() {
    let checker = check("namespace MyNs { enum E { A, B } }");
    let ns_type = checker.declared_types.get("MyNs").copied().unwrap();
    let t = checker.get_type(ns_type).cloned().unwrap();
    match t {
        Type::Namespace(ns) => {
            assert!(
                ns.enums.contains_key("E"),
                "Namespace should contain E enum"
            );
        }
        _ => panic!("Expected Namespace type"),
    }
}

#[test]
fn test_enum_multiple_decorators() {
    let checker = check("@foo @bar enum E { A, B }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.decorators.len(), 2, "Enum should have 2 decorators");
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_mixed_members() {
    // Enum with both valueless and valued members
    let checker = check(r#"enum E { A, B: "b", C, D: 1 }"#);
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.member_names.len(), 4);
            assert!(e.members.contains_key("A"));
            assert!(e.members.contains_key("B"));
            assert!(e.members.contains_key("C"));
            assert!(e.members.contains_key("D"));

            // A should be valueless
            let a_id = e.members.get("A").copied().unwrap();
            let a = checker.get_type(a_id).cloned().unwrap();
            match a {
                Type::EnumMember(m) => assert!(m.value.is_none()),
                _ => panic!("Expected EnumMember"),
            }

            // B should have string value
            let b_id = e.members.get("B").copied().unwrap();
            let b = checker.get_type(b_id).cloned().unwrap();
            match b {
                Type::EnumMember(m) => {
                    assert!(m.value.is_some());
                    let val = checker.get_type(m.value.unwrap()).cloned().unwrap();
                    assert!(matches!(val, Type::String(s) if s.value == "b"));
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_single_member() {
    let checker = check("enum E { A }");
    let e_type = checker.declared_types.get("E").copied().unwrap();
    let t = checker.get_type(e_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.member_names.len(), 1);
            assert!(e.members.contains_key("A"));
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_no_duplicate_member_no_error_multiple() {
    // Multiple unique members, no error
    let checker = check("enum E { A: 1, B: 2, C: 3, D: 4 }");
    let diags = checker.diagnostics();
    assert!(
        !diags.iter().any(|d| d.code == "enum-member-duplicate"),
        "Should NOT report enum-member-duplicate: {:?}",
        diags
    );
}

// ============================================================================
// Enum Spread Member Tests
// Ported from TypeSpec compiler/test/checker/enum.test.ts
// ============================================================================

#[test]
fn test_enum_can_have_spread_members() {
    // Ported from TS: "can have spread members"
    let checker = check(
        r#"
        enum Bar {
            One: "1",
            Two: "2",
        }
        enum Foo {
            ...Bar,
            Three: "3"
        }
    "#,
    );
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();
    let t = checker.get_type(foo_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            // Foo should have 3 members: One, Two, Three
            assert_eq!(e.member_names.len(), 3, "Foo should have 3 members");
            assert!(e.members.contains_key("One"), "Foo should have 'One'");
            assert!(e.members.contains_key("Two"), "Foo should have 'Two'");
            assert!(e.members.contains_key("Three"), "Foo should have 'Three'");

            // Spread member's enum should point to Foo (the new parent)
            let one_id = e.members.get("One").copied().unwrap();
            let one_member = checker.get_type(one_id).cloned().unwrap();
            match one_member {
                Type::EnumMember(m) => {
                    assert_eq!(
                        m.r#enum,
                        Some(foo_type),
                        "Spread member One.enum should be Foo"
                    );
                    assert!(
                        m.source_member.is_some(),
                        "Spread member should have sourceMember"
                    );
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type"),
    }

    // Bar should still have 2 members, unaffected
    let bar_type = checker.declared_types.get("Bar").copied().unwrap();
    let t = checker.get_type(bar_type).cloned().unwrap();
    match t {
        Type::Enum(e) => {
            assert_eq!(e.member_names.len(), 2, "Bar should still have 2 members");
            let one_id = e.members.get("One").copied().unwrap();
            let one_member = checker.get_type(one_id).cloned().unwrap();
            match one_member {
                Type::EnumMember(m) => {
                    assert_eq!(m.r#enum, Some(bar_type), "Bar.One.enum should be Bar");
                    assert!(
                        m.source_member.is_none(),
                        "Original member should not have sourceMember"
                    );
                }
                _ => panic!("Expected EnumMember"),
            }
        }
        _ => panic!("Expected Enum type"),
    }
}

#[test]
fn test_enum_spread_source_member_tracking() {
    // Ported from TS: Foo.members.get("One").sourceMember === Bar.members.get("One")
    let checker = check(
        r#"
        enum Bar { One: "1", Two: "2" }
        enum Foo { ...Bar }
    "#,
    );

    let bar_type = checker.declared_types.get("Bar").copied().unwrap();
    let foo_type = checker.declared_types.get("Foo").copied().unwrap();

    // Get the source member from Bar
    let bar_one_id = {
        let t = checker.get_type(bar_type).cloned().unwrap();
        match t {
            Type::Enum(e) => e.members.get("One").copied().unwrap(),
            _ => panic!("Expected Enum"),
        }
    };

    // Get the spread member from Foo and check its source_member points to Bar.One
    let foo_one_id = {
        let t = checker.get_type(foo_type).cloned().unwrap();
        match t {
            Type::Enum(e) => e.members.get("One").copied().unwrap(),
            _ => panic!("Expected Enum"),
        }
    };

    let foo_one = checker.get_type(foo_one_id).cloned().unwrap();
    match foo_one {
        Type::EnumMember(m) => {
            assert_eq!(
                m.source_member,
                Some(bar_one_id),
                "Spread member's sourceMember should point to the original Bar.One"
            );
        }
        _ => panic!("Expected EnumMember"),
    }
}

#[test]
fn test_enum_spread_non_enum_diagnostic() {
    // Spreading a non-enum type should report "spread-enum" diagnostic
    let checker = check(
        r#"
        model Foo {}
        enum E { ...Foo }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "spread-enum"),
        "Should report spread-enum when spreading non-enum: {:?}",
        diags
    );
}

#[test]
fn test_enum_spread_duplicate_member_detected() {
    // Spreading an enum with overlapping member names should report enum-member-duplicate
    let checker = check(
        r#"
        enum Bar { One: "1" }
        enum Foo { One: "override", ...Bar }
    "#,
    );
    let diags = checker.diagnostics();
    assert!(
        diags.iter().any(|d| d.code == "enum-member-duplicate"),
        "Should report enum-member-duplicate for spread duplicate: {:?}",
        diags
    );
}
