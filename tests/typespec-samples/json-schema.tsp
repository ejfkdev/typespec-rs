model Person {
  firstName: string;
  lastName: string;
  @minValue(0) age: int32;
  address: Address;
  @uniqueItems nickNames?: string[];
  cars?: Car[];
}

model Address {
  street: string;
  city: string;
  state: string;
  zipCode: string;
}

enum CarMake {
  Toyota,
  Honda,
  Ford,
  Tesla,
}

model Car {
  make: CarMake;
  model: string;
  year: int32;
}
