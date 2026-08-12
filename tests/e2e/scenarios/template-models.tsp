// Template models and template references
model Page<T> {
  item: T;
  count: int32;
}

model Pet {
  name: string;
  age: int32;
}

model PetPage is Page<Pet> {}
