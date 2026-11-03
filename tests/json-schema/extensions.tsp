using TypeSpec.JsonSchema;

@extension("x-model", "m")
@extension("x-order", 2)
model Tagged {
  @extension("x-note", "hi") noted: string;
  @extension("x-flag", true) flagged: int32;
  @extension("x-list", #["a", "b"]) listed: string;
}

@extension("x-enum", "e")
enum Mode {
  On,
  Off,
}
