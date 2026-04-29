//! Path utilities for TypeSpec-Rust
//!
//! Ported from TypeSpec compiler/src/core/path-utils.ts
//!
//! This module provides path manipulation utilities for cross-platform
//! path handling with support for URLs and DOS-style paths.

use std::borrow::Cow;

use crate::charcode::CharCode;

/// Directory separator for internal path representation
pub const DIRECTORY_SEPARATOR: char = '/';
/// Alternative directory separator (Windows)
pub const ALT_DIRECTORY_SEPARATOR: char = '\\';

/// URL scheme separator
const URL_SCHEME_SEPARATOR: &str = "://";

/// Check if a character code corresponds to a directory separator (`/` or `\`)
pub fn is_any_directory_separator(char_code: u32) -> bool {
    char_code == DIRECTORY_SEPARATOR as u32 || char_code == ALT_DIRECTORY_SEPARATOR as u32
}

/// Check if a path starts with a URL scheme (e.g. `http://`, `ftp://`, `file://`)
pub fn is_url(path: &str) -> bool {
    get_encoded_root_length(path) < 0
}

/// Check if a path starts with an absolute path component
pub fn is_path_absolute(path: &str) -> bool {
    get_encoded_root_length(path) != 0
}

/// Check if a character is a volume character (a-z, A-Z)
fn is_volume_character(char_code: u32) -> bool {
    (char_code >= CharCode::a as u32 && char_code <= CharCode::z as u32)
        || (char_code >= CharCode::A as u32 && char_code <= CharCode::Z as u32)
}

/// Get file URL volume separator end position
fn get_file_url_volume_separator_end(url: &str, start: usize) -> isize {
    let bytes = url.as_bytes();
    if start >= bytes.len() {
        return -1;
    }

    let ch0 = bytes[start] as u32;
    if ch0 == CharCode::Colon as u32 {
        return (start + 1) as isize;
    }
    if ch0 == CharCode::Percent as u32
        && start + 2 < bytes.len()
        && bytes[start + 1] as u32 == CharCode::_3 as u32
    {
        let ch2 = bytes[start + 2] as u32;
        if ch2 == CharCode::a as u32 || ch2 == CharCode::A as u32 {
            return (start + 3) as isize;
        }
    }
    -1
}

/// Returns length of the root part of a path or URL.
/// Returns a negative value for URLs (two's complement of root length).
fn get_encoded_root_length(path: &str) -> isize {
    if path.is_empty() {
        return 0;
    }

    let bytes = path.as_bytes();
    let ch0 = bytes[0] as u32;

    // POSIX or UNC
    if ch0 == CharCode::Slash as u32 || ch0 == CharCode::Backslash as u32 {
        if bytes.len() < 2 || bytes[1] as u32 != ch0 {
            return 1; // POSIX: "/" (or non-normalized "\")
        }

        let sep = if ch0 == CharCode::Slash as u32 {
            DIRECTORY_SEPARATOR
        } else {
            ALT_DIRECTORY_SEPARATOR
        };

        let Some(p1) = path[2..].find(sep) else {
            return path.len() as isize; // UNC: "//server" or "\\server"
        };

        return (p1 + 3) as isize; // UNC: "//server/" or "\\server\"
    }

    // DOS
    if is_volume_character(ch0) && bytes.len() >= 2 && bytes[1] as u32 == CharCode::Colon as u32 {
        if bytes.len() >= 3 {
            let ch2 = bytes[2] as u32;
            if ch2 == CharCode::Slash as u32 || ch2 == CharCode::Backslash as u32 {
                return 3; // DOS: "c:/" or "c:\"
            }
        }
        if path.len() == 2 {
            return 2; // DOS: "c:"
        }
    }

    // URL
    if let Some(scheme_end) = path.find(URL_SCHEME_SEPARATOR) {
        let authority_start = scheme_end + URL_SCHEME_SEPARATOR.len();
        if let Some(authority_end) = path[authority_start..].find(DIRECTORY_SEPARATOR) {
            let authority_end = authority_start + authority_end;
            let scheme = &path[..scheme_end];
            let authority = &path[authority_start..authority_end];

            if (scheme == "file")
                && (authority.is_empty() || authority == "localhost")
                && authority_end + 1 < bytes.len()
                && is_volume_character(bytes[authority_end + 1] as u32)
            {
                let volume_separator_end =
                    get_file_url_volume_separator_end(path, authority_end + 2);
                if volume_separator_end != -1 {
                    let vsep = volume_separator_end as usize;
                    if vsep == bytes.len() {
                        // URL: "file:///c:", "file://localhost/c:", etc.
                        return !(vsep as isize);
                    }
                    if path.as_bytes()[vsep] as u32 == CharCode::Slash as u32 {
                        // URL: "file:///c:/", "file://localhost/c:/", etc.
                        return !((vsep + 1) as isize);
                    }
                }
            }
            return !((authority_end + 1) as isize); // URL: "file://server/", "http://server/"
        }
        return !(path.len() as isize); // URL: "file://server", "http://server"
    }

    // relative
    0
}

