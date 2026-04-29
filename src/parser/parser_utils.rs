//! Parser utility functions
//!
//! Ported from TypeSpec compiler/src/core/parser-utils.ts
//!
//! Provides helper functions for working with parsed comments and positions.

use crate::charcode::is_whitespace;

/// A comment range in source text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// Start position of the comment (inclusive)
    pub pos: usize,
    /// End position of the comment (exclusive)
    pub end: usize,
}

/// Find the comment that is at the given position, if any.
///
/// A comment is at the given position if `comment.pos <= position < comment.end`.
/// Unlike node-at-position, the end position is not included since comments
/// can be adjacent to each other with no trivia or punctuation between them.
///
/// Comments must be ordered by increasing position for binary search to work.
///
/// Ported from TS getCommentAtPosition()
pub fn get_comment_at_position(comments: &[Comment], pos: usize) -> Option<&Comment> {
    let mut low = 0isize;
    let mut high = comments.len() as isize - 1;

    while low <= high {
        let middle = low + ((high - low) >> 1);
        let candidate = &comments[middle as usize];
        if pos >= candidate.end {
            low = middle + 1;
        } else if pos < candidate.pos {
            high = middle - 1;
        } else {
            return Some(candidate);
        }
    }
    None
}

/// Adjust the given position backwards before any trivia (whitespace or comments).
///
/// Returns the position of the last non-trivia character before the given position.
///
/// Ported from TS getPositionBeforeTrivia()
pub fn get_position_before_trivia(text: &str, comments: &[Comment], pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut pos = pos;

    while pos > 0 {
        if is_whitespace(bytes[pos - 1] as u32) {
            while pos > 0 && is_whitespace(bytes[pos - 1] as u32) {
                pos -= 1;
            }
        } else if let Some(comment) = get_comment_at_position(comments, pos - 1) {
            pos = comment.pos;
        } else {
            // Not at whitespace or comment
            break;
        }
    }

    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_comments(positions: &[(usize, usize)]) -> Vec<Comment> {
        positions
            .iter()
            .map(|(pos, end)| Comment {
                pos: *pos,
                end: *end,
            })
            .collect()
    }

    // ==================== getCommentAtPosition ====================

    #[test]
    fn test_get_comment_at_position_finds_comment() {
        let comments = make_comments(&[(0, 20), (21, 40), (41, 70)]);
        let result = get_comment_at_position(&comments, 50);
        assert!(result.is_some());
        assert_eq!(result.unwrap().pos, 41);
        assert_eq!(result.unwrap().end, 70);
    }

    #[test]
    fn test_get_comment_at_position_first_comment() {
        let comments = make_comments(&[(5, 25), (30, 50)]);
        let result = get_comment_at_position(&comments, 10);
        assert!(result.is_some());
        assert_eq!(result.unwrap().pos, 5);
    }

    #[test]
    fn test_get_comment_at_position_not_found_between() {
        let comments = make_comments(&[(0, 20), (30, 50)]);
        // Position 25 is between the two comments
        let result = get_comment_at_position(&comments, 25);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_comment_at_position_adjacent_comments() {
        // Adjacent: first ends at 20, second starts at 20
        let comments = make_comments(&[(0, 20), (20, 40)]);
        // At the boundary pos=20, the first comment is NOT included (end is exclusive)
        // so we should find the second comment
        let result = get_comment_at_position(&comments, 20);
        assert!(result.is_some());
        assert_eq!(result.unwrap().pos, 20);
    }

    #[test]
    fn test_get_comment_at_position_empty() {
        let comments: Vec<Comment> = vec![];
        let result = get_comment_at_position(&comments, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_comment_at_position_at_start() {
        let comments = make_comments(&[(10, 30)]);
        let result = get_comment_at_position(&comments, 10);
        assert!(result.is_some());
        assert_eq!(result.unwrap().pos, 10);
    }

    #[test]
    fn test_get_comment_at_position_at_end_excluded() {
        let comments = make_comments(&[(10, 30)]);
        // end is exclusive, so position 30 should not match
        let result = get_comment_at_position(&comments, 30);
        assert!(result.is_none());
    }

    // ==================== getPositionBeforeTrivia ====================

    #[test]
    fn test_get_position_before_trivia_no_trivia() {
        let text = "model Test {}";
        let comments = vec![];
        let pos = text.len();
        let result = get_position_before_trivia(text, &comments, pos);
        assert_eq!(result, text.len());
    }

    #[test]
    fn test_get_position_before_trivia_whitespace() {
        let base = "model Test {}";
        let text = format!("{} ", base);
        let comments = vec![];
        let pos = text.len();
        let result = get_position_before_trivia(&text, &comments, pos);
        assert_eq!(result, base.len()); // position after '}'
    }

    #[test]
    fn test_get_position_before_trivia_trailing_comment() {
        let base = "model Test {}";
        let text = format!("{} /* Test */", base);
        let comment_start = base.len() + 1; // position of '/'
        let comment_end = text.len();
        let comments = make_comments(&[(comment_start, comment_end)]);
        let pos = text.len();
        let result = get_position_before_trivia(&text, &comments, pos);
        assert_eq!(result, base.len()); // position right after '}'
    }

    #[test]
    fn test_get_position_before_trivia_multiple_comments() {
        let base = "model Test {}";
        // Build text with comments after the base, compute positions from actual string
        let text = format!("{} /* c1 */ // c2\n  /* c3 */", base);
        let offset = base.len();

        // Find actual comment positions by searching for "/*" and "//" patterns
        let c1_start = text[offset..].find("/*").unwrap() + offset;
        let c1_end = text[c1_start..].find("*/").unwrap() + c1_start + 2;
        let c2_start = text[c1_end..].find("//").unwrap() + c1_end;
        let c2_end = text[c2_start..].find('\n').unwrap() + c2_start;
        let c3_start = text[c2_end..].find("/*").unwrap() + c2_end;
        let c3_end = text[c3_start..].find("*/").unwrap() + c3_start + 2;

        let comments = make_comments(&[(c1_start, c1_end), (c2_start, c2_end), (c3_start, c3_end)]);
        let pos = text.len();
        let result = get_position_before_trivia(&text, &comments, pos);
        assert_eq!(result, base.len()); // position right after '}'
    }

    #[test]
    fn test_get_position_before_trivia_at_zero() {
        let text = "hello";
        let comments = vec![];
        let result = get_position_before_trivia(text, &comments, 0);
        assert_eq!(result, 0);
    }
}
