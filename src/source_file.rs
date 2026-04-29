//! Source file utilities for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/source-file.ts
//!
//! This module provides source file creation, line position tracking,
//! and source file kind detection utilities.

/// Represents a source file with text and line position tracking
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// The source text
    pub text: String,
    /// The file path
    pub path: String,
    /// Cached line starts
    line_starts: Option<Vec<usize>>,
}

impl SourceFile {
    /// Create a new source file
    pub fn new(text: String, path: String) -> Self {
        Self {
            text,
            path,
            line_starts: None,
        }
    }

    /// Get the line starts (byte positions where each line begins)
    pub fn get_line_starts(&mut self) -> &[usize] {
        self.line_starts
            .get_or_insert_with(|| scan_line_starts(&self.text))
    }

    /// Get line and character position for a byte offset
    pub fn get_line_and_character_of_position(&mut self, position: usize) -> LineAndCharacter {
        let starts = self.get_line_starts();
        let mut line = binary_search(starts, position);

        // When binarySearch returns < 0 indicating that the value was not found, it
        // returns the bitwise complement of the index where the value would need to
        // be inserted to keep the array sorted. So flipping the bits back to this
        // positive index tells us what the line number would be if we were to
        // create a new line starting at the given position, and subtracting 1 from
        // that therefore gives us the line number we're after.
        if line < 0 {
            line = !line - 1;
        }

        let character = position - starts[line as usize];
        LineAndCharacter {
            line: line as u32,
            character,
        }
    }
}

/// Represents a line and character position
#[derive(Debug, Clone, Copy)]
pub struct LineAndCharacter {
    /// The line number (0-indexed)
    pub line: u32,
    /// The character offset within the line
    pub character: usize,
}

/// Source file kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFileKind {
    /// JavaScript source file
    Js,
    /// TypeSpec source file
    TypeSpec,
}

impl SourceFileKind {
    /// Get the source file kind from a file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            ".js" | ".mjs" => Some(SourceFileKind::Js),
            ".tsp" => Some(SourceFileKind::TypeSpec),
            _ => None,
        }
    }
}

/// Get the source file kind from a file path
pub fn get_source_file_kind_from_path(path: &str) -> Option<SourceFileKind> {
    let ext = get_extension_from_path(path)?;
    SourceFileKind::from_extension(&ext)
}

/// Get extension from a path (including the dot)
fn get_extension_from_path(path: &str) -> Option<String> {
    path.rsplit_once('.').map(|(_, ext)| format!(".{}", ext))
}

/// Scan text for line start positions
fn scan_line_starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut start = 0;
    let mut pos = 0;

    let bytes = text.as_bytes();
    while pos < bytes.len() {
        let ch = bytes[pos];
        pos += 1;

        match ch {
            0x0d => {
                // CarriageReturn
                if pos < bytes.len() && bytes[pos] == 0x0a {
                    pos += 1;
                }
                starts.push(start);
                start = pos;
            }
            0x0a => {
                // LineFeed
                starts.push(start);
                start = pos;
            }
            _ => {}
        }
    }

    starts.push(start);
    starts
}

/// Search sorted array of numbers for the given value.
/// If found, return index in array where value was found.
/// If not found, return a negative number that is the bitwise complement
/// of the index where value would need to be inserted to keep the array sorted.
fn binary_search(array: &[usize], value: usize) -> isize {
    let mut low = 0isize;
    let mut high = array.len() as isize - 1;

    while low <= high {
        let middle = low + ((high - low) >> 1);
        let v = array[middle as usize];

        if v < value {
            low = middle + 1;
        } else if v > value {
            high = middle - 1;
        } else {
            return middle;
        }
    }

    !low
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_new() {
        let sf = SourceFile::new("model Foo {}".to_string(), "test.tsp".to_string());
        assert_eq!(sf.text, "model Foo {}");
        assert_eq!(sf.path, "test.tsp");
    }

    #[test]
    fn test_scan_line_starts_empty() {
        let starts = scan_line_starts("");
        assert_eq!(starts, vec![0]);
    }

    #[test]
    fn test_scan_line_starts_single_line() {
        let starts = scan_line_starts("hello");
        assert_eq!(starts, vec![0]);
    }

    #[test]
    fn test_scan_line_starts_multiple_lines() {
        let starts = scan_line_starts("line1\nline2\nline3");
        assert_eq!(starts, vec![0, 6, 12]);
    }

    #[test]
    fn test_scan_line_starts_crlf() {
        let starts = scan_line_starts("line1\r\nline2\r\nline3");
        assert_eq!(starts, vec![0, 7, 14]);
    }

    #[test]
    fn test_scan_line_starts_mixed() {
        let starts = scan_line_starts("line1\nline2\r\nline3\nline4");
        assert_eq!(starts, vec![0, 6, 13, 19]);
    }

    #[test]
    fn test_binary_search_found() {
        let arr = [1, 3, 5, 7, 9];
        assert_eq!(binary_search(&arr, 5), 2);
        assert_eq!(binary_search(&arr, 1), 0);
        assert_eq!(binary_search(&arr, 9), 4);
    }

    #[test]
    fn test_binary_search_not_found() {
        let arr = [1, 3, 5, 7, 9];
        // Not found, returns complement of insertion point
        // For 4: insertion point is 1, complement is !1 = -2
        let result = binary_search(&arr, 4);
        assert!(result < 0);
    }

    #[test]
    fn test_get_line_starts() {
        let mut sf = SourceFile::new("line1\nline2\nline3".to_string(), "test.tsp".to_string());
        let starts = sf.get_line_starts();
        assert_eq!(starts, vec![0, 6, 12]);
    }

    #[test]
    fn test_get_line_and_character() {
        let mut sf = SourceFile::new("line1\nline2\nline3".to_string(), "test.tsp".to_string());

        let pos = sf.get_line_and_character_of_position(0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);

        let pos = sf.get_line_and_character_of_position(6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);

        let pos = sf.get_line_and_character_of_position(12);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);

        let pos = sf.get_line_and_character_of_position(8);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 2);
    }

    #[test]
    fn test_source_file_kind_from_extension() {
        assert_eq!(
            SourceFileKind::from_extension(".js"),
            Some(SourceFileKind::Js)
        );
        assert_eq!(
            SourceFileKind::from_extension(".mjs"),
            Some(SourceFileKind::Js)
        );
        assert_eq!(
            SourceFileKind::from_extension(".tsp"),
            Some(SourceFileKind::TypeSpec)
        );
        assert_eq!(SourceFileKind::from_extension(".txt"), None);
    }

    #[test]
    fn test_get_source_file_kind_from_path() {
        assert_eq!(
            get_source_file_kind_from_path("file.js"),
            Some(SourceFileKind::Js)
        );
        assert_eq!(
            get_source_file_kind_from_path("file.mjs"),
            Some(SourceFileKind::Js)
        );
        assert_eq!(
            get_source_file_kind_from_path("file.tsp"),
            Some(SourceFileKind::TypeSpec)
        );
        assert_eq!(get_source_file_kind_from_path("file.ts"), None);
    }
}
