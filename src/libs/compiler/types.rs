//! TypeSpec standard library type definitions
//!
//! Ported from TypeSpec compiler/lib/std/decorators.tsp

use crate::checker::types::TypeId;

// ============================================================================
// Types
// ============================================================================

/// Service details.
/// Ported from TS ServiceDetails interface.
#[derive(Debug, Clone, Default)]
pub struct ServiceDetails {
    /// Title of the service
    pub title: Option<String>,
}

/// Service information.
/// Ported from TS Service interface (extends ServiceDetails).
#[derive(Debug, Clone)]
pub struct Service {
    /// The namespace type that represents this service
    pub namespace_type: TypeId,
    /// Service details
    pub details: ServiceDetails,
}

/// Service options.
/// Ported from TS ServiceOptions model.
#[derive(Debug, Clone)]
pub struct ServiceOptions {
    /// Title of the service
    pub title: Option<String>,
}

/// Options for @discriminated decorator.
/// Ported from TS DiscriminatedOptions model.
#[derive(Debug, Clone)]
pub struct DiscriminatedOptions {
    /// How is the discriminated union serialized
    pub envelope: Option<DiscriminatedEnvelope>,
    /// Name of the discriminator property
    pub discriminator_property_name: Option<String>,
    /// Name of the property enveloping the data
    pub envelope_property_name: Option<String>,
}

/// Envelope type for discriminated unions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscriminatedEnvelope {
    Object,
    None,
}

/// Example options.
/// Ported from TS ExampleOptions model.
#[derive(Debug, Clone)]
pub struct ExampleOptions {
    /// Title of the example
    pub title: Option<String>,
    /// Description of the example
    pub description: Option<String>,
}

/// Operation example configuration.
/// Ported from TS OperationExample model.
#[derive(Debug, Clone)]
pub struct OperationExample {
    /// Example request body (serialized as string)
    pub parameters: Option<String>,
    /// Example response body (serialized as string)
    pub return_type: Option<String>,
}

/// Pattern data for @pattern decorator.
/// Ported from TS PatternData interface.
#[derive(Debug, Clone)]
pub struct PatternData {
    /// The regex pattern
    pub pattern: String,
    /// Optional validation message
    pub validation_message: Option<String>,
}

/// Encode data for @encode decorator.
/// Ported from TS EncodeData interface.
#[derive(Debug, Clone)]
pub struct EncodeData {
    /// Encoding key (e.g. "rfc3339", "base64", or custom string)
    pub encoding: Option<String>,
    /// The type to encode as (TypeId reference)
    pub encode_as_type: Option<TypeId>,
}

/// Doc data with source information.
/// Re-exported from intrinsic_type_state to avoid duplication.
pub use crate::intrinsic_type_state::{DocData, DocSource};

string_enum! {
    /// Known encodings for date/time types.
    /// Ported from TS DateTimeKnownEncoding enum.
    pub enum DateTimeKnownEncoding {
        Rfc3339 => "rfc3339",
        Rfc7231 => "rfc7231",
        UnixTimestamp => "unixTimestamp",
    }
}

string_enum! {
    /// Known encodings for duration type.
    /// Ported from TS DurationKnownEncoding enum.
    pub enum DurationKnownEncoding {
        Iso8601 => "ISO8601",
        Seconds => "seconds",
        Milliseconds => "milliseconds",
    }
}

string_enum! {
    /// Known encodings for bytes type.
    /// Ported from TS BytesKnownEncoding enum.
    pub enum BytesKnownEncoding {
        Base64 => "base64",
        Base64url => "base64url",
    }
}

string_enum! {
    /// Encoding for serializing arrays.
    /// Ported from TS ArrayEncoding enum.
    pub enum ArrayEncoding {
        PipeDelimited => "pipeDelimited",
        SpaceDelimited => "spaceDelimited",
        CommaDelimited => "commaDelimited",
        NewlineDelimited => "newlineDelimited",
    }
}
