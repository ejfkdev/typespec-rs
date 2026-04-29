// Ported from TS e2e/cli/scenarios/warn/main.tsp
// expect-warning(deprecated)
#deprecated "Deprecated"
model Foo {}

model Bar {
  foo: Foo;
}
