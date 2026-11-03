model Inner {
  x: string;
}

model Spread {
  a: string;
  ...Record<int32>;
}

model IsRecord is Record<string>;

model ExtendsRecord extends Record<Inner> {
  b: Inner;
}

model SpreadUnknown {
  ...Record<unknown>;
}

model Page<T> {
  item: T;
}

model Holder {
  nested: Record<Inner>;
  again: Record<Inner>;
  anything: Record<unknown>;
  ints: Record<int32>;
  page: Page<Inner>;
  literal: Record<"a">;
}
