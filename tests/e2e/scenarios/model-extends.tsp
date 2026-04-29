// Model extends and property inheritance
model Animal {
  name: string;
  age: int32;
}

model Dog extends Animal {
  breed: string;
}

model Cat extends Animal {
  indoor: boolean;
}
