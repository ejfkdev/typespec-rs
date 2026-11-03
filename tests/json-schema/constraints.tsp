model Constrained {
  @minValue(1) @maxValue(5) rating: int8;
  @minValueExclusive(0) @maxValueExclusive(1.5) ratio: float64;
  @minLength(2) @maxLength(8) @pattern("^[a-z]+$") slug: string;
  @minItems(1) @maxItems(3) codes: int32[];
  @format("email") email: string;
  @format("uuid") id?: string;
}
