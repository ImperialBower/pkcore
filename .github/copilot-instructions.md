# GitHub Copilot Instructions for pkcore

These instructions guide GitHub Copilot to generate code that aligns with our project standards for testing, documentation, and code quality.

## Testing Requirements

### Unit Tests
- **Every public function must have at least one unit test** covering the happy path
- **Every public struct/enum must have tests** that validate construction, methods, and trait implementations
- Unit tests should be placed in a `#[cfg(test)]` module at the end of the file or in a `tests/` directory
- Test names should be descriptive and follow the pattern: `<function_or_struct_name>_<scenario>`
- Include edge cases, error conditions, and boundary conditions

### Doc Tests
- **Every public function and method must include at least one doc test** demonstrating basic usage
- Doc tests should be included in the doc comment (`///`) using triple backticks with `rust` syntax highlighting
- Doc tests must compile and run successfully
- Doc tests should show the most common usage pattern
- If a function can fail, include a doc test that shows success cases

### Example Structure
```rust
/// Brief description of what this function does.
///
/// Longer explanation of behavior, parameters, and return values.
///
/// # Panics
/// Panics if the condition X is violated.
///
/// # Errors
/// Returns `Err` if the operation fails due to reason Y.
///
/// # Examples
/// ```
/// use pkcore::your_module::YourFunction;
/// let result = your_function(42);
/// assert_eq!(result, expected_value);
/// ```
pub fn your_function(param: Type) -> Result<ReturnType, ErrorType> {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn your_function_happy_path() {
        let result = your_function(42);
        assert!(result.is_ok());
    }

    #[test]
    fn your_function_edge_case() {
        let result = your_function(0);
        assert!(result.is_err());
    }
}
```

## Documentation Requirements

### For Functions
- **Brief summary**: First line should be a concise description
- **Detailed explanation**: Explain what the function does, why it exists, and when to use it
- **Parameters**: Document all parameters and their expected types/ranges
- **Return value**: Explain what is returned and under what conditions
- **Errors**: Document all possible error cases using `# Errors` section
- **Panics**: Document if the function can panic using `# Panics` section
- **Examples**: Include `# Examples` with working doc test code
- **Safety**: Use `# Safety` section if the function is `unsafe`

### For Structs and Enums
- **Brief description**: Explain the purpose of the type
- **Fields**: Document each field and its purpose
- **Examples**: Show common construction and usage patterns
- **Trait implementations**: Explain behavior of implemented traits if non-obvious

### For Modules
- **Module-level documentation**: Each module file should start with a doc comment explaining its purpose
- Example: `//! Handles poker hand ranking and comparison logic.`

### For Crate Root
- **Lib.rs documentation**: Include a comprehensive overview of the crate, main concepts, and links to key modules

## Code Quality Standards

### Naming Conventions
- Use clear, descriptive names for functions, variables, and types
- Use snake_case for functions and variables
- Use PascalCase for types, structs, enums, and traits
- Avoid single-letter variable names except for loop indices

### Error Handling
- Never use `unwrap()`, `expect()` or `panic!()` in library code.
- It's ok to use `unwrap()`, `expect()` or `panic!()` in tests and doc tests to simplify test code, but only when the failure case would indicate a bug in the test itself rather than a recoverable error.
- Document error types clearly
- Create custom error types for domain-specific errors that implement `std::error::Error`
- Use `Result<T, E>` for fallible functions and document all error cases

### Trait Implementations
- Implement `Display` for user-facing types
- Implement `Debug` for all public types
- Consider implementing `Default` for types that have a sensible default
- Document non-obvious trait behavior

## Checklist for Copilot-Generated Code

Before accepting Copilot suggestions:
- ✓ Does the function have doc comments with examples?
- ✓ Are there unit tests for the function?
- ✓ Are there doc tests in the doc comments?
- ✓ Are error cases handled and documented?
- ✓ Are edge cases covered in tests?
- ✓ Is the code readable and maintainable?
- ✓ Are all public APIs documented?
- ✓ Do tests run successfully with `cargo test`?
- ✓ Do doc tests pass with `cargo test --doc`?

## Running Tests

```bash
# Run all tests
cargo test

# Run doc tests only
cargo test --doc

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_function_name
```

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust by Example: Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)
- [Writing Unsafe Rust](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)

