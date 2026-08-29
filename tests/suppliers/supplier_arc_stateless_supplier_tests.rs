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
mod test_arc_stateless_supplier {
    use super::Arc;
    use super::ArcSupplier;
    use super::Supplier;
    use super::thread;

    mod test_new {
        use super::ArcSupplier;
        use super::Supplier;

        #[test]
        fn test_new_basic() {
            // Test creating a new ArcSupplier
            let supplier = ArcSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_new_with_closure() {
            // Test with a closure that captures variables
            let value = 100;
            let supplier = ArcSupplier::new(move || value);
            assert_eq!(supplier.get(), 100);
        }

        #[test]
        fn test_new_returns_same_value() {
            // Test that successive calls return same value
            let supplier = ArcSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
        }
    }

    mod test_constant {
        use super::ArcSupplier;
        use super::Supplier;

        #[test]
        fn test_constant_basic() {
            // Test constant supplier
            let constant = ArcSupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_constant_string() {
            // Test constant with String type
            let constant = ArcSupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }
    }

    mod test_map {
        use super::ArcSupplier;
        use super::Supplier;

        #[test]
        fn test_map_basic() {
            // Test map transformation
            let source = ArcSupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_map_chain() {
            // Test chained map operations
            let source = ArcSupplier::new(|| 10);
            let pipeline = source.map(|x| x * 2).map(|x| x + 5);
            assert_eq!(pipeline.get(), 25);
        }

        #[test]
        fn test_map_preserves_original() {
            // Test that mapping doesn't consume original
            let source = ArcSupplier::new(|| 10);
            let _mapped = source.map(|x| x * 2);
            // source is still usable
            assert_eq!(source.get(), 10);
        }
    }

    mod test_filter {
        use super::ArcSupplier;
        use super::Supplier;

        #[test]
        fn test_filter_passes() {
            // Test filter that passes
            let source = ArcSupplier::new(|| 42);
            let filtered = source.filter(|x: &i32| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let source = ArcSupplier::new(|| 43);
            let filtered = source.filter(|x: &i32| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }
    }

    mod test_zip {
        use super::ArcSupplier;
        use super::Supplier;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = ArcSupplier::new(|| 42);
            let second = ArcSupplier::new(|| "hello");
            let zipped = first.zip(second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_preserves_originals() {
            // Test that zip doesn't consume originals
            let first = ArcSupplier::new(|| 42);
            let second = ArcSupplier::new(|| "hello");
            let _zipped = first.zip(second.clone());
            // Both are still usable
            assert_eq!(first.get(), 42);
            assert_eq!(second.get(), "hello");
        }
    }

    mod test_clone {
        use super::ArcSupplier;
        use super::Supplier;

        #[test]
        fn test_clone_basic() {
            // Test cloning supplier
            let original = ArcSupplier::new(|| 42);
            let cloned = original.clone();
            assert_eq!(original.get(), 42);
            assert_eq!(cloned.get(), 42);
        }

        #[test]
        fn test_clone_shares_function() {
            // Test that clone shares the underlying function
            let original = ArcSupplier::new(|| String::from("hello"));
            let cloned = original.clone();
            assert_eq!(original.get(), cloned.get());
        }
    }

    mod test_thread_safety {
        use super::Arc;
        use super::ArcSupplier;
        use super::Supplier;
        use super::thread;

        #[test]
        fn test_send_between_threads() {
            // Test that supplier can be sent between threads
            let supplier = ArcSupplier::new(|| 42);
            let handle = thread::spawn(move || supplier.get());
            assert_eq!(handle.join().expect("thread should not panic"), 42);
        }

        #[test]
        fn test_concurrent_access() {
            // Test lock-free concurrent access
            let factory = ArcSupplier::new(|| String::from("Hello, World!"));

            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let f = factory.clone();
                    thread::spawn(move || f.get())
                })
                .collect();

            for h in handles {
                assert_eq!(h.join().expect("thread should not panic"), "Hello, World!");
            }
        }

        #[test]
        fn test_shared_across_threads() {
            // Test sharing supplier across multiple threads
            let supplier = Arc::new(ArcSupplier::new(|| 100));

            let handles: Vec<_> = (0..5)
                .map(|_| {
                    let s = Arc::clone(&supplier);
                    thread::spawn(move || s.get())
                })
                .collect();

            for h in handles {
                assert_eq!(h.join().expect("thread should not panic"), 100);
            }
        }
    }

    mod test_trait_methods {
        use super::ArcSupplier;
        use super::Supplier;

        #[test]
        fn test_get() {
            // Test Supplier::get method
            let supplier = ArcSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }
    }
}

// ======================================================================
// RcSupplier Tests
// ======================================================================
