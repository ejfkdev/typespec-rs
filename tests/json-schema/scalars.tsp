@minLength(2)
scalar Slug extends string;

@maxValue(10)
scalar Small extends uint8;

model Numbers {
  a: int8;
  b: int16;
  c: int32;
  d: int64;
  e: uint8;
  f: uint16;
  g: uint32;
  h: uint64;
  i: float32;
  j: float64;
  k: integer;
  l: float;
  m: numeric;
  n: safeint;
  o: decimal;
}

model Others {
  s: string;
  b: boolean;
  dt: utcDateTime;
  odt: offsetDateTime;
  date: plainDate;
  time: plainTime;
  dur: duration;
  link: url;
  data: bytes;
  ts: unixTimestamp32;
  slug: Slug;
  small: Small;
  anything: unknown;
}
