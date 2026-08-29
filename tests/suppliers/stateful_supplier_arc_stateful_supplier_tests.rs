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
mod test_arc_stateful_supplier {
    use super::Arc;
    use super::ArcStatefulSupplier;
    use super::Mutex;
    use super::StatefulSupplier;
    use super::thread;

    mod test_new {
        use super::ArcStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_creates_stateful_supplier() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_i32() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = ArcStatefulSupplier::new(|| String::from("hello"));
            let mut s = supplier;
            assert_eq!(s.get(), "hello");
        }
    }

    mod test_constant {
        use super::ArcStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_returns_same_value() {
            let constant = ArcStatefulSupplier::constant(42);
            let mut s = constant;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }
    }

    mod test_get {
        use super::Arc;
        use super::ArcStatefulSupplier;
        use super::Mutex;
        use super::StatefulSupplier;

        #[test]
        fn test_can_be_called_multiple_times() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_stateful_counter() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().expect("mutex should not be poisoned");
                *c += 1;
                *c
            });

            let mut s = supplier;
            assert_eq!(s.get(), 1);
            assert_eq!(s.get(), 2);
            assert_eq!(s.get(), 3);
        }
    }

    mod test_clone {
        use super::Arc;
        use super::ArcStatefulSupplier;
        use super::Mutex;
        use super::StatefulSupplier;

        #[test]
        fn test_can_be_cloned() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let clone1 = supplier.clone();
            let clone2 = supplier.clone();

            let mut s1 = clone1;
            let mut s2 = clone2;
            assert_eq!(s1.get(), 42);
            assert_eq!(s2.get(), 42);
        }

        #[test]
        fn test_clones_share_state() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().expect("mutex should not be poisoned");
                *c += 1;
                *c
            });

            let mut s1 = supplier.clone();
            let mut s2 = supplier.clone();

            assert_eq!(s1.get(), 1);
            assert_eq!(s2.get(), 2);
        }
    }

    mod test_map {
        use super::Arc;
        use super::ArcStatefulSupplier;
        use super::Mutex;
        use super::StatefulSupplier;
        use super::thread;

        #[test]
        fn test_transforms_value() {
            let source = ArcStatefulSupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            let mut s = mapped;
            assert_eq!(s.get(), 20);
        }

        #[test]
        fn test_original_remains_usable() {
            let source = ArcStatefulSupplier::new(|| 10);
            let _mapped = source.map(|x| x * 2);
            // source is still usable
            let mut s = source;
            assert_eq!(s.get(), 10);
        }

        #[test]
        fn test_multiple_maps_from_same_source() {
            let source = ArcStatefulSupplier::new(|| 10);
            let doubled = source.map(|x| x * 2);
            let tripled = source.map(|x| x * 3);

            let mut d = doubled;
            let mut t = tripled;
            assert_eq!(d.get(), 20);
            assert_eq!(t.get(), 30);
        }

        // Test with function pointer
        #[test]
        fn test_with_function_pointer() {
            fn triple(x: i32) -> i32 {
                x * 3
            }
            let source = ArcStatefulSupplier::new(|| 10);
            let mapped = source.map(triple);
            let mut s = mapped;
            assert_eq!(s.get(), 30);
        }

        // Test thread safety with mapper
        #[test]
        fn test_thread_safety_with_mapper() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let source = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().expect("mutex should not be poisoned");
                *c += 1;
                *c
            });

            let mapped = source.map(|x| x * 10);
            let mut s1 = mapped.clone();
            let mut s2 = mapped.clone();

            let h1 = thread::spawn(move || s1.get());
            let h2 = thread::spawn(move || s2.get());

            let v1 = h1.join().expect("thread should not panic");
            let v2 = h2.join().expect("thread should not panic");

            // Both should get different values (10 and 20)
            assert!(v1 == 10 || v1 == 20);
            assert!(v2 == 10 || v2 == 20);
            assert_ne!(v1, v2);
        }
    }

    mod test_filter {
        use super::Arc;
        use super::ArcStatefulSupplier;
        use super::Mutex;
        use super::StatefulSupplier;

        #[test]
        fn test_filters_even_numbers() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let source = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().expect("mutex should not be poisoned");
                *c += 1;
                *c
            });
            let filtered = source.filter(|x: &i32| x % 2 == 0);

            let mut s = filtered;
            assert_eq!(s.get(), None); // 1 is odd
            assert_eq!(s.get(), Some(2)); // 2 is even
        }
    }

    mod test_zip {
        use super::ArcStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_combines_two_stateful_suppliers() {
            let first = ArcStatefulSupplier::new(|| 42);
            let second = ArcStatefulSupplier::new(|| "hello");
            let zipped = first.zip(second.clone());

            let mut z = zipped;
            assert_eq!(z.get(), (42, "hello"));
        }

        #[test]
        fn test_originals_remain_usable() {
            let first = ArcStatefulSupplier::new(|| 42);
            let second = ArcStatefulSupplier::new(|| "hello");
            let _zipped = first.zip(second.clone());

            // Both originals still usable
            let mut f = first;
            let mut s = second;
            assert_eq!(f.get(), 42);
            assert_eq!(s.get(), "hello");
        }
    }

    mod test_memoize {
        use super::Arc;
        use super::ArcStatefulSupplier;
        use super::Mutex;
        use super::StatefulSupplier;

        #[test]
        fn test_caches_first_value() {
            let call_count = Arc::new(Mutex::new(0));
            let call_count_clone = Arc::clone(&call_count);
            let source = ArcStatefulSupplier::new(move || {
                let mut c = call_count_clone.lock().expect("mutex should not be poisoned");
                *c += 1;
                42
            });
            let memoized = source.memoize();

            let mut s = memoized;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
            assert_eq!(*call_count.lock().expect("mutex should not be poisoned"), 1);
            let mut source = source;
            assert_eq!(source.get(), 42);
            assert_eq!(*call_count.lock().expect("mutex should not be poisoned"), 2);
        }
    }

    mod test_thread_safety {
        use super::Arc;
        use super::ArcStatefulSupplier;
        use super::Mutex;
        use super::StatefulSupplier;
        use super::thread;

        #[test]
        fn test_can_be_sent_across_threads() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().expect("mutex should not be poisoned");
                *c += 1;
                *c
            });

            let mut s1 = supplier.clone();
            let mut s2 = supplier.clone();

            let h1 = thread::spawn(move || s1.get());
            let h2 = thread::spawn(move || s2.get());

            let v1 = h1.join().expect("thread should not panic");
            let v2 = h2.join().expect("thread should not panic");

            assert!(v1 != v2);
            assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 2);
        }
    }
}

// ==========================================================================
// RcStatefulSupplier Tests
// ==========================================================================
