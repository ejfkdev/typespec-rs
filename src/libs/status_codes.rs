//! HTTP Status Code Utilities
//!
//! Ported from TypeSpec packages/http/src/status-codes.ts and decorators.ts
//!
//! Provides validation, parsing, and description of HTTP status codes.
//! Status codes must be integers in the range 100-599.
//!
//! ## Types
//! - `HttpStatusCodeRange` - Flexible range with start/end (matches TS interface)
//! - `StatusCodeEntry` - Either a specific code, a range, or a wildcard
//! - `HttpStatusCodes` - List of status code entries

/// HTTP status code range.
/// Ported from TS `interface HttpStatusCodeRange { start: number; end: number }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpStatusCodeRange {
    /// Start of the range (inclusive)
    pub start: u16,
    /// End of the range (inclusive)
    pub end: u16,
}

impl HttpStatusCodeRange {
    /// Create a new status code range.
    /// Returns None if the range is invalid (start < 100, end > 599, or start > end).
    pub fn new(start: u16, end: u16) -> Option<Self> {
        if start < 100 || end > 599 || start > end {
            return None;
        }
        Some(Self { start, end })
    }

    /// Check if a status code falls within this range
    pub fn contains(&self, code: u16) -> bool {
        (self.start..=self.end).contains(&code)
    }

    /// Get a human-readable description for this range.
    /// Ported from TS `rangeDescription()`.
    pub fn description(&self) -> Option<&'static str> {
        range_description(self.start, self.end)
    }
}

/// A status code entry - either a specific code, a range, or a wildcard.
/// Ported from TS `type HttpStatusCodesEntry = HttpStatusCodeRange | number | "*"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCodeEntry {
    /// A specific status code
    Code(u16),
    /// A status code range
    Range(HttpStatusCodeRange),
    /// Wildcard (any status code)
    Wildcard,
}

/// Validate a status code value.
/// Ported from TS `validateStatusCode()`.
///
/// Returns the validated status code, or an error message if invalid.
pub fn validate_status_code(code: u16) -> Result<u16, &'static str> {
    if !(100..=599).contains(&code) {
        return Err("Status code must be between 100 and 599");
    }
    Ok(code)
}

/// Validate a status code from a string value.
/// Ported from TS `validateStatusCode()`.
///
/// Returns the validated status code, or an error message if invalid.
pub fn validate_status_code_str(code: &str) -> Result<u16, &'static str> {
    let code_as_number = code
        .parse::<u16>()
        .map_err(|_| "Status code is not a valid number")?;
    validate_status_code(code_as_number)
}

/// Check if a status code is in the information range (1xx)
pub fn is_informational(code: u16) -> bool {
    (100..=199).contains(&code)
}

/// Check if a status code is in the success range (2xx)
pub fn is_success(code: u16) -> bool {
    (200..=299).contains(&code)
}

/// Check if a status code is in the redirection range (3xx)
pub fn is_redirection(code: u16) -> bool {
    (300..=399).contains(&code)
}

/// Check if a status code is in the client error range (4xx)
pub fn is_client_error(code: u16) -> bool {
    (400..=499).contains(&code)
}

/// Check if a status code is in the server error range (5xx)
pub fn is_server_error(code: u16) -> bool {
    (500..=599).contains(&code)
}

/// Get the status code class (e.g., 200 for 2xx range)
pub fn status_code_class(code: u16) -> u16 {
    (code / 100) * 100
}

/// Check if two status codes are in the same class
pub fn is_same_class(a: u16, b: u16) -> bool {
    status_code_class(a) == status_code_class(b)
}

/// Get a human-readable description for a status code.
/// Ported from TS `getStatusCodeDescription()`.
///
/// Provides detailed descriptions for common status codes (matching TS),
/// and falls back to range descriptions for other codes.
/// Also supports `StatusCodeEntry` for ranges and wildcards.
pub fn get_status_code_description(code: u16) -> Option<&'static str> {
    match code {
        200 => Some("The request has succeeded."),
        201 => Some("The request has succeeded and a new resource has been created as a result."),
        202 => Some(
            "The request has been accepted for processing, but processing has not yet completed.",
        ),
        204 => Some("There is no content to send for this request, but the headers may be useful."),
        301 => Some(
            "The URL of the requested resource has been changed permanently. The new URL is given in the response.",
        ),
        304 => Some(
            "The client has made a conditional request and the resource has not been modified.",
        ),
        400 => Some("The server could not understand the request due to invalid syntax."),
        401 => Some("Access is unauthorized."),
        403 => Some("Access is forbidden."),
        404 => Some("The server cannot find the requested resource."),
        409 => Some("The request conflicts with the current state of the server."),
        412 => Some("Precondition failed."),
        503 => Some("Service unavailable."),
        _ => range_description(code, code),
    }
}

/// Get description for a StatusCodeEntry (code, range, or wildcard).
/// Ported from TS `getStatusCodeDescription()`.
pub fn get_status_code_entry_description(entry: &StatusCodeEntry) -> Option<&'static str> {
    match entry {
        StatusCodeEntry::Code(code) => get_status_code_description(*code),
        StatusCodeEntry::Range(range) => range.description(),
        StatusCodeEntry::Wildcard => Some("Any status code"),
    }
}