/// Returns length of the root part of a path or URL
pub fn get_root_length(path: &str) -> usize {
    let root_length = get_encoded_root_length(path);
    if root_length < 0 {
        return (!root_length) as usize;
    }
    root_length as usize
}

/// Normalize path separators, converting `\` into `/`.
/// Returns a borrow when no change is needed, avoiding allocation on POSIX.
pub fn normalize_slashes<'a>(path: &'a str) -> Cow<'a, str> {
    if !path.contains('\\') {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(path.replace('\\', "/"))
    }
}

/// Check if path has a trailing directory separator
pub fn has_trailing_directory_separator(path: &str) -> bool {
    !path.is_empty() && is_any_directory_separator(path.as_bytes()[path.len() - 1] as u32)
}

/// Remove trailing directory separator from path
pub fn remove_trailing_directory_separator(path: &str) -> String {
    if has_trailing_directory_separator(path) {
        path[..path.len() - 1].to_string()
    } else {
        path.to_string()
    }
}

/// Ensure path has a trailing directory separator
pub fn ensure_trailing_directory_separator(path: &str) -> String {
    if !has_trailing_directory_separator(path) {
        format!("{}{}", path, DIRECTORY_SEPARATOR)
    } else {
        path.to_string()
    }
}

/// Returns the directory path (path without the final component)
pub fn get_directory_path(path: &str) -> String {
    let path = normalize_slashes(path);
    let root_length = get_root_length(&path);

    // If the path provided is itself the root, return it
    if root_length == path.len() {
        return path.into_owned();
    }

    let path = remove_trailing_directory_separator(&path);
    let last_sep = path.rfind(DIRECTORY_SEPARATOR);

    if let Some(sep_pos) = last_sep
        && sep_pos >= root_length
    {
        return path[..sep_pos].to_string();
    }

    // Fallback to root
    if root_length > 0 {
        path[..root_length].to_string()
    } else {
        String::new()
    }
}

/// Returns the base file name (final component of path)
pub fn get_base_file_name(path: &str) -> String {
    let path = normalize_slashes(path);
    let root_length = get_root_length(&path);

    // If the path provided is itself the root, return empty
    if root_length == path.len() {
        return String::new();
    }

    let path = remove_trailing_directory_separator(&path);
    let last_sep = path.rfind(DIRECTORY_SEPARATOR);

    if let Some(sep_pos) = last_sep
        && sep_pos >= root_length
    {
        return path[sep_pos + 1..].to_string();
    }

    // No separator found, return the whole path
    path[root_length..].to_string()
}

/// Gets the file extension for a path (lowercased, with dot)
pub fn get_any_extension_from_path(path: &str) -> String {
    let base_file_name = get_base_file_name(path);
    if let Some(dot_pos) = base_file_name.rfind('.') {
        base_file_name[dot_pos..].to_lowercase()
    } else {
        String::new()
    }
}

/// Split path into components
fn path_components(path: &str, root_length: usize) -> Vec<String> {
    let root = path[..root_length].to_string();
    let rest = &path[root_length..];

    if rest.is_empty() {
        return vec![root];
    }

    let mut components = vec![root];
    for part in rest.split('/') {
        if !part.is_empty() {
            components.push(part.to_string());
        }
    }

    components
}

/// Get path components
pub fn get_path_components(path: &str, current_directory: Option<&str>) -> Vec<String> {
    let path = if let Some(cur) = current_directory {
        join_paths(cur, path)
    } else {
        path.to_string()
    };

    let normalized = normalize_slashes(&path);
    path_components(&normalized, get_root_length(&normalized))
}

