# Prism3 Function

Common functional programming type aliases for Rust, providing Java-style functional interfaces.

## Overview

This crate provides type aliases for common functional programming patterns, similar to Java's functional interfaces. These type aliases simplify type declarations in functional programming and provide better readability and maintainability.

## Features

- **Predicate<T>**: Represents a function that takes a reference to type `T` and returns a `bool`
- Thread-safe: All types implement `Send + Sync`
- Easy to use with closures and function pointers
- Compatible with standard library collections and iterators

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
prism3-function = "0.1.0"
```

## Usage

### Predicate

```rust
use prism3_function::Predicate;

// Create a predicate to check if a number is even
let is_even: Predicate<i32> = Box::new(|x| x % 2 == 0);

assert!(is_even(&4));
assert!(!is_even(&3));

// Use with collections
let numbers = vec![1, 2, 3, 4, 5, 6];
let even_numbers: Vec<i32> = numbers
    .into_iter()
    .filter(|x| is_even(x))
    .collect();

assert_eq!(even_numbers, vec![2, 4, 6]);
```

## Planned Types

- `Consumer<T>`: Consumes a value
- `Function<T, R>`: Transforms a value from type `T` to type `R`
- `Supplier<T>`: Supplies a value
- `Operator<T>`: Unary operator
- `BiOperator<T>`: Binary operator
- `Transformer<T>`: Transforms a value of the same type
- `Filter<T>`: Filters values

## License

Licensed under Apache License, Version 2.0.

## Author

Hu Haixing <starfish.hu@gmail.com>

