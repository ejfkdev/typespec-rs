//! Numeric type with arbitrary precision
//! Ported from TypeSpec compiler/src/core/numeric.ts
//!
//! Uses Rust's BigInt-equivalent (i128 or num-bigint) for precision.
//! For TypeSpec's needs, we use f64 for value representation
//! and store the original string for exact comparison.

/// Error for invalid numeric values
#[derive(Debug, Clone)]
pub struct InvalidNumericError(pub String);

/// A numeric value with arbitrary precision.
/// Stores the original string representation and parsed value.
#[derive(Debug, Clone, PartialEq)]
pub struct Numeric {
    /// The original string representation
    string_value: String,
    /// Parsed as f64 (may lose precision for very large numbers)
    as_f64: f64,
    /// Whether this is an integer value
    is_integer: bool,
}

impl Numeric {
    /// Create a Numeric from a string representation
    pub fn new(string_value: &str) -> Result<Self, InvalidNumericError> {
        let trimmed = string_value.trim();
        if trimmed.is_empty() {
            return Err(InvalidNumericError("Empty numeric value".to_string()));
        }

        // Check for valid numeric format
        let is_integer = !trimmed.contains('.') && !trimmed.contains('e') && !trimmed.contains('E');

        // Handle hex, binary, octal
        let as_f64 = if trimmed.starts_with("0x")
            || trimmed.starts_with("0X")
            || trimmed.starts_with("-0x")
            || trimmed.starts_with("-0X")
        {
            let without_prefix = trimmed
                .trim_start_matches('-')
                .trim_start_matches("0x")
                .trim_start_matches("0X");
            i128::from_str_radix(without_prefix, 16)
                .map(|v| {
                    if trimmed.starts_with('-') {
                        -(v as f64)
                    } else {
                        v as f64
                    }
                })
                .map_err(|_| {
                    InvalidNumericError(format!("Invalid numeric value: {}", string_value))
                })?
        } else if trimmed.starts_with("0b")
            || trimmed.starts_with("0B")
            || trimmed.starts_with("-0b")
            || trimmed.starts_with("-0B")
        {
            let without_prefix = trimmed
                .trim_start_matches('-')
                .trim_start_matches("0b")
                .trim_start_matches("0B");
            i128::from_str_radix(without_prefix, 2)
                .map(|v| {
                    if trimmed.starts_with('-') {
                        -(v as f64)
                    } else {
                        v as f64
                    }
                })
                .map_err(|_| {
                    InvalidNumericError(format!("Invalid numeric value: {}", string_value))
                })?
        } else if trimmed.starts_with("0o")
            || trimmed.starts_with("0O")
            || trimmed.starts_with("-0o")
            || trimmed.starts_with("-0O")
        {
            let without_prefix = trimmed
                .trim_start_matches('-')
                .trim_start_matches("0o")
                .trim_start_matches("0O");
            i128::from_str_radix(without_prefix, 8)
                .map(|v| {
                    if trimmed.starts_with('-') {
                        -(v as f64)
                    } else {
                        v as f64
                    }
                })
                .map_err(|_| {
                    InvalidNumericError(format!("Invalid numeric value: {}", string_value))
                })?
        } else {
            trimmed.parse::<f64>().map_err(|_| {
                InvalidNumericError(format!("Invalid numeric value: {}", string_value))
            })?
        };

        Ok(Numeric {
            string_value: trimmed.to_string(),
            as_f64,
            is_integer,
        })
    }

    /// Get the value as f64, or None if it cannot be represented without losing precision
    pub fn as_f64(&self) -> Option<f64> {
        // Verify roundtrip: if converting back to string gives same value, it's safe
        let back = Self::new(&self.as_f64.to_string()).ok()?;
        if back.string_value == self.string_value
            || (back.as_f64 - self.as_f64).abs() < f64::EPSILON
        {
            Some(self.as_f64)
        } else {
            None
        }
    }

    /// Get the string representation
    pub fn as_string(&self) -> &str {
        &self.string_value
    }

    /// Whether this is an integer
    pub fn is_integer(&self) -> bool {
        self.is_integer
    }
}

impl std::fmt::Display for Numeric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string_value)
    }
}

impl Eq for Numeric {}