/// Get description for a status code range.
/// Ported from TS `rangeDescription()`.
fn range_description(start: u16, end: u16) -> Option<&'static str> {
    if start >= 100 && end <= 199 {
        Some("Informational")
    } else if start >= 200 && end <= 299 {
        Some("Successful")
    } else if start >= 300 && end <= 399 {
        Some("Redirection")
    } else if start >= 400 && end <= 499 {
        Some("Client error")
    } else if start >= 500 && end <= 599 {
        Some("Server error")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_status_codes() {
        assert!(validate_status_code(200).is_ok());
        assert!(validate_status_code(100).is_ok());
        assert!(validate_status_code(599).is_ok());
        assert_eq!(validate_status_code(200).unwrap(), 200);
    }

    #[test]
    fn test_validate_invalid_status_codes() {
        assert!(validate_status_code(99).is_err());
        assert!(validate_status_code(600).is_err());
        assert!(validate_status_code(0).is_err());
    }

    #[test]
    fn test_validate_status_code_str() {
        assert_eq!(validate_status_code_str("200"), Ok(200));
        assert_eq!(validate_status_code_str("404"), Ok(404));
        assert!(validate_status_code_str("abc").is_err());
        assert!(validate_status_code_str("99").is_err());
        assert!(validate_status_code_str("600").is_err());
    }

    #[test]
    fn test_status_code_categories() {
        assert!(is_informational(100));
        assert!(is_informational(199));
        assert!(!is_informational(200));

        assert!(is_success(200));
        assert!(is_success(299));
        assert!(!is_success(300));

        assert!(is_redirection(301));
        assert!(is_redirection(399));
        assert!(!is_redirection(400));

        assert!(is_client_error(400));
        assert!(is_client_error(499));
        assert!(!is_client_error(500));

        assert!(is_server_error(500));
        assert!(is_server_error(599));
        assert!(!is_server_error(600));
    }

    #[test]
    fn test_status_code_class() {
        assert_eq!(status_code_class(200), 200);
        assert_eq!(status_code_class(201), 200);
        assert_eq!(status_code_class(404), 400);
        assert_eq!(status_code_class(503), 500);
    }

    #[test]
    fn test_is_same_class() {
        assert!(is_same_class(200, 201));
        assert!(is_same_class(404, 400));
        assert!(!is_same_class(200, 301));
    }

    #[test]
    fn test_status_code_descriptions() {
        // Detailed descriptions matching TS
        assert_eq!(
            get_status_code_description(200),
            Some("The request has succeeded.")
        );
        assert_eq!(
            get_status_code_description(201),
            Some("The request has succeeded and a new resource has been created as a result.")
        );
        assert_eq!(
            get_status_code_description(404),
            Some("The server cannot find the requested resource.")
        );
        assert_eq!(
            get_status_code_description(409),
            Some("The request conflicts with the current state of the server.")
        );
        assert_eq!(
            get_status_code_description(412),
            Some("Precondition failed.")
        );
        // Falls back to range description
        assert_eq!(get_status_code_description(205), Some("Successful"));
        assert_eq!(get_status_code_description(418), Some("Client error"));
        assert_eq!(get_status_code_description(500), Some("Server error")); // 500 falls through to range
        assert_eq!(
            get_status_code_description(503),
            Some("Service unavailable.")
        ); // 503 has specific description
    }

    #[test]
    fn test_status_code_entry_descriptions() {
        let code_entry = StatusCodeEntry::Code(200);
        assert_eq!(
            get_status_code_entry_description(&code_entry),
            Some("The request has succeeded.")
        );

        let range = HttpStatusCodeRange::new(200, 299).unwrap();
        let range_entry = StatusCodeEntry::Range(range);
        assert_eq!(
            get_status_code_entry_description(&range_entry),
            Some("Successful")
        );

        let wildcard = StatusCodeEntry::Wildcard;
        assert_eq!(
            get_status_code_entry_description(&wildcard),
            Some("Any status code")
        );
    }

    #[test]
    fn test_create_status_code_range() {
        let range = HttpStatusCodeRange::new(200, 299);
        assert!(range.is_some());
        let range = range.unwrap();
        assert_eq!(range.start, 200);
        assert_eq!(range.end, 299);
        assert!(range.contains(200));
        assert!(range.contains(250));
        assert!(!range.contains(300));
    }

    #[test]
    fn test_range_description() {
        let range = HttpStatusCodeRange::new(200, 299).unwrap();
        assert_eq!(range.description(), Some("Successful"));

        let range = HttpStatusCodeRange::new(400, 499).unwrap();
        assert_eq!(range.description(), Some("Client error"));

        let range = HttpStatusCodeRange::new(500, 599).unwrap();
        assert_eq!(range.description(), Some("Server error"));
    }

    #[test]
    fn test_create_invalid_range() {
        assert!(HttpStatusCodeRange::new(99, 200).is_none());
        assert!(HttpStatusCodeRange::new(200, 600).is_none());
        assert!(HttpStatusCodeRange::new(300, 200).is_none());
    }
}
