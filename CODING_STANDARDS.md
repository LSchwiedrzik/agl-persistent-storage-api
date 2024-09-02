# Coding Standards

To maintain consistency and readability in the codebase, all contributors 
should adhere to the following coding standards when working on this Rust project.

## Table of Contents

1. [General Principles](#general-principles)
2. [Code Formatting](#code-formatting)
3. [Naming Conventions](#naming-conventions)
4. [Comments and Documentation](#comments-and-documentation)
5. [Testing](#testing)
6. [Version Control](#version-control)
7. [Tools](#tools)

## General Principles

- Write clean, readable, and maintainable code.
- Follow Rust's philosophy of safety, speed, and concurrency.
- Ensure code is DRY (Don't Repeat Yourself) and follows the principle of single responsibility.
- Prioritize readability and simplicity over cleverness.
- Strive for consistency across the codebase.

## Code Formatting

- Use `rustfmt` to automatically format your code.
- Ensure code adheres to the Rust community style guidelines.
- Limit lines to 100 characters where possible.
- Place opening braces on the same line as the control statement.
- Use spaces around operators and after commas for better readability.

Example:
```rust
// Good
if a == b {
    do_something();
}

// Bad
if(a==b){
    do_something();
}
```

## Naming Conventions

- **Variables and Functions**: Use `snake_case` for variables, function names, and module names.
- **Structs and Enums**: Use `CamelCase` for struct and enum names.
- **Constants**: Use `UPPER_SNAKE_CASE` for constant values.
- **Lifetimes**: Use short, descriptive names like `_a` or `_b` for lifetimes.

Example:
```rust
// Good
struct Circle {
    radius: f64,
}

fn calculate_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius.powi(2)
}

// Bad
struct circle {
    Radius: f64,
}

fn CalculateArea(Radius: f64) -> f64 {
    std::f64::consts::PI * Radius.powi(2)
}
```

## Comments and Documentation

- Write clear and concise comments explaining the why behind complex logic.
- Use Rustdoc comments (`///`) to document public functions, structs, and modules.
- Avoid obvious comments; focus on explaining non-trivial code.
- Keep comments up-to-date with code changes.

Example:
```rust
/// Calculates the area of a circle given its radius.
///
/// # Arguments
///
/// * `radius` - A floating-point number representing the radius of the circle.
///
/// # Returns
///
/// A floating-point number representing the area of the circle.
fn calculate_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius.powi(2)
}
```

## Testing

- Write unit tests for all new features and bug fixes.
- Follow the Arrange-Act-Assert (AAA) pattern in tests.
- Ensure all tests pass before submitting a pull request.
- Use `cargo test` to run tests.

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_area() {
        let radius = 2.0;
        let area = calculate_area(radius);
        assert_eq!(area, std::f64::consts::PI * 4.0);
    }
}
```

## Version Control

- Commit messages should be concise but descriptive, following the format: `Type: Short description`.
- Group related changes into a single commit.
- Reference issue numbers in commit messages where relevant.
- Use branches for all work, and merge to `master` through pull requests.

Example:
```text
fix: Corrected off-by-one error in loop
feat: Added user authentication module
```

## Tools

- Use the following tools to ensure code quality:
    - **`rustfmt`**: For consistent code formatting.
    - **`cargo test`**: For running tests and ensuring code correctness.

- Configure your IDE/editor to run these tools automatically before committing code.

## Acknowledgments

Thank you for adhering to these coding standards! Consistency and quality in the codebase help make the project maintainable and welcoming for all contributors.

