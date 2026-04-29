//! Numeric ranges for TypeSpec standard scalar types
//! Ported from TypeSpec compiler/src/core/numeric-ranges.ts

use crate::numeric::Numeric;
use std::sync::OnceLock;

/// A numeric range with min, max, and constraints
#[derive(Debug, Clone)]
pub struct NumericRange {
    pub min: Numeric,
    pub max: Numeric,
    pub is_integer: bool,
    pub is_js_number: bool,
}

static NUMERIC_RANGES: OnceLock<Vec<(&'static str, NumericRange)>> = OnceLock::new();

/// Get the known numeric ranges for standard TypeSpec scalar types (cached)
pub fn get_numeric_ranges() -> &'static [(&'static str, NumericRange)] {
    NUMERIC_RANGES
        .get_or_init(|| {
            vec![
                (
                    "int64",
                    NumericRange {
                        min: Numeric::new("-9223372036854775808").expect("valid numeric constant"),
                        max: Numeric::new("9223372036854775807").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: false,
                    },
                ),
                (
                    "int32",
                    NumericRange {
                        min: Numeric::new("-2147483648").expect("valid numeric constant"),
                        max: Numeric::new("2147483647").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: true,
                    },
                ),
                (
                    "int16",
                    NumericRange {
                        min: Numeric::new("-32768").expect("valid numeric constant"),
                        max: Numeric::new("32767").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: true,
                    },
                ),
                (
                    "int8",
                    NumericRange {
                        min: Numeric::new("-128").expect("valid numeric constant"),
                        max: Numeric::new("127").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: true,
                    },
                ),
                (
                    "uint64",
                    NumericRange {
                        min: Numeric::new("0").expect("valid numeric constant"),
                        max: Numeric::new("18446744073709551615").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: false,
                    },
                ),
                (
                    "uint32",
                    NumericRange {
                        min: Numeric::new("0").expect("valid numeric constant"),
                        max: Numeric::new("4294967295").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: true,
                    },
                ),
                (
                    "uint16",
                    NumericRange {
                        min: Numeric::new("0").expect("valid numeric constant"),
                        max: Numeric::new("65535").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: true,
                    },
                ),
                (
                    "uint8",
                    NumericRange {
                        min: Numeric::new("0").expect("valid numeric constant"),
                        max: Numeric::new("255").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: true,
                    },
                ),
                (
                    "safeint",
                    NumericRange {
                        min: Numeric::new("-9007199254740991").expect("valid numeric constant"),
                        max: Numeric::new("9007199254740991").expect("valid numeric constant"),
                        is_integer: true,
                        is_js_number: true,
                    },
                ),
                (
                    "float32",
                    NumericRange {
                        min: Numeric::new("-3.4e38").expect("valid numeric constant"),
                        max: Numeric::new("3.4e38").expect("valid numeric constant"),
                        is_integer: false,
                        is_js_number: true,
                    },
                ),
                (
                    "float64",
                    NumericRange {
                        min: Numeric::new("-1.7976931348623157e308")
                            .expect("valid numeric constant"),
                        max: Numeric::new("1.7976931348623157e308")
                            .expect("valid numeric constant"),
                        is_integer: false,
                        is_js_number: true,
                    },
                ),
            ]
        })
        .as_slice()
}

/// Check if a numeric value fits within a named scalar type's range
pub fn is_value_in_range(value: &Numeric, scalar_name: &str) -> bool {
    for (name, range) in get_numeric_ranges() {
        if *name == scalar_name {
            return value >= &range.min && value <= &range.max;
        }
    }
    // Unknown scalar type - can't check range
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int32_range() {
        let range = get_numeric_ranges()
            .iter()
            .find(|(n, _)| *n == "int32")
            .map(|(_, r)| r)
            .expect("valid numeric constant");
        assert!(range.is_integer);

        let val = Numeric::new("100").expect("valid numeric constant");
        assert!(val >= range.min && val <= range.max);

        let overflow = Numeric::new("3000000000").expect("valid numeric constant");
        assert!(overflow > range.max);
    }

    #[test]
    fn test_uint8_range() {
        let range = get_numeric_ranges()
            .iter()
            .find(|(n, _)| *n == "uint8")
            .map(|(_, r)| r)
            .expect("valid numeric constant");
        assert!(range.is_integer);
        assert!(Numeric::new("0").expect("valid numeric constant") >= range.min);
        assert!(Numeric::new("255").expect("valid numeric constant") <= range.max);
        assert!(Numeric::new("256").expect("valid numeric constant") > range.max);
    }

    #[test]
    fn test_is_value_in_range() {
        assert!(is_value_in_range(
            &Numeric::new("100").expect("valid numeric constant"),
            "int32"
        ));
        assert!(!is_value_in_range(
            &Numeric::new("3000000000").expect("valid numeric constant"),
            "int32"
        ));
        assert!(is_value_in_range(
            &Numeric::new("200").expect("valid numeric constant"),
            "uint8"
        ));
        assert!(!is_value_in_range(
            &Numeric::new("300").expect("valid numeric constant"),
            "uint8"
        ));
    }

    #[test]
    fn test_all_ranges_present() {
        let ranges = get_numeric_ranges();
        let names: Vec<&str> = ranges.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"int8"));
        assert!(names.contains(&"int16"));
        assert!(names.contains(&"int32"));
        assert!(names.contains(&"int64"));
        assert!(names.contains(&"uint8"));
        assert!(names.contains(&"uint16"));
        assert!(names.contains(&"uint32"));
        assert!(names.contains(&"uint64"));
        assert!(names.contains(&"safeint"));
        assert!(names.contains(&"float32"));
        assert!(names.contains(&"float64"));
    }
}
