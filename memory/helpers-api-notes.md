---
name: helpers-api-and-decorator-evaluation
description: Key findings from implementing helpers.rs, evaluate_std_decorators, and their tests
type: project
---

## Decorator API and Evaluation Notes

**find_decorator/get_effective_decorators bug**: `get_effective_decorators()` only handles `Type::Operation` (returns interface+operation decorators). For all other types it returns empty Vec. `find_decorator()` was originally calling `get_effective_decorators()`, which meant it never found decorators on Model, Enum, etc. **Fix**: `find_decorator` now uses `get_effective_decorators` only for Operation, and `Type.decorators()` directly for other types.

**Decorator definition is often None**: Built-in TypeSpec decorators (`@doc`, `@summary`, `@minValue`, etc.) don't have `DecoratorType` instances registered in the namespace hierarchy. `resolve_decorator_by_name("doc")` returns `None` because these are only handled at the stdlib level. This means `DecoratorApplication.definition` is `None` for built-in decorators. Both `decorator_matches_name()` and `evaluate_std_decorators()` need AST-based name resolution as fallback.

**evaluate_std_decorators**: Must use AST node name as fallback when `definition` is `None`. The method iterates stored decorators, extracts names via `definition → DecoratorType.name` with `AST → DecoratorExpression.target → get_identifier_name()` fallback, then applies side effects to both StateAccessors AND Type.doc/summary fields.

**TypeSpec decorator syntax**: Decorators go BEFORE the declaration keyword (`@doc("...") model Foo {}`), not after the name. Property decorators go before the property name (`@doc("...") name: string`).

**Type::String vs Type::Scalar**: In typespec-rs, `string` resolves to `Type::Scalar` (not `Type::String`). `Type::String` is for string literal types. When checking property types, accept both.
