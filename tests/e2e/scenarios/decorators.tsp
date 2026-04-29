// Decorators on models, properties, operations
@doc("A pet in the store")
model Pet {
  @doc("The pet's name")
  name: string;

  @minValue(0)
  age: int32;
}

@doc("Pet store operations")
interface PetStore {
  @doc("List all pets")
  listPets(): Pet[];
}
