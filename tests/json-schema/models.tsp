model Address {
  street: string;
  city?: string;
}

model Person {
  name: string;
  nickname?: string;
  home: Address;
  previous: Address[];
  parent: Person | null;
  tags: string[];
  inline: { x: int32; y?: int32 };
}

model Employee extends Person {
  badge: int32;
}

model Tags is string[];

model Empty {}
