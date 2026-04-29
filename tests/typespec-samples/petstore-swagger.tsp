namespace SwaggerPetstore;

model Pet {
  id: int64;
  name: string;
  tag?: string;
}

model Pets is Pet[];

model Error {
  code: int32;
  message: string;
}

@tag("pets")
@route("/pets")
@get
op listPets(@query limit?: int32): Pets | Error;

@tag("pets")
@route("/pets")
@post
op createPets(): void | Error;

@tag("pets")
@route("/pets/{petId}")
@get
op showPetById(@path petId: string): Pet | Error;
