use yaml_rust::scanner::Scanner;

fn main() {
    let s = "key: value";
    let mut scanner = Scanner::new(s.chars());
    while let Some(token) = scanner.next_token().unwrap() {
        println!("{:?}", token.0.line); // marker.line
        println!("{:?}", token.1); // token type
    }
}
