//! TypeSpec source for HTTP library decorators
//!
//! Ported from TypeSpec @typespec/http package

/// The TypeSpec source for the HTTP library decorators
pub const HTTP_DECORATORS_TSP: &str = r#"
namespace TypeSpec.Http;

using TypeSpec.Reflection;

model HeaderOptions {
  name?: string;
  explode?: boolean;
}

extern dec header(target: ModelProperty, headerNameOrOptions?: valueof string | HeaderOptions);

model CookieOptions {
  name?: string;
}

extern dec cookie(target: ModelProperty, cookieNameOrOptions?: valueof string | CookieOptions);

model QueryOptions {
  name?: string;
  explode?: boolean;
}

extern dec query(target: ModelProperty, queryNameOrOptions?: valueof string | QueryOptions);

model PathOptions {
  name?: string;
  explode?: boolean;
  style?: valueof "simple" | "label" | "matrix" | "fragment" | "path";
  allowReserved?: boolean;
}

extern dec path(target: ModelProperty, paramNameOrOptions?: valueof string | PathOptions);

extern dec body(target: ModelProperty);
extern dec bodyRoot(target: ModelProperty);
extern dec bodyIgnore(target: ModelProperty);
extern dec multipartBody(target: ModelProperty);
extern dec statusCode(target: ModelProperty);

extern dec get(target: Operation);
extern dec put(target: Operation);
extern dec post(target: Operation);

model PatchOptions {
  implicitOptionality?: boolean;
}

extern dec patch(target: Operation, options?: valueof PatchOptions);

extern dec delete(target: Operation);
extern dec head(target: Operation);

extern dec server(
  target: Namespace,
  url: valueof string,
  description?: valueof string,
  parameters?: Record<unknown>
);

extern dec useAuth(target: Namespace | Interface | Operation, auth: {} | Union | {}[]);
extern dec route(target: Namespace | Interface | Operation, path: valueof string);
extern dec sharedRoute(target: Operation);
"#;

/// The TypeSpec source for the HTTP library private decorators
/// Ported from TS packages/http/lib/private.decorators.tsp
pub const HTTP_PRIVATE_DECORATORS_TSP: &str = r#"
namespace TypeSpec.Http.Private;

extern dec plainData(target: TypeSpec.Reflection.Model);
extern dec httpFile(target: TypeSpec.Reflection.Model);
extern dec httpPart(
  target: TypeSpec.Reflection.Model,
  type: unknown,
  options: valueof HttpPartOptions
);

extern dec includeInapplicableMetadataInPayload(target: unknown, value: valueof boolean);

enum MergePatchVisibilityMode {
  Update,
  CreateOrUpdate,
}

model ApplyMergePatchOptions {
  visibilityMode: MergePatchVisibilityMode;
}

#deprecated "applyMergePatch is deprecated and will be removed in a future release."
extern dec applyMergePatch(
  target: Reflection.Model,
  source: Reflection.Model,
  nameTemplate: valueof string,
  options: valueof ApplyMergePatchOptions
);

extern dec mergePatchModel(target: Reflection.Model, source: Reflection.Model);
extern dec mergePatchProperty(target: Reflection.ModelProperty, source: Reflection.ModelProperty);
"#;

/// The TypeSpec source for the HTTP library main types
/// Ported from TS packages/http/lib/main.tsp
pub const HTTP_MAIN_TSP: &str = r#"
namespace TypeSpec.Http;

using Private;

@doc("")
model Response<Status> {
  @doc("The status code.")
  @statusCode
  statusCode: Status;
}

@doc("")
model Body<Type> {
  @body
  @doc("The body type of the operation request or response.")
  body: Type;
}

model LocationHeader {
  @doc("The Location header contains the URL where the status of the long running operation can be checked.")
  @header
  location: string;
}

model OkResponse is Response<200>;
model CreatedResponse is Response<201>;
model AcceptedResponse is Response<202>;
model NoContentResponse is Response<204>;
model MovedResponse is Response<301> {
  ...LocationHeader;
}
model NotModifiedResponse is Response<304>;
model BadRequestResponse is Response<400>;
model UnauthorizedResponse is Response<401>;
model ForbiddenResponse is Response<403>;
model NotFoundResponse is Response<404>;
model ConflictResponse is Response<409>;

@plainData
model PlainData<Data> {
  ...Data;
}

@Private.httpFile
model File<ContentType extends string = string, Contents extends bytes | string = bytes> {
  contentType?: ContentType;
  filename?: string;
  contents: Contents;
}

model HttpPartOptions {
  name?: string;
}

@Private.httpPart(Type, Options)
model HttpPart<Type, Options extends valueof HttpPartOptions = #{}> {}

model Link {
  target: url;
  rel: string;
  attributes?: Record<unknown>;
}

scalar LinkHeader<T extends Record<url> | Link[]> extends string;
"#;

/// The TypeSpec source for the HTTP library auth types
pub const HTTP_AUTH_TSP: &str = r#"
namespace TypeSpec.Http;

model BasicAuth {
  type: "http";
  scheme: "Basic";
}

model BearerAuth {
  type: "http";
  scheme: "Bearer";
}

model ApiKeyAuth<location extends "header" | "query" | "cookie", name extends string> {
  type: "apiKey";
  in: location;
  name: name;
}

model Oauth2Auth<flows extends OAuth2Flow[]> {
  type: "oauth2";
  flows: flows;
}

model OpenIDConnectAuth<openIdConnectUrl extends string> {
  type: "openIdConnect";
  openIdConnectUrl: openIdConnectUrl;
}

model NoAuth {
  type: "none";
}

union Auth {
  BasicAuth,
  BearerAuth,
  ApiKeyAuth<"header" | "query" | "cookie", string>,
  Oauth2Auth<OAuth2Flow[]>,
  OpenIDConnectAuth<string>,
  NoAuth,
}

model OAuth2Flow {
  authorizationUrl?: string;
  tokenUrl?: string;
  refreshUrl?: string;
  scopes: Record<string>;
}
"#;