/// Reduce path components by resolving . and .. entries
pub fn reduce_path_components(components: &[String]) -> Vec<String> {
    if components.is_empty() {
        return vec![];
    }

    let mut reduced: Vec<String> = vec![components[0].clone()];

    for component in components.iter().skip(1) {
        if component.is_empty() {
            continue;
        }
        if component == "." {
            continue;
        }
        if component == ".." {
            // For ".." component in TypeScript:
            // - If reduced.len() > 1 AND last != "..", pop and continue
            // - Else if reduced[0] is truthy (non-empty root), continue (don't push)
            // - Otherwise push ".."
            if reduced.len() > 1 && reduced[reduced.len() - 1] != ".." {
                reduced.pop();
                continue;
            } else if !reduced[0].is_empty() {
                continue;
            }
            reduced.push(component.clone());
        } else {
            reduced.push(component.clone());
        }
    }

    reduced
}

/// Join paths together
pub fn join_paths(base: &str, path: &str) -> String {
    let mut result = base.to_string();

    if !result.is_empty() {
        result = normalize_slashes(&result).into_owned();
    }

    let path = normalize_slashes(path);
    let root_length = get_root_length(&path);

    if root_length != 0 {
        // path is absolute, replace result
        result = path.to_string();
    } else if path.is_empty() {
        // path is empty, do nothing (TypeSpec behavior)
    } else if !result.is_empty() {
        result = ensure_trailing_directory_separator(&result) + &path;
    } else {
        result = path.to_string();
    }

    result
}

/// Resolve paths (join and normalize)
pub fn resolve_path(path: &str, paths: &[&str]) -> String {
    if paths.is_empty() {
        return normalize_path(path);
    }

    let mut all_paths = vec![path];
    all_paths.extend_from_slice(paths);

    let mut result = all_paths[0].to_string();
    for p in &all_paths[1..] {
        if !p.is_empty() {
            result = join_paths(&result, p);
        }
    }

    normalize_path(&result)
}

/// Normalize a path (resolve . and .., remove extra slashes)
pub fn normalize_path(path: &str) -> String {
    let path = normalize_slashes(path);

    // Check if path needs normalization
    // These patterns indicate path segments that need resolution:
    // - /./  (current directory marker)
    // - /../ (parent directory marker)
    // - //    (double slash - UNC or just extra)
    // - . at start (current directory marker)
    // - ending with . or .. (current or parent directory)
    let needs_normalization = path.contains("/./")
        || path.starts_with("./")
        || path.contains("/../")
        || path.contains("//")
        || path == "."
        || path == ".."
        || path.ends_with("/.")
        || path.ends_with("/..");

    if !needs_normalization {
        return path.into_owned();
    }

    // Some paths only require cleanup of /./ or leading ./
    let simplified = path
        .replace("/./", "/")
        .trim_start_matches("./")
        .to_string();
    if simplified != path && !simplified.contains("..") && !simplified.contains("//") {
        return simplified;
    }

    let components = get_path_components(&path, None);
    let reduced = reduce_path_components(&components);
    let result = get_path_from_path_components(&reduced);

    // Preserve trailing separator if original had it
    if has_trailing_directory_separator(&path) && !result.is_empty() {
        ensure_trailing_directory_separator(&result)
    } else {
        result
    }
}

