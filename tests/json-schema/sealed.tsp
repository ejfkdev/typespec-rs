model Inner {
  x: string;
}

model Base {
  id: string;
}

model Derived extends Base {
  d: string;
}

model Spread {
  a: string;
  ...Record<int32>;
}

model IsRecord is Record<string>;

model ExtendsRecord extends Record<Inner> {
  b: Inner;
}

model Holder {
  anon: { q: string };
  nested: Record<Inner>;
  list: Inner[];
}

model Empty {}
