//! TypeSpec source for standard library decorators
//!
//! Ported from TypeSpec compiler/lib/std/decorators.tsp

/// The TypeSpec source for the standard library decorators
pub const STD_DECORATORS_TSP: &str = r#"
import "../../dist/src/lib/tsp-index.js";

using TypeSpec.Reflection;

namespace TypeSpec;

extern dec summary(target: unknown, summary: valueof string);
extern dec doc(target: unknown, doc: valueof string, formatArgs?: {});
extern dec returnsDoc(target: Operation, doc: valueof string);
extern dec errorsDoc(target: Operation, doc: valueof string);

model ServiceOptions {
  title?: string;
}

extern dec service(target: Namespace, options?: valueof ServiceOptions);
extern dec error(target: Model);
extern dec mediaTypeHint(target: Model | Scalar | Enum | Union, mediaType: valueof string);
extern dec format(target: string | ModelProperty, format: valueof string);
extern dec pattern(target: string | bytes | ModelProperty, pattern: valueof string, validationMessage?: valueof string);
extern dec minLength(target: string | ModelProperty, value: valueof integer);
extern dec maxLength(target: string | ModelProperty, value: valueof integer);
extern dec minItems(target: unknown[] | ModelProperty, value: valueof integer);
extern dec maxItems(target: unknown[] | ModelProperty, value: valueof integer);
extern dec minValue(target: numeric | utcDateTime | offsetDateTime | plainDate | plainTime | duration | ModelProperty, value: valueof numeric);
extern dec maxValue(target: numeric | utcDateTime | offsetDateTime | plainDate | plainTime | duration | ModelProperty, value: valueof numeric);
extern dec minValueExclusive(target: numeric | utcDateTime | offsetDateTime | plainDate | plainTime | duration | ModelProperty, value: valueof numeric);
extern dec maxValueExclusive(target: numeric | utcDateTime | offsetDateTime | plainDate | plainTime | duration | ModelProperty, value: valueof numeric);
extern dec secret(target: Scalar | ModelProperty | Model | Union | Enum);
extern dec tag(target: Namespace | Interface | Operation, tag: valueof string);
extern dec friendlyName(target: unknown, name: valueof string, formatArgs?: unknown);
extern dec key(target: ModelProperty, altName?: valueof string);
extern dec overload(target: Operation, overloadbase: Operation);
extern dec encodedName(target: unknown, mimeType: valueof string, name: valueof string);

model DiscriminatedOptions {
  envelope?: valueof "object" | "none";
  discriminatorPropertyName?: string;
  envelopePropertyName?: string;
}

extern dec discriminated(target: Union, options?: valueof DiscriminatedOptions);
extern dec discriminator(target: Model, propertyName: valueof string);

enum DateTimeKnownEncoding { rfc3339, rfc7231, unixTimestamp }
enum DurationKnownEncoding { ISO8601, seconds, milliseconds }
enum BytesKnownEncoding { base64, base64url }
enum ArrayEncoding { pipeDelimited, spaceDelimited, commaDelimited, newlineDelimited }

extern dec encode(target: Scalar | ModelProperty, encodingOrEncodeAs: (valueof string | EnumMember) | Scalar, encodedAs?: Scalar);

model ExampleOptions {
  title?: string;
  description?: string;
}

extern dec example(target: Model | Enum | Scalar | Union | ModelProperty | UnionVariant, example: valueof unknown, options?: valueof ExampleOptions);

model OperationExample {
  parameters?: unknown;
  returnType?: unknown;
}

extern dec opExample(target: Operation, example: valueof OperationExample, options?: valueof ExampleOptions);

extern dec withOptionalProperties(target: Model);
extern dec withoutDefaultValues(target: Model);
extern dec withoutOmittedProperties(target: Model, omit: string | Union);
extern dec withPickedProperties(target: Model, pick: string | Union);

extern dec list(target: Operation);
extern dec offset(target: ModelProperty);
extern dec pageIndex(target: ModelProperty);
extern dec pageSize(target: ModelProperty);
extern dec pageItems(target: ModelProperty);
extern dec continuationToken(target: ModelProperty);
extern dec nextLink(target: ModelProperty);
extern dec prevLink(target: ModelProperty);
extern dec firstLink(target: ModelProperty);
extern dec lastLink(target: ModelProperty);

extern dec inspectType(target: unknown, text: valueof string);
extern dec inspectTypeName(target: unknown, text: valueof string);
"#;
