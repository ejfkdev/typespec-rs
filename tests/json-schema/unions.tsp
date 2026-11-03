enum Color {
  Red,
  Green,
}

enum Level {
  Low: 1,
  High: 10,
}

enum Named {
  First: "first",
  Second: "second",
}

union Shape {
  circle: string,
  square: int32,
}

model Choices {
  color: Color;
  level: Level;
  red: Color.Red;
  shape: Shape;
  kind: "a" | "b";
  count: 3;
  flag: true;
  either: string | int32;
  maybe: string | null;
  pair: [string, int32];
}
