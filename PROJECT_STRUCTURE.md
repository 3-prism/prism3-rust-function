# Project Structure

This document describes the directory structure and organization of the
prism3-rust-function crate.

## Directory Layout

```
prism3-rust-function/
├── src/                    # Source code
│   ├── consumers/          # Consumer-related abstractions
│   ├── predicates/         # Predicate-related abstractions
│   ├── transformers/       # Transformer-related abstractions
│   ├── functions/          # Function-related abstractions
│   ├── suppliers/          # Supplier-related abstractions
│   ├── mutators/           # Mutator-related abstractions
│   ├── comparator.rs       # Comparator abstraction (standalone)
│   ├── tester.rs           # Tester abstraction (standalone)
│   └── lib.rs              # Library root and re-exports
├── tests/                  # Integration tests
│   ├── consumers/          # Consumer tests
│   ├── predicates/         # Predicate tests
│   ├── transformers/       # Transformer tests
│   ├── functions/          # Function tests
│   ├── suppliers/          # Supplier tests
│   ├── mutators/           # Mutator tests
│   ├── comparator_tests.rs # Comparator tests (standalone)
│   ├── tester_tests.rs     # Tester tests (standalone)
│   └── mod.rs              # Test module root
├── examples/               # Example programs
│   ├── consumers/          # Consumer examples
│   ├── predicates/         # Predicate examples
│   ├── transformers/       # Transformer examples
│   ├── suppliers/          # Supplier examples
│   └── mutators/           # Mutator examples
└── doc/                    # Design documentation
```

## Module Organization

### Consumers Module (`src/consumers/`)

Contains all consumer-related functional abstractions:

- `consumer.rs` - Immutable consumer (`Fn(&T)`)
- `consumer_once.rs` - One-time consumer (`FnOnce(&T)`)
- `stateful_consumer.rs` - Stateful consumer (`FnMut(&T)`)
- `bi_consumer.rs` - Two-parameter consumer (`Fn(&T, &U)`)
- `bi_consumer_once.rs` - One-time two-parameter consumer
  (`FnOnce(&T, &U)`)
- `stateful_bi_consumer.rs` - Stateful two-parameter consumer
  (`FnMut(&T, &U)`)
- `mod.rs` - Module exports

### Predicates Module (`src/predicates/`)

Contains predicate-related functional abstractions:

- `predicate.rs` - Single-parameter predicate (`Fn(&T) -> bool`)
- `bi_predicate.rs` - Two-parameter predicate (`Fn(&T, &U) -> bool`)
- `mod.rs` - Module exports

### Transformers Module (`src/transformers/`)

Contains transformer-related functional abstractions:

- `transformer.rs` - Value transformer (`Fn(T) -> R`)
- `transformer_once.rs` - One-time transformer (`FnOnce(T) -> R`)
- `stateful_transformer.rs` - Stateful transformer (`FnMut(T) -> R`)
- `bi_transformer.rs` - Two-parameter transformer (`Fn(T, U) -> R`)
- `bi_transformer_once.rs` - One-time two-parameter transformer
  (`FnOnce(T, U) -> R`)
- `stateful_bi_transformer.rs` - Stateful two-parameter transformer
  (`FnMut(T, U) -> R`)
- `mod.rs` - Module exports

### Functions Module (`src/functions/`)

Contains function-related abstractions (reference-based transformations):

- `function.rs` - Reference function (`Fn(&T) -> R`)
- `function_once.rs` - One-time reference function (`FnOnce(&T) -> R`)
- `stateful_function.rs` - Stateful reference function
  (`FnMut(&T) -> R`)
- `mod.rs` - Module exports

### Suppliers Module (`src/suppliers/`)

Contains supplier-related abstractions (value generators):

- `supplier.rs` - Immutable supplier (`Fn() -> T`)
- `supplier_once.rs` - One-time supplier (`FnOnce() -> T`)
- `stateful_supplier.rs` - Stateful supplier (`FnMut() -> T`)
- `mod.rs` - Module exports

### Mutators Module (`src/mutators/`)

Contains mutator-related abstractions (in-place modifications):

- `mutator.rs` - Stateful mutator (`FnMut(&mut T)`)
- `mutator_once.rs` - One-time mutator (`FnOnce(&mut T)`)
- `stateful_mutator.rs` - Stateful mutator with additional state
- `mod.rs` - Module exports

### Standalone Modules

- `comparator.rs` - Comparator abstraction (`Fn(&T, &T) -> Ordering`)
- `tester.rs` - Tester abstraction (`Fn() -> bool`)

## Test Organization

The `tests/` directory mirrors the `src/` directory structure:

- Each module has a corresponding test directory
- Test files are named `{module}_tests.rs`
- Standalone modules have their tests in the root of `tests/`

## Example Organization

The `examples/` directory is organized by functionality:

- Each module has a corresponding examples directory
- Example files demonstrate typical usage patterns
- Examples are named descriptively (e.g., `consumer_demo.rs`,
  `predicate_demo.rs`)

## Import Paths

### Internal Imports (within the crate)

Modules reference each other using the new structure:

```rust
use crate::consumers::consumer::Consumer;
use crate::predicates::predicate::Predicate;
use crate::transformers::transformer::Transformer;
```

### External Imports (for users)

Users can import types in two ways:

1. **Direct from root** (recommended, backward compatible):
   ```rust
   use prism3_function::{Consumer, Predicate, Transformer};
   ```

2. **From specific modules** (explicit):
   ```rust
   use prism3_function::consumers::Consumer;
   use prism3_function::predicates::Predicate;
   use prism3_function::transformers::Transformer;
   ```

Both styles are supported through re-exports in `lib.rs`.

## Design Rationale

### Why Plural Directory Names?

Using plural names (e.g., `consumers`, `predicates`) avoids path
ambiguity:

- ✅ `src/consumers/consumer.rs` → `crate::consumers::consumer`
- ❌ `src/consumer/consumer.rs` → `crate::consumer::consumer` (confusing)

### Why Standalone Files?

`comparator.rs` and `tester.rs` remain at the root because:

1. They are single-file modules with no variants
2. They serve as utility abstractions
3. Keeping them flat reduces unnecessary nesting

### Module Grouping Logic

Modules are grouped by:

1. **Primary functionality** (consumers, predicates, transformers, etc.)
2. **Parameter count** (single vs. bi-parameter variants in same module)
3. **State management** (stateless, once, stateful variants in same
   module)

This organization makes it easy to find related abstractions and
understand the relationships between different variants.

## Migration Notes

If you're updating code that references old paths:

| Old Path | New Path |
|----------|----------|
| `crate::consumer` | `crate::consumers::consumer` |
| `crate::predicate` | `crate::predicates::predicate` |
| `crate::transformer` | `crate::transformers::transformer` |
| `crate::function` | `crate::functions::function` |
| `crate::supplier` | `crate::suppliers::supplier` |
| `crate::mutator` | `crate::mutators::mutator` |

Public API imports remain unchanged due to re-exports in `lib.rs`.

## Author

Haixing Hu <starfish.hu@gmail.com>

