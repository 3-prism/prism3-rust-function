// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for StatefulSupplier types

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use qubit_function::ArcStatefulSupplier;
use qubit_function::BoxStatefulSupplier;
use qubit_function::RcStatefulSupplier;
use qubit_function::StatefulSupplier;

// ==========================================================================
// StatefulSupplier Trait Tests (for closures)
// ==========================================================================

#[cfg(test)]
mod test_box_stateful_supplier {
    use super::BoxStatefulSupplier;
    use super::StatefulSupplier;

    mod test_new {
        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_creates_stateful_supplier() {
            let mut supplier = BoxStatefulSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_with_i32() {
            let mut supplier = BoxStatefulSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let mut supplier = BoxStatefulSupplier::new(|| String::from("hello"));
            assert_eq!(supplier.get(), "hello");
        }

        #[test]
        fn test_with_vec() {
            let mut supplier = BoxStatefulSupplier::new(|| vec![1, 2, 3]);
            assert_eq!(supplier.get(), vec![1, 2, 3]);
        }

        #[test]
        fn test_with_bool() {
            let mut supplier = BoxStatefulSupplier::new(|| true);
            assert!(supplier.get());
        }
    }

    mod test_constant {
        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_returns_same_value() {
            let mut constant = BoxStatefulSupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let mut constant = BoxStatefulSupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }
    }

    mod test_get {
        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_can_be_called_multiple_times() {
            let mut supplier = BoxStatefulSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_stateful_counter() {
            let mut counter = 0;
            let mut supplier = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            });

            assert_eq!(supplier.get(), 1);
            assert_eq!(supplier.get(), 2);
            assert_eq!(supplier.get(), 3);
        }
    }

    mod test_map {
        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_transforms_value() {
            let mut mapped = BoxStatefulSupplier::new(|| 10).map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_multiple_chains() {
            let mut chained = BoxStatefulSupplier::new(|| 5).map(|x| x * 2).map(|x| x + 5);
            assert_eq!(chained.get(), 15);
        }

        #[test]
        fn test_type_conversion() {
            let mut converted = BoxStatefulSupplier::new(|| 42).map(|x: i32| x.to_string());
            assert_eq!(converted.get(), "42");
        }

        #[test]
        fn test_with_stateful_stateful_supplier() {
            let mut counter = 0;
            let mut mapped = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            })
            .map(|x| x * 10);

            assert_eq!(mapped.get(), 10);
            assert_eq!(mapped.get(), 20);
            assert_eq!(mapped.get(), 30);
        }

        // Test with function pointer
        #[test]
        fn test_with_function_pointer() {
            fn double(x: i32) -> i32 {
                x * 2
            }
            let mut mapped = BoxStatefulSupplier::new(|| 10).map(double);
            assert_eq!(mapped.get(), 20);
        }
    }

    mod test_filter {
        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_filters_even_numbers() {
            let mut counter = 0;
            let mut filtered = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            })
            .filter(|x: &i32| x % 2 == 0);

            assert_eq!(filtered.get(), None); // 1 is odd
            assert_eq!(filtered.get(), Some(2)); // 2 is even
            assert_eq!(filtered.get(), None); // 3 is odd
            assert_eq!(filtered.get(), Some(4)); // 4 is even
        }

        #[test]
        fn test_with_constant_stateful_supplier() {
            let mut filtered = BoxStatefulSupplier::constant(5).filter(|x: &i32| x % 2 == 0);
            assert_eq!(filtered.get(), None); // 5 is odd
            assert_eq!(filtered.get(), None);
        }
    }

    mod test_zip {
        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_combines_two_stateful_suppliers() {
            let first = BoxStatefulSupplier::new(|| 42);
            let second = BoxStatefulSupplier::new(|| "hello");
            let mut zipped = first.zip(second);

            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_with_stateful_stateful_suppliers() {
            let mut counter1 = 0;
            let first = BoxStatefulSupplier::new(move || {
                counter1 += 1;
                counter1
            });
            let mut counter2 = 0;
            let second = BoxStatefulSupplier::new(move || {
                counter2 += 10;
                counter2
            });
            let mut zipped = first.zip(second);

            assert_eq!(zipped.get(), (1, 10));
            assert_eq!(zipped.get(), (2, 20));
        }
    }

    mod test_memoize {
        use std::cell::Cell;
        use std::rc::Rc;

        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_caches_first_value() {
            // Use a shared counter to verify memoization
            let call_count = Rc::new(Cell::new(0));
            let call_count_capture = Rc::clone(&call_count);
            let mut memoized = BoxStatefulSupplier::new(move || {
                call_count_capture.set(call_count_capture.get() + 1);
                42
            })
            .memoize();

            assert_eq!(memoized.get(), 42);
            assert_eq!(memoized.get(), 42);
            assert_eq!(memoized.get(), 42);
            assert_eq!(call_count.get(), 1);
        }

        #[test]
        fn test_with_stateful_stateful_supplier() {
            let mut counter = 0;
            let mut memoized = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            })
            .memoize();

            assert_eq!(memoized.get(), 1); // First call
            assert_eq!(memoized.get(), 1); // Cached
            assert_eq!(memoized.get(), 1); // Cached
        }
    }
}

// ==========================================================================
// ArcStatefulSupplier Tests
// ==========================================================================