/// Format path components back into a path string
pub fn get_path_from_path_components(components: &[String]) -> String {
    if components.is_empty() {
        return String::new();
    }

    let root = if components[0].is_empty() {
        String::new()
    } else {
        ensure_trailing_directory_separator(&components[0])
    };

    let rest = components[1..].join(&DIRECTORY_SEPARATOR.to_string());

    if root.is_empty() {
        rest
    } else {
        format!("{}{}", root, rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_any_directory_separator() {
        assert!(is_any_directory_separator('/' as u32));
        assert!(is_any_directory_separator('\\' as u32));
        assert!(!is_any_directory_separator('a' as u32));
    }

    #[test]
    fn test_normalize_slashes() {
        assert_eq!(normalize_slashes("a"), "a");
        assert_eq!(normalize_slashes("a/b"), "a/b");
        assert_eq!(normalize_slashes("a\\b"), "a/b");
        assert_eq!(normalize_slashes("\\\\server\\path"), "//server/path");
    }

    #[test]
    fn test_has_trailing_directory_separator() {
        assert!(has_trailing_directory_separator("/path/to/"));
        assert!(has_trailing_directory_separator("/path/to\\"));
        assert!(!has_trailing_directory_separator("/path/to"));
    }

    #[test]
    fn test_remove_trailing_directory_separator() {
        assert_eq!(remove_trailing_directory_separator("/path/to/"), "/path/to");
        assert_eq!(remove_trailing_directory_separator("/path/to"), "/path/to");
    }

    #[test]
    fn test_ensure_trailing_directory_separator() {
        assert_eq!(ensure_trailing_directory_separator("/path/to"), "/path/to/");
        assert_eq!(
            ensure_trailing_directory_separator("/path/to/"),
            "/path/to/"
        );
    }

    #[test]
    fn test_get_root_length() {
        assert_eq!(get_root_length("a"), 0);
        assert_eq!(get_root_length("/"), 1);
        assert_eq!(get_root_length("/path"), 1);
        // DOS paths
        assert_eq!(get_root_length("c:"), 2);
        assert_eq!(get_root_length("c:d"), 0);
        assert_eq!(get_root_length("c:/"), 3);
        assert_eq!(get_root_length("c:\\"), 3);
        // UNC paths
        assert_eq!(get_root_length("//server"), 8);
        assert_eq!(get_root_length("//server/share"), 9);
        assert_eq!(get_root_length("\\\\server"), 8);
        assert_eq!(get_root_length("\\\\server\\share"), 9);
        // File URLs
        assert_eq!(get_root_length("file:///"), 8);
        assert_eq!(get_root_length("file:///path"), 8);
        assert_eq!(get_root_length("file:///c:"), 10);
        assert_eq!(get_root_length("file:///c:d"), 8);
        assert_eq!(get_root_length("file:///c:/path"), 11);
        assert_eq!(get_root_length("file:///c%3a"), 12);
        assert_eq!(get_root_length("file:///c%3ad"), 8);
        assert_eq!(get_root_length("file:///c%3a/path"), 13);
        assert_eq!(get_root_length("file:///c%3A"), 12);
        assert_eq!(get_root_length("file:///c%3Ad"), 8);
        assert_eq!(get_root_length("file:///c%3A/path"), 13);
        // File URLs with localhost
        assert_eq!(get_root_length("file://localhost"), 16);
        assert_eq!(get_root_length("file://localhost/"), 17);
        assert_eq!(get_root_length("file://localhost/path"), 17);
        assert_eq!(get_root_length("file://localhost/c:"), 19);
        assert_eq!(get_root_length("file://localhost/c:d"), 17);
        assert_eq!(get_root_length("file://localhost/c:/path"), 20);
        assert_eq!(get_root_length("file://localhost/c%3a"), 21);
        assert_eq!(get_root_length("file://localhost/c%3ad"), 17);
        assert_eq!(get_root_length("file://localhost/c%3a/path"), 22);
        assert_eq!(get_root_length("file://localhost/c%3A"), 21);
        assert_eq!(get_root_length("file://localhost/c%3Ad"), 17);
        assert_eq!(get_root_length("file://localhost/c%3A/path"), 22);
        // File URLs with server
        assert_eq!(get_root_length("file://server"), 13);
        assert_eq!(get_root_length("file://server/"), 14);
        assert_eq!(get_root_length("file://server/path"), 14);
        assert_eq!(get_root_length("file://server/c:"), 14);
        assert_eq!(get_root_length("file://server/c:d"), 14);
        assert_eq!(get_root_length("file://server/c:/d"), 14);
        assert_eq!(get_root_length("file://server/c%3a"), 14);
        assert_eq!(get_root_length("file://server/c%3ad"), 14);
        assert_eq!(get_root_length("file://server/c%3a/d"), 14);
        assert_eq!(get_root_length("file://server/c%3A"), 14);
        assert_eq!(get_root_length("file://server/c%3Ad"), 14);
        assert_eq!(get_root_length("file://server/c%3A/d"), 14);
        // HTTP URLs
        assert_eq!(get_root_length("http://server"), 13);
        assert_eq!(get_root_length("http://server/path"), 14);
    }

    #[test]
    fn test_is_url() {
        // NOT url
        assert!(!is_url("a"));
        assert!(!is_url("/"));
        assert!(!is_url("c:"));
        assert!(!is_url("c:d"));
        assert!(!is_url("c:/"));
        assert!(!is_url("c:\\"));
        assert!(!is_url("//server"));
        assert!(!is_url("//server/share"));
        assert!(!is_url("\\\\server"));
        assert!(!is_url("\\\\server\\share"));

        // Is Url
        assert!(is_url("file:///path"));
        assert!(is_url("file:///c:"));
        assert!(is_url("file:///c:d"));
        assert!(is_url("file:///c:/path"));
        assert!(is_url("file://server"));
        assert!(is_url("file://server/path"));
        assert!(is_url("http://server"));
        assert!(is_url("http://server/path"));
    }

    #[test]
    fn test_is_path_absolute() {
        assert!(is_path_absolute("/path/to/file"));
        assert!(!is_path_absolute("path/to/file"));
        assert!(!is_path_absolute("./path/to/file"));
    }

    #[test]
    fn test_get_base_file_name() {
        assert_eq!(get_base_file_name(""), "");
        assert_eq!(get_base_file_name("a"), "a");
        assert_eq!(get_base_file_name("a/"), "a");
        assert_eq!(get_base_file_name("/"), "");
        assert_eq!(get_base_file_name("/a"), "a");
        assert_eq!(get_base_file_name("/a/"), "a");
        assert_eq!(get_base_file_name("/a/b"), "b");
        // DOS paths
        assert_eq!(get_base_file_name("c:"), "");
        assert_eq!(get_base_file_name("c:d"), "c:d");
        assert_eq!(get_base_file_name("c:/"), "");
        assert_eq!(get_base_file_name("c:\\"), "");
        assert_eq!(get_base_file_name("c:/path"), "path");
        assert_eq!(get_base_file_name("c:/path/"), "path");
        // UNC paths
        assert_eq!(get_base_file_name("//server"), "");
        assert_eq!(get_base_file_name("//server/"), "");
        assert_eq!(get_base_file_name("//server/share"), "share");
        assert_eq!(get_base_file_name("//server/share/"), "share");
        // File URLs
        assert_eq!(get_base_file_name("file:///"), "");
        assert_eq!(get_base_file_name("file:///path"), "path");
        assert_eq!(get_base_file_name("file:///path/"), "path");
        assert_eq!(get_base_file_name("file:///c:"), "");
        assert_eq!(get_base_file_name("file:///c:/"), "");
        assert_eq!(get_base_file_name("file:///c:d"), "c:d");
        assert_eq!(get_base_file_name("file:///c:/d"), "d");
        assert_eq!(get_base_file_name("file:///c:/d/"), "d");
        // HTTP URLs
        assert_eq!(get_base_file_name("http://server"), "");
        assert_eq!(get_base_file_name("http://server/"), "");
        assert_eq!(get_base_file_name("http://server/a"), "a");
        assert_eq!(get_base_file_name("http://server/a/"), "a");
    }

    #[test]
    fn test_get_any_extension_from_path() {
        assert_eq!(get_any_extension_from_path(""), "");
        assert_eq!(get_any_extension_from_path(".ext"), ".ext");
        assert_eq!(get_any_extension_from_path("a.ext"), ".ext");
        assert_eq!(get_any_extension_from_path("/a.ext"), ".ext");
        assert_eq!(get_any_extension_from_path("a.ext/"), ".ext");
        assert_eq!(get_any_extension_from_path(".EXT"), ".ext");
        assert_eq!(get_any_extension_from_path("a.EXT"), ".ext");
        assert_eq!(get_any_extension_from_path("/a.EXT"), ".ext");
        assert_eq!(get_any_extension_from_path("a.EXT/"), ".ext");
    }

    #[test]
    fn test_get_directory_path() {
        assert_eq!(get_directory_path(""), "");
        assert_eq!(get_directory_path("a"), "");
        assert_eq!(get_directory_path("a/b"), "a");
        assert_eq!(get_directory_path("/"), "/");
        assert_eq!(get_directory_path("/a"), "/");
        assert_eq!(get_directory_path("/a/"), "/");
        assert_eq!(get_directory_path("/a/b"), "/a");
        assert_eq!(get_directory_path("/a/b/"), "/a");
        // DOS paths
        assert_eq!(get_directory_path("c:"), "c:");
        assert_eq!(get_directory_path("c:d"), "");
        assert_eq!(get_directory_path("c:/"), "c:/");
        assert_eq!(get_directory_path("c:/path"), "c:/");
        assert_eq!(get_directory_path("c:/path/"), "c:/");
        // UNC paths
        assert_eq!(get_directory_path("//server"), "//server");
        assert_eq!(get_directory_path("//server/"), "//server/");
        assert_eq!(get_directory_path("//server/share"), "//server/");
        assert_eq!(get_directory_path("//server/share/"), "//server/");
        assert_eq!(get_directory_path("\\\\server"), "//server");
        assert_eq!(get_directory_path("\\\\server\\"), "//server/");
        assert_eq!(get_directory_path("\\\\server\\share"), "//server/");
        assert_eq!(get_directory_path("\\\\server\\share\\"), "//server/");
        // File URLs
        assert_eq!(get_directory_path("file:///"), "file:///");
        assert_eq!(get_directory_path("file:///path"), "file:///");
        assert_eq!(get_directory_path("file:///path/"), "file:///");
        assert_eq!(get_directory_path("file:///c:"), "file:///c:");
        assert_eq!(get_directory_path("file:///c:d"), "file:///");
        assert_eq!(get_directory_path("file:///c:/"), "file:///c:/");
        assert_eq!(get_directory_path("file:///c:/path"), "file:///c:/");
        assert_eq!(get_directory_path("file:///c:/path/"), "file:///c:/");
        assert_eq!(get_directory_path("file://server"), "file://server");
        assert_eq!(get_directory_path("file://server/"), "file://server/");
        assert_eq!(get_directory_path("file://server/path"), "file://server/");
        assert_eq!(get_directory_path("file://server/path/"), "file://server/");
        // HTTP URLs
        assert_eq!(get_directory_path("http://server"), "http://server");
        assert_eq!(get_directory_path("http://server/"), "http://server/");
        assert_eq!(get_directory_path("http://server/path"), "http://server/");
        assert_eq!(get_directory_path("http://server/path/"), "http://server/");
    }

    #[test]
    fn test_join_paths() {
        assert_eq!(
            join_paths("/", "/node_modules/@types"),
            "/node_modules/@types"
        );
        assert_eq!(join_paths("/a/..", ""), "/a/..");
        assert_eq!(join_paths("/a/..", "b"), "/a/../b");
        assert_eq!(join_paths("/a/..", "b/"), "/a/../b/");
        assert_eq!(join_paths("/a/..", "/"), "/");
        assert_eq!(join_paths("/a/..", "/b"), "/b");
        // Basic cases
        assert_eq!(join_paths("path", "to"), "path/to");
        assert_eq!(join_paths("/path", "to"), "/path/to");
        assert_eq!(join_paths("path", "/to"), "/to");
        assert_eq!(join_paths("path", ""), "path");
    }

    #[test]
    fn test_get_path_components() {
        assert_eq!(get_path_components("", None), vec![""]);
        assert_eq!(get_path_components("a", None), vec!["", "a"]);
        assert_eq!(get_path_components("./a", None), vec!["", ".", "a"]);
        assert_eq!(get_path_components("/", None), vec!["/"]);
        assert_eq!(get_path_components("/a", None), vec!["/", "a"]);
        assert_eq!(get_path_components("/a/", None), vec!["/", "a"]);
        // DOS paths
        assert_eq!(get_path_components("c:", None), vec!["c:"]);
        assert_eq!(get_path_components("c:d", None), vec!["", "c:d"]);
        assert_eq!(get_path_components("c:/", None), vec!["c:/"]);
        assert_eq!(get_path_components("c:/path", None), vec!["c:/", "path"]);
        // UNC paths
        assert_eq!(get_path_components("//server", None), vec!["//server"]);
        assert_eq!(get_path_components("//server/", None), vec!["//server/"]);
        assert_eq!(
            get_path_components("//server/share", None),
            vec!["//server/", "share"]
        );
        // File URLs
        assert_eq!(get_path_components("file:///", None), vec!["file:///"]);
        assert_eq!(
            get_path_components("file:///path", None),
            vec!["file:///", "path"]
        );
        assert_eq!(get_path_components("file:///c:", None), vec!["file:///c:"]);
        assert_eq!(
            get_path_components("file:///c:d", None),
            vec!["file:///", "c:d"]
        );
        assert_eq!(
            get_path_components("file:///c:/", None),
            vec!["file:///c:/"]
        );
        assert_eq!(
            get_path_components("file:///c:/path", None),
            vec!["file:///c:/", "path"]
        );
        assert_eq!(
            get_path_components("file://server", None),
            vec!["file://server"]
        );
        assert_eq!(
            get_path_components("file://server/", None),
            vec!["file://server/"]
        );
        assert_eq!(
            get_path_components("file://server/path", None),
            vec!["file://server/", "path"]
        );
        // HTTP URLs
        assert_eq!(
            get_path_components("http://server", None),
            vec!["http://server"]
        );
        assert_eq!(
            get_path_components("http://server/", None),
            vec!["http://server/"]
        );
        assert_eq!(
            get_path_components("http://server/path", None),
            vec!["http://server/", "path"]
        );
    }

    #[test]
    fn test_reduce_path_components() {
        assert_eq!(reduce_path_components(&[]), Vec::<String>::new());
        assert_eq!(
            reduce_path_components(&["".to_string()]),
            vec!["".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["".to_string(), ".".to_string()]),
            vec!["".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["".to_string(), ".".to_string(), "a".to_string()]),
            vec!["".to_string(), "a".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["".to_string(), "a".to_string(), ".".to_string()]),
            vec!["".to_string(), "a".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["".to_string(), "..".to_string()]),
            vec!["".to_string(), "..".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["".to_string(), "..".to_string(), "..".to_string()]),
            vec!["".to_string(), "..".to_string(), "..".to_string()]
        );
        assert_eq!(
            reduce_path_components(&[
                "".to_string(),
                "..".to_string(),
                ".".to_string(),
                "..".to_string()
            ]),
            vec!["".to_string(), "..".to_string(), "..".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["".to_string(), "a".to_string(), "..".to_string()]),
            vec!["".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["".to_string(), "..".to_string(), "a".to_string()]),
            vec!["".to_string(), "..".to_string(), "a".to_string()]
        );
        // Root paths
        assert_eq!(
            reduce_path_components(&["/".to_string()]),
            vec!["/".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["/".to_string(), ".".to_string()]),
            vec!["/".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["/".to_string(), "..".to_string()]),
            vec!["/".to_string()]
        );
        assert_eq!(
            reduce_path_components(&["/".to_string(), "a".to_string(), "..".to_string()]),
            vec!["/".to_string()]
        );
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/path/to/./file"), "/path/to/file");
        assert_eq!(normalize_path("/path/to/../file"), "/path/file");
        assert_eq!(normalize_path("./path/to"), "path/to");
        assert_eq!(normalize_path("/path/to//file"), "/path/to/file");
    }

    #[test]
    fn test_resolve_path() {
        assert_eq!(resolve_path("", &[]), "");
        assert_eq!(resolve_path(".", &[]), "");
        assert_eq!(resolve_path("./", &[]), "");
        assert_eq!(resolve_path("..", &[]), "..");
        assert_eq!(resolve_path("../", &[]), "../");
        assert_eq!(resolve_path("/", &[]), "/");
        assert_eq!(resolve_path("/a", &[]), "/a");
        assert_eq!(resolve_path("/a/", &[]), "/a/");
        assert_eq!(resolve_path("/a", &["b"]), "/a/b");
        assert_eq!(resolve_path("/a", &["b", "c"]), "/a/b/c");
        assert_eq!(resolve_path("/a", &["b", "/c"]), "/c");
        assert_eq!(resolve_path("/a", &["b", "../c"]), "/a/c");
        assert_eq!(resolve_path("a", &["b", "c"]), "a/b/c");
    }

    #[test]
    fn test_get_path_from_path_components() {
        assert_eq!(
            get_path_from_path_components(&[
                "/".to_string(),
                "path".to_string(),
                "file.ext".to_string()
            ]),
            "/path/file.ext"
        );
        assert_eq!(
            get_path_from_path_components(&["/".to_string(), "path".to_string()]),
            "/path"
        );
        assert_eq!(
            get_path_from_path_components(&[
                "".to_string(),
                "path".to_string(),
                "file.ext".to_string()
            ]),
            "path/file.ext"
        );
    }
}
