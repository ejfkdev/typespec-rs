model Animal {
  kind: string;
  name: string;
}

model Dog extends Animal {
  kind: "dog";
  breed: string;
}

model Cat extends Animal {
  kind: "cat";
  meows: boolean;
}

@discriminated
union Pet {
  dog: Dog,
  cat: Cat
}
