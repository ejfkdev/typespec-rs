// Template models and template references
// Note: T property type produces invalid-ref until full template resolution
// expect-error(invalid-ref)
model Page<T> {
  item: T;
  count: int32;
}

model Pet {
  name: string;
  age: int32;
}

model PetPage is Page<Pet> {}
