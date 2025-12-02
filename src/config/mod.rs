pub mod exclusions;
pub mod patterns;

pub use exclusions::{detect_project_type, get_default_exclusions, ProjectType};
pub use patterns::default_patterns;
