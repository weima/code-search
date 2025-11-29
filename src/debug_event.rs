use yaml_rust::parser::Event;
use yaml_rust::scanner::Marker;

fn main() {
    let e = Event::StreamStart;
    match e {
        Event::Scalar(_, _, _, m) => {
            let _: Option<Marker> = m; // This should compile if m is Option<Marker>
        }
        _ => {}
    }
}
