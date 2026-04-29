// Union types with variants
union PetKind {
  cat: Cat,
  dog: Dog,
}

model Cat {
  name: string;
  meow: boolean;
}

model Dog {
  name: string;
  bark: boolean;
}

alias StringOrInt = string | int32;
