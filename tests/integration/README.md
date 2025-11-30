# Integration Test Suite

This directory contains comprehensive integration tests for the Code Search CLI tool, organized by user story.

## Test Organization

### `basic_search.rs` - US-1: Basic Text-to-Code Trace (11 tests)
Tests the core functionality of tracing UI text through translation files to implementation code.

**Coverage:**
- ✅ Complete search chain display (text → translation → code)
- ✅ Case-insensitive search (default)
- ✅ Case-sensitive search (--case-sensitive flag)
- ✅ YAML translation file detection
- ✅ Full key path extraction (dot notation)
- ✅ Code reference detection with line numbers
- ✅ Performance verification (< 5 seconds)
- ✅ No matches handling (success exit code)
- ✅ Special characters in search text
- ✅ React project support
- ✅ Vue project support

### `multi_match.rs` - US-3: Multiple Match Handling (11 tests)
Tests the tool's ability to find and display all locations where translation keys are used.

**Coverage:**
- ✅ Multiple code references displayed
- ✅ Line numbers for each usage
- ✅ Multiple translation files (multi-language)
- ✅ Partial key matching (namespace caching pattern)
- ✅ Multiple i18n patterns detected
- ✅ Grouped display of related usages
- ✅ Multiple keys with same value
- ✅ Deeply nested keys (up to 10 levels)
- ✅ Cross-file references
- ✅ Duplicate result prevention
- ✅ Multiple frameworks in one project

### `error_cases.rs` - US-5: Error Handling & Edge Cases (18 tests)
Tests error handling, helpful guidance, and edge case scenarios.

**Error Handling:**
- ✅ No translation files found
- ✅ Empty search text rejected
- ✅ Whitespace-only search rejected
- ✅ Malformed YAML shows clear error
- ✅ Malformed YAML suggests next steps
- ✅ Malformed YAML mentions common issues

**Edge Cases:**
- ✅ Empty YAML file
- ✅ YAML with only comments
- ✅ YAML with null values
- ✅ Very long translation keys (deeply nested)
- ✅ Special YAML characters in values
- ✅ Unicode in translations
- ✅ YAML with array values
- ✅ Very large YAML files (1000+ keys)
- ✅ Mixed YAML and JSON files
- ✅ Nested directory structures
- ✅ Symlinks (Unix-only)

## Running Tests

### Run all integration tests:
```bash
cargo test --test integration_*
```

### Run specific test suite:
```bash
cargo test --test integration_basic_search
cargo test --test integration_multi_match
cargo test --test integration_error_cases
```

### Run a specific test:
```bash
cargo test --test integration_basic_search test_basic_search_shows_complete_chain
```

### Run with output:
```bash
cargo test --test integration_basic_search -- --nocapture
```

## Test Fixtures

All integration tests use fixtures located in `tests/fixtures/`:
- `rails-app/` - Rails application with i18n (primary test fixture)
- `react-app/` - React application with react-i18next
- `vue-app/` - Vue application with vue-i18n
- `code-examples/` - Code examples for call tracing

See `tests/fixtures/README.md` for detailed documentation of test data.

## Coverage Verification

### Using cargo-tarpaulin (recommended):
```bash
# Install (if not already installed)
cargo install cargo-tarpaulin

# Run coverage for integration tests
cargo tarpaulin --test integration_basic_search --test integration_multi_match --test integration_error_cases

# Run coverage for all tests
cargo tarpaulin --all

# Generate HTML report
cargo tarpaulin --out Html --all
```

### Expected Coverage
- **Target**: ≥ 80% code coverage
- **Current Status**: 234 passing tests across all test suites
- **Integration Tests**: 40 tests covering all major user stories

## Test Maintenance

### Adding New Tests

1. **Choose the appropriate file:**
   - `basic_search.rs` - Core search functionality
   - `multi_match.rs` - Multiple matches and complex scenarios
   - `error_cases.rs` - Error handling and edge cases

2. **Follow naming convention:**
   ```rust
   #[test]
   fn test_<scenario>_<expected_behavior>() {
       // Given
       // When
       // Then
   }
   ```

3. **Use descriptive comments:**
   ```rust
   // Given a project with i18n translation files
   // When I run `cs "search text"`
   // Then I see the complete reference chain
   ```

4. **Verify the test fails first** (TDD):
   ```bash
   cargo test --test integration_basic_search test_new_test
   # Should fail initially
   ```

### Test Fixtures

When adding new test fixtures:
1. Add files to appropriate `tests/fixtures/` subdirectory
2. Document in `tests/fixtures/README.md`
3. Verify manually with `rg` or `fd`
4. Keep fixtures minimal but realistic

### Updating Tests

When modifying functionality:
1. Update tests FIRST (TDD)
2. Run tests to verify they fail: `cargo test`
3. Implement the change
4. Verify tests pass: `cargo test`
5. Update this README if test structure changes

## Continuous Integration

These integration tests are designed to run in CI/CD pipelines:
- Fast execution (< 5 seconds total)
- No external dependencies beyond ripgrep
- Cross-platform compatible (Windows, macOS, Linux)
- Deterministic results (no flaky tests)

## Troubleshooting

### Test Failures

1. **"ripgrep not found"**: Install ripgrep (`brew install ripgrep`)
2. **"No such file or directory"**: Ensure you're running from project root
3. **Temp directory errors**: Check disk space and permissions
4. **Timing failures**: System under heavy load, tests use generous timeouts

### Debugging Tests

```bash
# Run with detailed output
RUST_BACKTRACE=1 cargo test --test integration_basic_search -- --nocapture

# Run a single test with logging
RUST_LOG=debug cargo test --test integration_basic_search test_specific_test -- --nocapture
```

## Contributing

When contributing new integration tests:
1. Follow existing patterns and structure
2. Add tests for both happy path and error cases
3. Use temporary directories for file creation
4. Clean up resources (handled by TempDir automatically)
5. Document the test purpose in comments
6. Ensure tests are deterministic and fast

## References

- **User Stories**: `.specify/specs/code-search/spec.md`
- **Test Fixtures**: `tests/fixtures/README.md`
- **Main Test Suite**: `tests/*.rs`
- **Acceptance Criteria**: `.specify/specs/code-search/spec.md`
