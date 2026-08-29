// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for Supplier types

use std::sync::Arc;
use std::thread;

use qubit_function::ArcSupplier;
use qubit_function::ArcTransformer;
use qubit_function::BoxSupplier;
use qubit_function::BoxTransformer;
use qubit_function::RcSupplier;
use qubit_function::RcTransformer;
use qubit_function::Supplier;

// ======================================================================
// Supplier Trait Tests (for closures)
// ======================================================================

#[cfg(test)]
mod test_box_stateless_supplier {
    use super::BoxSupplier;
    use super::Supplier;

    mod test_new {
        use super::BoxSupplier;
        use super::Supplier;

        #[test]
        fn test_new_basic() {
            // Test creating a new BoxSupplier
            let supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_new_with_closure() {
            // Test with a closure that captures variables
            let value = 100;
            let supplier = BoxSupplier::new(move || value);
            assert_eq!(supplier.get(), 100);
        }

        #[test]
        fn test_new_returns_same_value() {
            // Test that successive calls return same value
            let supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
        }
    }

    mod test_constant {
        use super::BoxSupplier;
        use super::Supplier;

        #[test]
        fn test_constant_basic() {
            // Test constant supplier
            let constant = BoxSupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_constant_string() {
            // Test constant with String type
            let constant = BoxSupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }

        #[test]
        fn test_constant_vec() {
            // Test constant with Vec type
            let constant = BoxSupplier::constant(vec![1, 2, 3]);
            assert_eq!(constant.get(), vec![1, 2, 3]);
            assert_eq!(constant.get(), vec![1, 2, 3]);
        }
    }

    mod test_map {
        use super::BoxSupplier;
        use super::Supplier;

        #[test]
        fn test_map_basic() {
            // Test map transformation
            let mapped = BoxSupplier::new(|| 10).map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_map_chain() {
            // Test chained map operations
            let pipeline = BoxSupplier::new(|| 10).map(|x| x * 2).map(|x| x + 5);
            assert_eq!(pipeline.get(), 25);
        }

        #[test]
        fn test_map_type_conversion() {
            // Test map with type conversion
            let mapped = BoxSupplier::new(|| 42).map(|x: i32| x.to_string());
            assert_eq!(mapped.get(), "42");
        }
    }

    mod test_filter {
        use super::BoxSupplier;
        use super::Supplier;

        #[test]
        fn test_filter_passes() {
            // Test filter that passes
            let filtered = BoxSupplier::new(|| 42).filter(|x: &i32| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let filtered = BoxSupplier::new(|| 43).filter(|x: &i32| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }

        #[test]
        fn test_filter_with_map() {
            // Test combining filter and map
            let pipeline = BoxSupplier::new(|| 10).map(|x| x * 2).filter(|x: &i32| *x > 15);
            assert_eq!(pipeline.get(), Some(20));
        }
    }

    mod test_zip {
        use super::BoxSupplier;
        use super::Supplier;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = BoxSupplier::new(|| 42);
            let second = BoxSupplier::new(|| "hello");
            let zipped = first.zip(second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_different_types() {
            // Test zipping suppliers of different types
            let first = BoxSupplier::new(|| 100);
            let second = BoxSupplier::new(|| vec![1, 2, 3]);
            let zipped = first.zip(second);
            assert_eq!(zipped.get(), (100, vec![1, 2, 3]));
        }
    }

    mod test_trait_methods {
        use super::BoxSupplier;
        use super::Supplier;

        #[test]
        fn test_get() {
            // Test Supplier::get method
            let supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        // Note: test_into_arc is not included here because
        // BoxSupplier cannot be converted to
        // ArcSupplier (inner function may not be Send +
        // Sync). This is enforced at compile time by trait bounds.
    }
}

// ======================================================================
// ArcSupplier Tests
// ======================================================================
