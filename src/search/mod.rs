pub mod text_search;
pub mod pattern_match;

pub use text_search::{Match, TextSearcher};
pub use pattern_match::{CodeReference, PatternMatcher};