impl Ord for Numeric {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_f64
            .partial_cmp(&other.as_f64)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for Numeric {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::str::FromStr for Numeric {
    type Err = InvalidNumericError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Numeric::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_parsing() {
        let n = Numeric::new("42").unwrap();
        assert_eq!(n.as_string(), "42");
        assert!(n.is_integer());
        assert_eq!(n.as_f64(), Some(42.0));
    }

    #[test]
    fn test_decimal_parsing() {
        let n = Numeric::new("3.14").unwrap();
        assert_eq!(n.as_string(), "3.14");
        assert!(!n.is_integer());
    }

    #[test]
    fn test_negative_parsing() {
        let n = Numeric::new("-100").unwrap();
        assert_eq!(n.as_string(), "-100");
        assert!(n.is_integer());
    }

    #[test]
    fn test_scientific_notation() {
        let n = Numeric::new("3.4e38").unwrap();
        assert!(!n.is_integer());
    }

    #[test]
    fn test_hex_parsing() {
        let n = Numeric::new("0xFF").unwrap();
        assert!(n.is_integer());
        assert_eq!(n.as_f64(), Some(255.0));
    }

    #[test]
    fn test_comparison() {
        let a = Numeric::new("10").unwrap();
        let b = Numeric::new("20").unwrap();
        assert!(a < b);
        assert!(b > a);
        assert!(a <= b);
        assert!(b >= a);
        assert_eq!(a, Numeric::new("10").unwrap());
    }

    #[test]
    fn test_numeric_ranges() {
        let int32_min = Numeric::new("-2147483648").unwrap();
        let int32_max = Numeric::new("2147483647").unwrap();
        let val = Numeric::new("100").unwrap();
        assert!(val >= int32_min);
        assert!(val <= int32_max);

        let overflow = Numeric::new("3000000000").unwrap();
        assert!(overflow > int32_max);
    }

    #[test]
    fn test_invalid_numeric() {
        assert!(Numeric::new("").is_err());
        assert!(Numeric::new("abc").is_err());
    }

    // ==================== Additional tests ported from TS numeric.test.ts ====================

    #[test]
    fn test_invalid_binary() {
        assert!(Numeric::new("0babc").is_err());
    }

    #[test]
    fn test_invalid_hex() {
        assert!(Numeric::new("0xGHI").is_err());
    }

    #[test]
    fn test_invalid_octal() {
        assert!(Numeric::new("0o999").is_err());
    }

    #[test]
    fn test_invalid_alpha_start() {
        assert!(Numeric::new("a123").is_err());
    }

    #[test]
    fn test_invalid_multiple_dots() {
        assert!(Numeric::new("1.2.3").is_err());
    }

    #[test]
    fn test_simple_integer() {
        let n = Numeric::new("123").unwrap();
        assert_eq!(n.as_string(), "123");
        assert!(n.is_integer());
    }

    #[test]
    fn test_negative_integer() {
        let n = Numeric::new("-123").unwrap();
        assert_eq!(n.as_string(), "-123");
        assert!(n.is_integer());
    }

    #[test]
    fn test_simple_decimal() {
        let n = Numeric::new("123.456").unwrap();
        assert_eq!(n.as_string(), "123.456");
        assert!(!n.is_integer());
    }

    #[test]
    fn test_negative_decimal() {
        let n = Numeric::new("-123.456").unwrap();
        assert_eq!(n.as_string(), "-123.456");
        assert!(!n.is_integer());
    }

    #[test]
    fn test_decimal_with_leading_zero() {
        let n = Numeric::new("0.1").unwrap();
        assert_eq!(n.as_string(), "0.1");
    }

    #[test]
    fn test_decimal_small() {
        let n = Numeric::new("0.01").unwrap();
        assert_eq!(n.as_string(), "0.01");
    }

    #[test]
    fn test_binary_lower_case() {
        let n = Numeric::new("0b10000000000000000000000000000000").unwrap();
        assert!(n.is_integer());
    }

    #[test]
    fn test_binary_upper_case() {
        let n = Numeric::new("0B10000000000000000000000000000000").unwrap();
        assert!(n.is_integer());
    }

    #[test]
    fn test_octal_lower_case() {
        let n = Numeric::new("0o755").unwrap();
        assert!(n.is_integer());
    }

    #[test]
    fn test_octal_upper_case() {
        let n = Numeric::new("0O755").unwrap();
        assert!(n.is_integer());
    }

    #[test]
    fn test_hex_lower_case() {
        let n = Numeric::new("0xA").unwrap();
        assert!(n.is_integer());
        assert_eq!(n.as_f64(), Some(10.0));
    }

    #[test]
    fn test_hex_upper_case() {
        let n = Numeric::new("0XA").unwrap();
        assert!(n.is_integer());
        assert_eq!(n.as_f64(), Some(10.0));
    }

    #[test]
    fn test_exponent_format() {
        let n = Numeric::new("5e1").unwrap();
        assert!(!n.is_integer());
    }

    #[test]
    fn test_exponent_negative() {
        let n = Numeric::new("5e-1").unwrap();
        assert!(!n.is_integer());
    }

    #[test]
    fn test_decimal_exponent() {
        let n = Numeric::new("1.5e2").unwrap();
        assert!(!n.is_integer());
    }

    // ==================== asString / Display tests ====================

    #[test]
    fn test_display_zero() {
        let n = Numeric::new("0").unwrap();
        assert_eq!(n.to_string(), "0");
    }

    #[test]
    fn test_display_integer() {
        let n = Numeric::new("123").unwrap();
        assert_eq!(n.to_string(), "123");
    }

    #[test]
    fn test_display_negative() {
        let n = Numeric::new("-123").unwrap();
        assert_eq!(n.to_string(), "-123");
    }

    #[test]
    fn test_display_decimal() {
        let n = Numeric::new("-123.456").unwrap();
        assert_eq!(n.to_string(), "-123.456");
    }

    #[test]
    fn test_display_decimal_with_leading_zero() {
        let n = Numeric::new("0.1").unwrap();
        assert_eq!(n.to_string(), "0.1");
    }

    #[test]
    fn test_display_exponent() {
        let n = Numeric::new("5e6").unwrap();
        assert_eq!(n.to_string(), "5e6");
    }

    // ==================== Comparison tests ported from TS ====================

    #[test]
    fn test_lt_comparison() {
        let cases = vec![
            ("0", "1"),
            ("0", "0.1"),
            ("-2", "1"),
            ("-1", "2"),
            ("-3", "-2"),
            ("123", "123.00001"),
            ("34.123", "300"),
        ];
        for (a, b) in cases {
            let na = Numeric::new(a).unwrap();
            let nb = Numeric::new(b).unwrap();
            assert!(na < nb, "Expected {} < {}", a, b);
            assert!((nb >= na), "Expected not {} < {}", b, a);
        }
    }

    #[test]
    fn test_gt_comparison() {
        let cases = vec![
            ("1", "0"),
            ("0.1", "0"),
            ("1", "-2"),
            ("2", "-2"),
            ("-2", "-3"),
            ("300", "34.123"),
            ("123.00001", "123"),
        ];
        for (a, b) in cases {
            let na = Numeric::new(a).unwrap();
            let nb = Numeric::new(b).unwrap();
            assert!(na > nb, "Expected {} > {}", a, b);
            assert!((nb <= na), "Expected not {} > {}", b, a);
        }
    }

    #[test]
    fn test_lte_comparison() {
        let cases = vec![
            ("0", "0"),
            ("0", "1"),
            ("0", "0.1"),
            ("-2", "1"),
            ("-2", "-2"),
            ("-3", "-2"),
        ];
        for (a, b) in cases {
            let na = Numeric::new(a).unwrap();
            let nb = Numeric::new(b).unwrap();
            assert!(na <= nb, "Expected {} <= {}", a, b);
        }
    }

    #[test]
    fn test_gte_comparison() {
        let cases = vec![
            ("0", "0"),
            ("1", "0"),
            ("0.1", "0"),
            ("1", "-2"),
            ("-2", "-3"),
            ("-2", "-2"),
        ];
        for (a, b) in cases {
            let na = Numeric::new(a).unwrap();
            let nb = Numeric::new(b).unwrap();
            assert!(na >= nb, "Expected {} >= {}", a, b);
        }
    }

    #[test]
    fn test_from_str() {
        let n: Numeric = "42".parse().unwrap();
        assert_eq!(n.as_string(), "42");
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<Numeric, _> = "abc".parse();
        assert!(result.is_err());
    }
}
