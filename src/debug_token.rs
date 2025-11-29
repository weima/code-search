use yaml_rust::scanner::TokenType;
use yaml_rust::scanner::Marker;

fn main() {
    let t: Option<TokenType> = None;
    if let Some(token_type) = t {
        match token_type {
            TokenType::Scalar => {}, // Try unit variant
            // TokenType::Scalar(..) => {}, // Try tuple variant
            _ => {}
        }
    }
}
