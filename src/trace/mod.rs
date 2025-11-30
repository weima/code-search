pub mod call_extractor;
pub mod function_finder;
pub mod graph_builder; // Task 8.4 - not yet implemented

pub use call_extractor::{CallExtractor, CallerInfo};
pub use function_finder::{FunctionDef, FunctionFinder};
pub use graph_builder::{CallGraphBuilder, CallNode, CallTree, TraceDirection}; // Task 8.4
