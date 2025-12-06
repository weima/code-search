use cs::trace::FunctionFinder;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sitter_c_sharp_function() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    let file_path = base_dir.join("Test.cs");
    fs::write(
        &file_path,
        r#"
public class Test {
    public void MyFunction() {
        Console.WriteLine("Hello");
    }
    
    // public void FakeFunction() {}
}
        "#,
    )
    .unwrap();

    let mut finder = FunctionFinder::new(base_dir);

    // Should find MyFunction
    let func = finder.find_function("MyFunction");
    assert!(func.is_some(), "Should find MyFunction in C#");
    let func = func.unwrap();
    assert_eq!(func.name, "MyFunction");

    // Should not find FakeFunction (commented)
    let fake = finder.find_function("FakeFunction");
    assert!(fake.is_none(), "Should not find FakeFunction in comment");
}

#[test]
fn test_sitter_ruby_function() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    let file_path = base_dir.join("test.rb");
    fs::write(
        &file_path,
        r#"
class Test
  def my_method
    puts "Hello"
  end
  
  # def fake_method
  # end
end
        "#,
    )
    .unwrap();

    let mut finder = FunctionFinder::new(base_dir);

    // Should find my_method
    let func = finder.find_function("my_method");
    assert!(func.is_some(), "Should find my_method in Ruby");
    let func = func.unwrap();
    assert_eq!(func.name, "my_method");

    // Should not find fake_method
    let fake = finder.find_function("fake_method");
    assert!(fake.is_none(), "Should not find fake_method in comment");
}

#[test]
fn test_sitter_erb_ignoring() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    let file_path = base_dir.join("view.html.erb");
    fs::write(
        &file_path,
        r#"
<div>
  <% def should_not_be_found %>
  <% end %>
  <%= link_to "Home", root_path %>
</div>
        "#,
    )
    .unwrap();

    let mut finder = FunctionFinder::new(base_dir);

    // ERB query is empty/unsupported for definitions, so it should find nothing, gracefully.
    // Importantly, it should NOT panic.
    let func = finder.find_function("should_not_be_found");
    assert!(
        func.is_none(),
        "Should not find function in ERB (unsupported)"
    );
}
