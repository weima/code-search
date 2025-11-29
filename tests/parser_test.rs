use cs::parse::{YamlParser, TranslationEntry};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_integration_parse_yaml() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "
section:
  subsection:
    key: value
").unwrap();

    let entries = YamlParser::parse_file(file.path()).unwrap();
    
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.key, "section.subsection.key");
    assert_eq!(entry.value, "value");
    assert_eq!(entry.line, 4); // Line numbers implemented!
}
