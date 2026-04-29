// Pet Store API - comprehensive model/interface/enum scenario
model Pet {
  id: int32;
  name: string;
  tag?: string;
}

enum Status {
  available,
  pending,
  sold,
}

model Error {
  code: int32;
  message: string;
}

interface PetStore {
  listPets(limit?: int32): Pet[];
  getPet(petId: int32): Pet | Error;
}
