/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for StatefulSupplier types

use prism3_function::{
    ArcStatefulSupplier,
    BoxStatefulSupplier,
    FnStatefulSupplierOps,
    RcStatefulSupplier,
    StatefulSupplier,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};
use std::thread;

// ==========================================================================
// StatefulSupplier Trait Tests (for closures)
// ==========================================================================

#[cfg(test)]
mod test_stateful_supplier_trait {
    use super::*;

    #[test]
    fn test_closure_to_box() {
        let closure = || 42;
        let mut boxed = closure.to_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_to_rc() {
        let closure = || 42;
        let mut rc = closure.to_rc();
        assert_eq!(rc.get(), 42);
    }

    #[test]
    fn test_closure_to_arc() {
        let closure = || 42;
        let mut arc = closure.to_arc();
        assert_eq!(arc.get(), 42);
    }

    #[test]
    fn test_closure_to_fn() {
        let closure = || 42;
        let mut f = closure.to_fn();
        assert_eq!(f(), 42);
        assert_eq!(f(), 42);
    }

    #[test]
    fn test_closure_implements_stateful_supplier() {
        let closure = || 42;
        let mut boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_stateful() {
        let mut counter = 0;
        let mut boxed = BoxStatefulSupplier::new(move || {
            counter += 1;
            counter
        });
        assert_eq!(boxed.get(), 1);
        assert_eq!(boxed.get(), 2);
        assert_eq!(boxed.get(), 3);
    }

    #[test]
    fn test_into_box() {
        let closure = || 42;
        let mut boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_rc() {
        let closure = || 42;
        let mut rc = closure.into_rc();
        assert_eq!(rc.get(), 42);
    }

    #[test]
    fn test_into_arc() {
        let closure = || 42;
        let mut arc = closure.into_arc();
        assert_eq!(arc.get(), 42);
    }

    #[test]
    fn test_closure_get() {
        // Test the get method in impl<T, F> StatefulSupplier<T> for F
        let mut closure = || 42;
        assert_eq!(closure.get(), 42);
        assert_eq!(closure.get(), 42);
    }

    #[test]
    fn test_closure_get_readonly() {
        // Test readonly closure (Fn)
        let value = 42;
        let mut closure = move || value;
        assert_eq!(closure.get(), 42);
        assert_eq!(closure.get(), 42);
        assert_eq!(closure.get(), 42);
    }

    #[test]
    fn test_closure_into_fn() {
        // Test closure into_fn returns itself
        let closure = || 42;
        let mut f = closure.into_fn();
        assert_eq!(f(), 42);
        assert_eq!(f(), 42);
    }

    #[test]
    fn test_closure_into_fn_stateful() {
        // Test stateful closure into_fn
        let mut counter = 0;
        let closure = move || {
            counter += 1;
            counter
        };
        let mut f = closure.into_fn();
        assert_eq!(f(), 1);
        assert_eq!(f(), 2);
        assert_eq!(f(), 3);
    }

    #[test]
    fn test_closure_into_fn_with_fnmut_function() {
        // Test that into_fn result can be used where FnMut is expected
        fn call_twice<F: FnMut() -> i32>(mut f: F) -> (i32, i32) {
            (f(), f())
        }

        let closure = || 100;
        let f = closure.into_fn();
        assert_eq!(call_twice(f), (100, 100));
    }

    #[test]
    fn test_closure_into_fn_with_string() {
        // Test closure into_fn with non-Copy type
        let closure = || String::from("hello");
        let mut f = closure.into_fn();
        assert_eq!(f(), "hello");
        assert_eq!(f(), "hello");
    }
}

// ==========================================================================
// BoxStatefulSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_box_stateful_supplier {
    use super::*;

    mod test_new {
        use super::*;

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
        use super::*;

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
        use super::*;

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
        use super::*;

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
        use super::*;

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
        use super::*;

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
        use super::*;

        #[test]
        fn test_caches_first_value() {
            // Use a shared counter to verify memoization
            use std::cell::Cell;
            let call_count = Cell::new(0);
            let mut memoized = BoxStatefulSupplier::new(move || {
                call_count.set(call_count.get() + 1);
                42
            })
            .memoize();

            assert_eq!(memoized.get(), 42);
            assert_eq!(memoized.get(), 42);
            assert_eq!(memoized.get(), 42);
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

    mod test_into_box {
        use super::*;

        #[test]
        fn test_returns_self() {
            let supplier = BoxStatefulSupplier::new(|| 42);
            let mut boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_into_rc {
        use super::*;

        #[test]
        fn test_converts_to_rc() {
            let supplier = BoxStatefulSupplier::new(|| 42);
            let mut rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_converts_to_fn() {
            let supplier = BoxStatefulSupplier::new(|| 42);
            let mut f = supplier.into_fn();
            assert_eq!(f(), 42);
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_into_fn_with_stateful_closure() {
            let mut counter = 0;
            let supplier = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            });
            let mut f = supplier.into_fn();
            assert_eq!(f(), 1);
            assert_eq!(f(), 2);
            assert_eq!(f(), 3);
        }

        #[test]
        fn test_into_fn_with_fnmut_function() {
            fn call_twice<F: FnMut() -> i32>(mut f: F) -> (i32, i32) {
                (f(), f())
            }

            let supplier = BoxStatefulSupplier::new(|| 100);
            let f = supplier.into_fn();
            assert_eq!(call_twice(f), (100, 100));
        }

        #[test]
        fn test_into_fn_with_string() {
            let supplier = BoxStatefulSupplier::new(|| String::from("hello"));
            let mut f = supplier.into_fn();
            assert_eq!(f(), "hello");
            assert_eq!(f(), "hello");
        }

        #[test]
        fn test_into_fn_zero_overhead() {
            // This test verifies that into_fn for BoxStatefulSupplier
            // directly returns the inner function without wrapping
            let supplier = BoxStatefulSupplier::new(|| 999);
            let mut f = supplier.into_fn();
            // Should work just like calling the original function
            assert_eq!(f(), 999);
        }
    }
}

// ==========================================================================
// ArcStatefulSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_arc_stateful_supplier {
    use super::*;

    mod test_new {
        use super::*;

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
        use super::*;

        #[test]
        fn test_returns_same_value() {
            let constant = ArcStatefulSupplier::constant(42);
            let mut s = constant;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }
    }

    mod test_get {
        use super::*;

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
                let mut c = counter_clone.lock().unwrap();
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
        use super::*;

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
                let mut c = counter_clone.lock().unwrap();
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
        use super::*;

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
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });

            let mapped = source.map(|x| x * 10);
            let mut s1 = mapped.clone();
            let mut s2 = mapped.clone();

            let h1 = thread::spawn(move || s1.get());
            let h2 = thread::spawn(move || s2.get());

            let v1 = h1.join().unwrap();
            let v2 = h2.join().unwrap();

            // Both should get different values (10 and 20)
            assert!(v1 == 10 || v1 == 20);
            assert!(v2 == 10 || v2 == 20);
            assert_ne!(v1, v2);
        }
    }

    mod test_filter {
        use super::*;

        #[test]
        fn test_filters_even_numbers() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let source = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
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
        use super::*;

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
        use super::*;

        #[test]
        fn test_caches_first_value() {
            let call_count = Arc::new(Mutex::new(0));
            let call_count_clone = Arc::clone(&call_count);
            let source = ArcStatefulSupplier::new(move || {
                let mut c = call_count_clone.lock().unwrap();
                *c += 1;
                42
            });
            let memoized = source.memoize();

            let mut s = memoized;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
            assert_eq!(*call_count.lock().unwrap(), 1);
        }
    }

    mod test_thread_safety {
        use super::*;

        #[test]
        fn test_can_be_sent_across_threads() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });

            let mut s1 = supplier.clone();
            let mut s2 = supplier.clone();

            let h1 = thread::spawn(move || s1.get());
            let h2 = thread::spawn(move || s2.get());

            let v1 = h1.join().unwrap();
            let v2 = h2.join().unwrap();

            assert!(v1 != v2);
            assert_eq!(*counter.lock().unwrap(), 2);
        }
    }

    mod test_into_box {
        use super::*;

        #[test]
        fn test_converts_to_box() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_into_rc {
        use super::*;

        #[test]
        fn test_converts_to_rc() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }
    }

    mod test_into_arc {
        use super::*;

        #[test]
        fn test_returns_self() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut arc = supplier.into_arc();
            assert_eq!(arc.get(), 42);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_converts_to_fn() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut f = supplier.into_fn();
            assert_eq!(f(), 42);
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_into_fn_with_stateful_closure() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });
            let mut f = supplier.into_fn();
            assert_eq!(f(), 1);
            assert_eq!(f(), 2);
            assert_eq!(f(), 3);
            assert_eq!(*counter.lock().unwrap(), 3);
        }

        #[test]
        fn test_into_fn_with_fnmut_function() {
            fn call_twice<F: FnMut() -> i32>(mut f: F) -> (i32, i32) {
                (f(), f())
            }

            let supplier = ArcStatefulSupplier::new(|| 100);
            let f = supplier.into_fn();
            assert_eq!(call_twice(f), (100, 100));
        }

        #[test]
        fn test_into_fn_with_string() {
            let supplier = ArcStatefulSupplier::new(|| String::from("hello"));
            let mut f = supplier.into_fn();
            assert_eq!(f(), "hello");
            assert_eq!(f(), "hello");
        }

        #[test]
        fn test_into_fn_with_mapped_stateful_supplier() {
            let supplier = ArcStatefulSupplier::new(|| 10);
            let mapped = supplier.map(|x| x * 2);
            let mut f = mapped.into_fn();
            assert_eq!(f(), 20);
            assert_eq!(f(), 20);
        }

        #[test]
        fn test_into_fn_thread_safe() {
            // Test that the closure returned by into_fn works with thread-safe data
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });

            let mut f = supplier.into_fn();

            // Call multiple times
            assert_eq!(f(), 1);
            assert_eq!(f(), 2);
            assert_eq!(f(), 3);

            // Verify the counter was incremented correctly
            assert_eq!(*counter.lock().unwrap(), 3);
        }
    }

    mod test_to_box {
        use super::*;

        #[test]
        fn test_creates_box_stateful_supplier() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut boxed = supplier.to_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_to_rc {
        use super::*;

        #[test]
        fn test_creates_rc_stateful_supplier() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut rc = supplier.to_rc();
            assert_eq!(rc.get(), 42);
            assert_eq!(rc.get(), 42);
        }
    }

    mod test_to_arc {
        use super::*;

        #[test]
        fn test_returns_clone() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut arc_clone = supplier.to_arc();
            let mut original = supplier;
            assert_eq!(arc_clone.get(), 42);
            assert_eq!(original.get(), 42);
        }
    }

    mod test_to_fn {
        use super::*;

        #[test]
        fn test_creates_fn() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let mut f = supplier.to_fn();
            assert_eq!(f(), 42);
            assert_eq!(f(), 42);
        }
    }
}

// ==========================================================================
// RcStatefulSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_rc_stateful_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_creates_stateful_supplier() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_i32() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = RcStatefulSupplier::new(|| String::from("hello"));
            let mut s = supplier;
            assert_eq!(s.get(), "hello");
        }
    }

    mod test_constant {
        use super::*;

        #[test]
        fn test_returns_same_value() {
            let constant = RcStatefulSupplier::constant(42);
            let mut s = constant;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn test_can_be_called_multiple_times() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_stateful_counter() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });

            let mut s = supplier;
            assert_eq!(s.get(), 1);
            assert_eq!(s.get(), 2);
            assert_eq!(s.get(), 3);
        }
    }

    mod test_to_box {
        use super::*;

        #[test]
        fn test_creates_box_stateful_supplier() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut boxed = supplier.to_box();
            assert_eq!(boxed.get(), 42);
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_to_rc {
        use super::*;

        #[test]
        fn test_returns_clone() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut first = supplier.to_rc();
            let mut second = supplier;
            assert_eq!(first.get(), 42);
            assert_eq!(second.get(), 42);
        }
    }

    mod test_to_fn {
        use super::*;

        #[test]
        fn test_creates_closure() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut f = supplier.to_fn();
            assert_eq!(f(), 42);
            assert_eq!(f(), 42);
        }
    }

    mod test_clone {
        use super::*;

        #[test]
        fn test_can_be_cloned() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let clone1 = supplier.clone();
            let clone2 = supplier.clone();

            let mut s1 = clone1;
            let mut s2 = clone2;
            assert_eq!(s1.get(), 42);
            assert_eq!(s2.get(), 42);
        }

        #[test]
        fn test_clones_share_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
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
        use super::*;

        #[test]
        fn test_transforms_value() {
            let source = RcStatefulSupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            let mut s = mapped;
            assert_eq!(s.get(), 20);
        }

        #[test]
        fn test_original_remains_usable() {
            let source = RcStatefulSupplier::new(|| 10);
            let _mapped = source.map(|x| x * 2);
            // source is still usable
            let mut s = source;
            assert_eq!(s.get(), 10);
        }

        #[test]
        fn test_multiple_maps_from_same_source() {
            let source = RcStatefulSupplier::new(|| 10);
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
            fn quadruple(x: i32) -> i32 {
                x * 4
            }
            let source = RcStatefulSupplier::new(|| 10);
            let mapped = source.map(quadruple);
            let mut s = mapped;
            assert_eq!(s.get(), 40);
        }

        // Test shared state with cloned StatefulSuppliers
        #[test]
        fn test_shared_state_with_mapper() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let source = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });

            let mapped = source.map(|x| x * 10);
            let mut s1 = mapped.clone();
            let mut s2 = mapped.clone();

            assert_eq!(s1.get(), 10); // counter = 1, 1 * 10
            assert_eq!(s2.get(), 20); // counter = 2, 2 * 10
            assert_eq!(s1.get(), 30); // counter = 3, 3 * 10
        }
    }

    mod test_filter {
        use super::*;

        #[test]
        fn test_filters_even_numbers() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let source = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
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
        use super::*;

        #[test]
        fn test_combines_two_stateful_suppliers() {
            let first = RcStatefulSupplier::new(|| 42);
            let second = RcStatefulSupplier::new(|| "hello");
            let zipped = first.zip(second.clone());

            let mut z = zipped;
            assert_eq!(z.get(), (42, "hello"));
        }

        #[test]
        fn test_originals_remain_usable() {
            let first = RcStatefulSupplier::new(|| 42);
            let second = RcStatefulSupplier::new(|| "hello");
            let _zipped = first.zip(second.clone());

            // Both originals still usable
            let mut f = first;
            let mut s = second;
            assert_eq!(f.get(), 42);
            assert_eq!(s.get(), "hello");
        }
    }

    mod test_memoize {
        use super::*;

        #[test]
        fn test_caches_first_value() {
            let call_count = Rc::new(RefCell::new(0));
            let call_count_clone = Rc::clone(&call_count);
            let source = RcStatefulSupplier::new(move || {
                let mut c = call_count_clone.borrow_mut();
                *c += 1;
                42
            });
            let memoized = source.memoize();

            let mut s = memoized;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
            assert_eq!(*call_count.borrow(), 1);
        }
    }

    mod test_into_box {
        use super::*;

        #[test]
        fn test_converts_to_box() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_into_rc {
        use super::*;

        #[test]
        fn test_returns_self() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_converts_to_fn() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let mut f = supplier.into_fn();
            assert_eq!(f(), 42);
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_into_fn_with_stateful_closure() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });
            let mut f = supplier.into_fn();
            assert_eq!(f(), 1);
            assert_eq!(f(), 2);
            assert_eq!(f(), 3);
            assert_eq!(*counter.borrow(), 3);
        }

        #[test]
        fn test_into_fn_with_fnmut_function() {
            fn call_twice<F: FnMut() -> i32>(mut f: F) -> (i32, i32) {
                (f(), f())
            }

            let supplier = RcStatefulSupplier::new(|| 100);
            let f = supplier.into_fn();
            assert_eq!(call_twice(f), (100, 100));
        }

        #[test]
        fn test_into_fn_with_string() {
            let supplier = RcStatefulSupplier::new(|| String::from("hello"));
            let mut f = supplier.into_fn();
            assert_eq!(f(), "hello");
            assert_eq!(f(), "hello");
        }

        #[test]
        fn test_into_fn_with_mapped_stateful_supplier() {
            let supplier = RcStatefulSupplier::new(|| 10);
            let mapped = supplier.map(|x| x * 2);
            let mut f = mapped.into_fn();
            assert_eq!(f(), 20);
            assert_eq!(f(), 20);
        }

        #[test]
        fn test_into_fn_with_shared_state() {
            // Test that the closure returned by into_fn shares state correctly
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });

            let mut f = supplier.into_fn();

            // Call multiple times
            assert_eq!(f(), 1);
            assert_eq!(f(), 2);
            assert_eq!(f(), 3);

            // Verify the counter was incremented correctly
            assert_eq!(*counter.borrow(), 3);
        }
    }

    // Note: RcStatefulSupplier cannot be converted to ArcStatefulSupplier because
    // Rc is not Send. This is prevented at compile time by the
    // trait bound, so we don't test it.
}

// ==========================================================================
// SupplierOnce Implementation Tests for BoxStatefulSupplier
// ==========================================================================

#[cfg(test)]
mod test_box_stateful_supplier_once {
    use super::*;
    use prism3_function::SupplierOnce;

    mod test_get {
        use super::*;

        #[test]
        fn test_consumes_stateful_supplier() {
            let supplier = BoxStatefulSupplier::new(|| 42);
            let value = supplier.get_once();
            assert_eq!(value, 42);
            // StatefulSupplier is consumed, cannot be used again
        }

        #[test]
        fn test_with_string() {
            let supplier = BoxStatefulSupplier::new(|| String::from("hello"));
            let value = supplier.get_once();
            assert_eq!(value, "hello");
        }

        #[test]
        fn test_with_vec() {
            let supplier = BoxStatefulSupplier::new(|| vec![1, 2, 3]);
            let value = supplier.get_once();
            assert_eq!(value, vec![1, 2, 3]);
        }

        #[test]
        fn test_moves_captured_value() {
            let data = String::from("captured");
            let supplier = BoxStatefulSupplier::new(move || data.clone());
            let value = supplier.get_once();
            assert_eq!(value, "captured");
        }

        #[test]
        fn test_with_stateful_closure() {
            let mut counter = 0;
            let supplier = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            });
            let value = supplier.get_once();
            assert_eq!(value, 1);
        }
    }

    mod test_into_box {
        use super::*;
        use prism3_function::BoxSupplierOnce;

        #[test]
        fn test_converts_to_box_stateful_supplier_once() {
            let supplier = BoxStatefulSupplier::new(|| 42);
            let once: BoxSupplierOnce<i32> = supplier.into_box_once();
            assert_eq!(once.get_once(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = BoxStatefulSupplier::new(|| String::from("test"));
            let once = supplier.into_box_once();
            assert_eq!(once.get_once(), "test");
        }

        #[test]
        fn test_with_moved_value() {
            let data = vec![1, 2, 3];
            let supplier = BoxStatefulSupplier::new(move || data.clone());
            let once = supplier.into_box_once();
            assert_eq!(once.get_once(), vec![1, 2, 3]);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_converts_to_fn_once() {
            let supplier = BoxStatefulSupplier::new(|| 42);
            let f = supplier.into_fn_once();
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = BoxStatefulSupplier::new(|| String::from("hello"));
            let f = supplier.into_fn_once();
            assert_eq!(f(), "hello");
        }

        #[test]
        fn test_with_moved_value() {
            let data = String::from("captured");
            let supplier = BoxStatefulSupplier::new(move || data.clone());
            let f = supplier.into_fn_once();
            assert_eq!(f(), "captured");
        }

        #[test]
        fn test_fn_once_closure_can_be_called() {
            let supplier = BoxStatefulSupplier::new(|| 100);
            let f = supplier.into_fn_once();
            let result = f();
            assert_eq!(result, 100);
        }

        #[test]
        fn test_with_stateful_closure() {
            let mut counter = 0;
            let supplier = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            });
            let f = supplier.into_fn_once();
            assert_eq!(f(), 1);
        }
    }

    // Note: BoxStatefulSupplier does not implement Clone, so it cannot have
    // to_box and to_fn implementations that borrow &self. Attempting
    // to call these methods will result in a compiler error.
}

// ==========================================================================
// SupplierOnce Implementation Tests for ArcStatefulSupplier
// ==========================================================================

#[cfg(test)]
mod test_arc_stateful_supplier_once {
    use super::*;
    use prism3_function::SupplierOnce;

    mod test_get {
        use super::*;

        #[test]
        fn test_consumes_stateful_supplier() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let value = supplier.get_once();
            assert_eq!(value, 42);
            // StatefulSupplier is consumed, cannot be used again
        }

        #[test]
        fn test_with_string() {
            let supplier = ArcStatefulSupplier::new(|| String::from("hello"));
            let value = supplier.get_once();
            assert_eq!(value, "hello");
        }

        #[test]
        fn test_with_vec() {
            let supplier = ArcStatefulSupplier::new(|| vec![1, 2, 3]);
            let value = supplier.get_once();
            assert_eq!(value, vec![1, 2, 3]);
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });
            let value = supplier.get_once();
            assert_eq!(value, 1);
            assert_eq!(*counter.lock().unwrap(), 1);
        }

        #[test]
        fn test_cloned_stateful_suppliers_share_state() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone1 = Arc::clone(&counter);

            let stateful_supplier1 = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone1.lock().unwrap();
                *c += 1;
                *c
            });

            let stateful_supplier2 = stateful_supplier1.clone();

            let value1 = SupplierOnce::get_once(stateful_supplier1);
            let value2 = SupplierOnce::get_once(stateful_supplier2);

            // Both should increment the same counter
            assert_eq!(value1 + value2, 3); // 1 + 2
            assert_eq!(*counter.lock().unwrap(), 2);
        }
    }

    mod test_into_box {
        use super::*;
        use prism3_function::BoxSupplierOnce;

        #[test]
        fn test_converts_to_box_stateful_supplier_once() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let once: BoxSupplierOnce<i32> = supplier.into_box_once();
            assert_eq!(once.get_once(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = ArcStatefulSupplier::new(|| String::from("test"));
            let once = supplier.into_box_once();
            assert_eq!(once.get_once(), "test");
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });
            let once = supplier.into_box_once();
            assert_eq!(once.get_once(), 1);
            assert_eq!(*counter.lock().unwrap(), 1);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_converts_to_fn_once() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let f = supplier.into_fn_once();
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = ArcStatefulSupplier::new(|| String::from("hello"));
            let f = supplier.into_fn_once();
            assert_eq!(f(), "hello");
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });
            let f = supplier.into_fn_once();
            assert_eq!(f(), 1);
            assert_eq!(*counter.lock().unwrap(), 1);
        }

        #[test]
        fn test_fn_once_with_thread_safety() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });
            let f = supplier.into_fn_once();
            assert_eq!(f(), 1);
        }
    }

    mod test_to_box {
        use super::*;
        use prism3_function::BoxSupplierOnce;

        #[test]
        fn test_creates_box_stateful_supplier_once() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let once: BoxSupplierOnce<i32> = supplier.to_box_once();
            assert_eq!(once.get_once(), 42);
        }

        #[test]
        fn test_original_remains_usable() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let _once = supplier.to_box_once();
            // Original StatefulSupplier still usable
            let s = supplier;
            assert_eq!(s.clone().get(), 42);
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });

            let once = supplier.to_box_once();
            assert_eq!(once.get_once(), 1);

            // Can still use original
            assert_eq!(supplier.clone().get(), 2);
            assert_eq!(*counter.lock().unwrap(), 2);
        }
    }

    mod test_to_fn {
        use super::*;

        #[test]
        fn test_creates_fn_once() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let f = supplier.to_fn_once();
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_original_remains_usable() {
            let supplier = ArcStatefulSupplier::new(|| 42);
            let f = supplier.to_fn_once();
            // Original StatefulSupplier still usable
            assert_eq!(supplier.clone().get(), 42);
            // Call the function we created
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcStatefulSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });

            let f = supplier.to_fn_once();
            assert_eq!(f(), 1);

            // Can still use original
            assert_eq!(supplier.clone().get(), 2);
            assert_eq!(*counter.lock().unwrap(), 2);
        }
    }
}

// ==========================================================================
// SupplierOnce Implementation Tests for RcStatefulSupplier
// ==========================================================================

#[cfg(test)]
mod test_rc_stateful_supplier_once {
    use super::*;
    use prism3_function::SupplierOnce;

    mod test_get {
        use super::*;

        #[test]
        fn test_consumes_stateful_supplier() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let value = supplier.get_once();
            assert_eq!(value, 42);
            // StatefulSupplier is consumed, cannot be used again
        }

        #[test]
        fn test_with_string() {
            let supplier = RcStatefulSupplier::new(|| String::from("hello"));
            let value = supplier.get_once();
            assert_eq!(value, "hello");
        }

        #[test]
        fn test_with_vec() {
            let supplier = RcStatefulSupplier::new(|| vec![1, 2, 3]);
            let value = supplier.get_once();
            assert_eq!(value, vec![1, 2, 3]);
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });
            let value = supplier.get_once();
            assert_eq!(value, 1);
            assert_eq!(*counter.borrow(), 1);
        }

        #[test]
        fn test_cloned_stateful_suppliers_share_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone1 = Rc::clone(&counter);

            let stateful_supplier1 = RcStatefulSupplier::new(move || {
                let mut c = counter_clone1.borrow_mut();
                *c += 1;
                *c
            });

            let stateful_supplier2 = stateful_supplier1.clone();

            let value1 = SupplierOnce::get_once(stateful_supplier1);
            let value2 = SupplierOnce::get_once(stateful_supplier2);

            // Both should increment the same counter
            assert_eq!(value1 + value2, 3); // 1 + 2
            assert_eq!(*counter.borrow(), 2);
        }
    }

    mod test_into_box {
        use super::*;
        use prism3_function::BoxSupplierOnce;

        #[test]
        fn test_converts_to_box_stateful_supplier_once() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let once: BoxSupplierOnce<i32> = supplier.into_box_once();
            assert_eq!(once.get_once(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = RcStatefulSupplier::new(|| String::from("test"));
            let once = supplier.into_box_once();
            assert_eq!(once.get_once(), "test");
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });
            let once = supplier.into_box_once();
            assert_eq!(once.get_once(), 1);
            assert_eq!(*counter.borrow(), 1);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_converts_to_fn_once() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let f = supplier.into_fn_once();
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = RcStatefulSupplier::new(|| String::from("hello"));
            let f = supplier.into_fn_once();
            assert_eq!(f(), "hello");
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });
            let f = supplier.into_fn_once();
            assert_eq!(f(), 1);
            assert_eq!(*counter.borrow(), 1);
        }

        #[test]
        fn test_fn_once_with_shared_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });
            let f = supplier.into_fn_once();
            assert_eq!(f(), 1);
        }
    }

    mod test_to_box {
        use super::*;
        use prism3_function::BoxSupplierOnce;

        #[test]
        fn test_creates_box_stateful_supplier_once() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let once: BoxSupplierOnce<i32> = supplier.to_box_once();
            assert_eq!(once.get_once(), 42);
        }

        #[test]
        fn test_original_remains_usable() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let _once = supplier.to_box_once();
            // Original StatefulSupplier still usable
            let s = supplier;
            assert_eq!(s.clone().get(), 42);
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });

            let once = supplier.to_box_once();
            assert_eq!(once.get_once(), 1);

            // Can still use original
            assert_eq!(supplier.clone().get(), 2);
            assert_eq!(*counter.borrow(), 2);
        }
    }

    mod test_to_fn {
        use super::*;

        #[test]
        fn test_creates_fn_once() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let f = supplier.to_fn_once();
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_original_remains_usable() {
            let supplier = RcStatefulSupplier::new(|| 42);
            let f = supplier.to_fn_once();
            // Original StatefulSupplier still usable
            assert_eq!(supplier.clone().get(), 42);
            // Call the function we created
            assert_eq!(f(), 42);
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });

            let f = supplier.to_fn_once();
            assert_eq!(f(), 1);

            // Can still use original
            assert_eq!(supplier.clone().get(), 2);
            assert_eq!(*counter.borrow(), 2);
        }
    }
}

// ==========================================================================
// Custom StatefulSupplier Implementation Tests
// ==========================================================================

#[cfg(test)]
mod test_custom_stateful_supplier_default_impl {
    use super::*;

    /// A custom StatefulSupplier implementation that only implements the
    /// core `get()` method, relying on default implementations for
    /// conversion methods.
    struct CounterStatefulSupplier {
        counter: i32,
    }

    impl CounterStatefulSupplier {
        fn new(initial: i32) -> Self {
            Self { counter: initial }
        }
    }

    impl StatefulSupplier<i32> for CounterStatefulSupplier {
        fn get(&mut self) -> i32 {
            // For readonly StatefulSupplier, we can't modify state
            // This is just a demo, return the counter value
            self.counter
        }
        // Note: into_box(), into_rc(), and into_arc() use the
        // default implementations from the trait
    }

    #[test]
    fn test_custom_stateful_supplier_into_box() {
        // Create a custom StatefulSupplier with initial value 42
        let custom = CounterStatefulSupplier::new(42);

        // Convert to BoxStatefulSupplier using the default implementation
        let mut boxed = custom.into_box();

        // Verify it works correctly
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_custom_stateful_supplier_into_rc() {
        // Create a custom StatefulSupplier with initial value 10
        let custom = CounterStatefulSupplier::new(10);

        // Convert to RcStatefulSupplier using the default implementation
        let mut rc = custom.into_rc();

        // Verify it works correctly
        assert_eq!(rc.get(), 10);
        assert_eq!(rc.get(), 10);
        assert_eq!(rc.get(), 10);
    }

    #[test]
    fn test_custom_stateful_supplier_into_arc() {
        // Create a custom StatefulSupplier with initial value 100
        let custom = CounterStatefulSupplier::new(100);

        // Convert to ArcStatefulSupplier using the default implementation
        let mut arc = custom.into_arc();

        // Verify it works correctly
        assert_eq!(arc.get(), 100);
        assert_eq!(arc.get(), 100);
        assert_eq!(arc.get(), 100);
    }

    #[test]
    fn test_custom_stateful_supplier_clone_and_share() {
        // Create a custom StatefulSupplier and convert to RcStatefulSupplier
        let custom = CounterStatefulSupplier::new(42);
        let rc = custom.into_rc();

        // Clone the RcStatefulSupplier to share state
        let mut s1 = rc.clone();
        let mut s2 = rc.clone();

        // Verify shared state works correctly - they share the
        // same underlying value
        assert_eq!(s1.get(), 42);
        assert_eq!(s2.get(), 42);
        assert_eq!(s1.get(), 42);
    }

    #[test]
    fn test_custom_stateful_supplier_thread_safety() {
        // Create a custom StatefulSupplier and convert to ArcStatefulSupplier
        let custom = CounterStatefulSupplier::new(100);
        let arc = custom.into_arc();

        // Clone for use in threads
        let mut s1 = arc.clone();
        let mut s2 = arc.clone();

        let h1 = thread::spawn(move || s1.get());
        let h2 = thread::spawn(move || s2.get());

        let v1 = h1.join().unwrap();
        let v2 = h2.join().unwrap();

        // Both threads should get the same value (readonly)
        assert_eq!(v1, 100);
        assert_eq!(v2, 100);
    }

    #[test]
    fn test_custom_stateful_supplier_with_string() {
        /// A custom StatefulSupplier that generates sequential string IDs
        struct IdStatefulSupplier {
            next_id: u32,
        }

        impl IdStatefulSupplier {
            fn new() -> Self {
                Self { next_id: 1 }
            }
        }

        impl StatefulSupplier<String> for IdStatefulSupplier {
            fn get(&mut self) -> String {
                // For readonly StatefulSupplier, return the same ID
                format!("ID-{:04}", self.next_id)
            }
        }

        // Test with BoxStatefulSupplier
        let id_gen = IdStatefulSupplier::new();
        let mut boxed = id_gen.into_box();
        assert_eq!(boxed.get(), "ID-0001");
        assert_eq!(boxed.get(), "ID-0001");
        assert_eq!(boxed.get(), "ID-0001");
    }

    #[test]
    fn test_custom_stateful_supplier_into_fn() {
        // Test the default implementation of into_fn for custom StatefulSupplier
        let custom = CounterStatefulSupplier::new(42);

        // Convert to closure using the default implementation
        let mut f = custom.into_fn();

        // Verify it works correctly
        assert_eq!(f(), 42);
        assert_eq!(f(), 42);
        assert_eq!(f(), 42);
    }

    #[test]
    fn test_custom_stateful_supplier_into_fn_with_fnmut_function() {
        // Test that custom StatefulSupplier's into_fn result works with FnMut
        fn call_twice<F: FnMut() -> i32>(mut f: F) -> (i32, i32) {
            (f(), f())
        }

        let custom = CounterStatefulSupplier::new(10);
        let f = custom.into_fn();
        assert_eq!(call_twice(f), (10, 10));
    }

    #[test]
    fn test_custom_stateful_supplier_into_fn_with_string() {
        /// A custom StatefulSupplier that generates sequential string IDs
        struct IdStatefulSupplier {
            next_id: u32,
        }

        impl IdStatefulSupplier {
            fn new() -> Self {
                Self { next_id: 1 }
            }
        }

        impl StatefulSupplier<String> for IdStatefulSupplier {
            fn get(&mut self) -> String {
                // For readonly StatefulSupplier, return the same ID
                format!("ID-{:04}", self.next_id)
            }
        }

        // Test with into_fn
        let id_gen = IdStatefulSupplier::new();
        let mut f = id_gen.into_fn();
        assert_eq!(f(), "ID-0001");
        assert_eq!(f(), "ID-0001");
        assert_eq!(f(), "ID-0001");
    }

    #[test]
    fn test_custom_stateful_supplier_into_fn_default_impl() {
        /// Test that the default into_fn implementation wraps get() correctly
        struct SimpleStatefulSupplier {
            value: i32,
        }

        impl SimpleStatefulSupplier {
            fn new(value: i32) -> Self {
                Self { value }
            }
        }

        impl StatefulSupplier<i32> for SimpleStatefulSupplier {
            fn get(&mut self) -> i32 {
                self.value
            }
            // Only implements get(), relying on default into_fn
        }

        let supplier = SimpleStatefulSupplier::new(999);
        let mut f = supplier.into_fn();

        // Verify it uses the get() method correctly
        assert_eq!(f(), 999);
        assert_eq!(f(), 999);
    }

    #[test]
    fn test_custom_stateful_supplier_into_fn_composition() {
        // Test that into_fn works correctly when composing with other operations
        let custom = CounterStatefulSupplier::new(0);

        // First convert to BoxStatefulSupplier, then to closure
        let boxed = custom.into_box();
        let mut f = boxed.into_fn();

        assert_eq!(f(), 0);
        assert_eq!(f(), 0);
        assert_eq!(f(), 0);
    }
}

// ==========================================================================
// FnStatefulSupplierOps Extension Trait Tests
// ==========================================================================

#[cfg(test)]
mod test_fn_stateful_supplier_ops {
    use super::*;

    #[test]
    fn test_closure_map() {
        // Test map method on closure
        let mut mapped = (|| 10).map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_closure_map_chain() {
        // Test chaining multiple map operations
        let mut mapped = (|| 10).map(|x| x * 2).map(|x| x + 5);
        assert_eq!(mapped.get(), 25);
        assert_eq!(mapped.get(), 25);
    }

    #[test]
    fn test_closure_map_stateful() {
        // Test map on stateful closure
        let mut counter = 0;
        let mut mapped = (move || {
            counter += 1;
            counter
        })
        .map(|x| x * 2);

        assert_eq!(mapped.get(), 2);
        assert_eq!(mapped.get(), 4);
        assert_eq!(mapped.get(), 6);
    }

    #[test]
    fn test_closure_map_type_conversion() {
        // Test map with type conversion
        let mut mapped = (|| 42).map(|x: i32| x.to_string());
        assert_eq!(mapped.get(), "42");
    }

    #[test]
    fn test_closure_filter() {
        // Test filter method on closure
        let mut counter = 0;
        let mut filtered = (move || {
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
    fn test_closure_filter_always_pass() {
        // Test filter that always passes
        let mut filtered = (|| 42).filter(|_: &i32| true);
        assert_eq!(filtered.get(), Some(42));
        assert_eq!(filtered.get(), Some(42));
    }

    #[test]
    fn test_closure_filter_always_fail() {
        // Test filter that always fails
        let mut filtered = (|| 42).filter(|_: &i32| false);
        assert_eq!(filtered.get(), None);
        assert_eq!(filtered.get(), None);
    }

    #[test]
    fn test_closure_filter_with_map() {
        // Test combining filter and map
        let mut counter = 0;
        let mut pipeline = (move || {
            counter += 1;
            counter
        })
        .filter(|x: &i32| x % 2 == 0)
        .map(|opt: Option<i32>| opt.map(|x| x * 10));

        assert_eq!(pipeline.get(), None); // 1 is odd
        assert_eq!(pipeline.get(), Some(20)); // 2 is even, doubled to 20
        assert_eq!(pipeline.get(), None); // 3 is odd
        assert_eq!(pipeline.get(), Some(40)); // 4 is even, doubled to 40
    }

    #[test]
    fn test_closure_zip() {
        // Test zip method on closure
        let first = || 42;
        let second = BoxStatefulSupplier::new(|| "hello");
        let mut zipped = first.zip(second);

        assert_eq!(zipped.get(), (42, "hello"));
        assert_eq!(zipped.get(), (42, "hello"));
    }

    #[test]
    fn test_closure_zip_stateful() {
        // Test zip with stateful closures
        let mut counter1 = 0;
        let first = move || {
            counter1 += 1;
            counter1
        };

        let mut counter2 = 100;
        let second = BoxStatefulSupplier::new(move || {
            counter2 += 1;
            counter2
        });

        let mut zipped = first.zip(second);

        assert_eq!(zipped.get(), (1, 101));
        assert_eq!(zipped.get(), (2, 102));
        assert_eq!(zipped.get(), (3, 103));
    }

    #[test]
    fn test_closure_zip_different_types() {
        // Test zip with different types
        let first = || 42;
        let second = BoxStatefulSupplier::new(|| "world");
        let mut zipped = first.zip(second);

        let result = zipped.get();
        assert_eq!(result.0, 42);
        assert_eq!(result.1, "world");
    }

    #[test]
    fn test_closure_memoize() {
        // Test memoize method on closure
        let mut memoized = (|| 42).memoize();

        // First call executes the closure
        assert_eq!(memoized.get(), 42);
        // Subsequent calls return cached value
        assert_eq!(memoized.get(), 42);
        assert_eq!(memoized.get(), 42);
    }

    #[test]
    fn test_closure_memoize_with_map() {
        // Test combining memoize and map
        let mut pipeline = (|| 10).memoize().map(|x| x * 2);

        assert_eq!(pipeline.get(), 20);
        assert_eq!(pipeline.get(), 20);
        assert_eq!(pipeline.get(), 20);
    }

    #[test]
    fn test_closure_complex_pipeline() {
        // Test complex pipeline with multiple operations
        let mut counter = 0;
        let mut pipeline = (move || {
            counter += 1;
            counter
        })
        .map(|x| x * 2)
        .filter(|x: &i32| x % 4 == 0)
        .map(|opt: Option<i32>| opt.unwrap_or(0));

        assert_eq!(pipeline.get(), 0); // 1*2=2, 2%4!=0, filtered out
        assert_eq!(pipeline.get(), 4); // 2*2=4, 4%4==0, passed
        assert_eq!(pipeline.get(), 0); // 3*2=6, 6%4!=0, filtered out
        assert_eq!(pipeline.get(), 8); // 4*2=8, 8%4==0, passed
    }

    #[test]
    fn test_closure_map_then_zip() {
        // Test combining map and zip
        let first = (|| 10).map(|x| x * 2);
        let second = BoxStatefulSupplier::new(|| 5);
        let mut zipped = first.zip(second);

        assert_eq!(zipped.get(), (20, 5));
    }

    #[test]
    fn test_closure_filter_then_zip() {
        // Test combining filter and zip
        let mut counter = 0;
        let filtered = (move || {
            counter += 1;
            counter
        })
        .filter(|x: &i32| x % 2 == 0);

        let second = BoxStatefulSupplier::new(|| "test");
        let mut zipped = filtered.zip(second);

        assert_eq!(zipped.get(), (None, "test")); // 1 is odd
        assert_eq!(zipped.get(), (Some(2), "test")); // 2 is even
    }

    #[test]
    fn test_closure_all_operations() {
        // Test using all operations in one pipeline
        let mut counter = 0;
        let mut pipeline = (move || {
            counter += 1;
            counter
        })
        .map(|x| x * 2) // Double the counter
        .filter(|x: &i32| x % 4 == 0) // Keep only multiples of 4
        .map(|opt| match opt {
            Some(x) => x / 2, // Convert back
            None => 0,
        });

        assert_eq!(pipeline.get(), 0); // 1*2=2, not multiple of 4
        assert_eq!(pipeline.get(), 2); // 2*2=4, multiple of 4, 4/2=2
        assert_eq!(pipeline.get(), 0); // 3*2=6, not multiple of 4
        assert_eq!(pipeline.get(), 4); // 4*2=8, multiple of 4, 8/2=4
    }

    #[test]
    fn test_function_pointer_map() {
        // Test map with function pointer
        fn double(x: i32) -> i32 {
            x * 2
        }

        let supplier = || 10;
        let mut mapped = supplier.map(double);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_function_pointer_filter() {
        // Test filter with function pointer
        fn is_even(x: &i32) -> bool {
            x % 2 == 0
        }

        let mut counter = 0;
        let mut filtered = (move || {
            counter += 1;
            counter
        })
        .filter(is_even);

        assert_eq!(filtered.get(), None); // 1 is odd
        assert_eq!(filtered.get(), Some(2)); // 2 is even
    }

    #[test]
    fn test_closure_string_operations() {
        // Test with String type
        let mut mapped = (|| "hello".to_string()).map(|s: String| s.to_uppercase());
        assert_eq!(mapped.get(), "HELLO");
    }

    #[test]
    fn test_closure_vec_operations() {
        // Test with Vec type
        let mut mapped = (|| vec![1, 2, 3]).map(|v: Vec<i32>| v.len());
        assert_eq!(mapped.get(), 3);
    }

    #[test]
    fn test_closure_option_operations() {
        // Test with Option type
        let mut mapped = (|| Some(42)).map(|opt: Option<i32>| opt.unwrap_or(0));
        assert_eq!(mapped.get(), 42);

        let mut mapped_none = (|| None::<i32>).map(|opt: Option<i32>| opt.unwrap_or(0));
        assert_eq!(mapped_none.get(), 0);
    }

    #[test]
    fn test_closure_result_operations() {
        // Test with Result type
        let mut mapped =
            (|| Ok::<i32, String>(42)).map(|res: Result<i32, String>| res.unwrap_or(0));
        assert_eq!(mapped.get(), 42);

        let mut mapped_err = (|| Err::<i32, String>("error".to_string()))
            .map(|res: Result<i32, String>| res.unwrap_or(0));
        assert_eq!(mapped_err.get(), 0);
    }

    #[test]
    fn test_closure_tuple_operations() {
        // Test with tuple type
        let mut mapped = (|| (1, 2)).map(|(a, b)| a + b);
        assert_eq!(mapped.get(), 3);
    }

    #[test]
    fn test_closure_nested_map() {
        // Test nested map operations
        let mut mapped = (|| 5)
            .map(|x| x + 1)
            .map(|x| x * 2)
            .map(|x| x - 3)
            .map(|x| x / 2);
        assert_eq!(mapped.get(), 4); // (5+1)*2-3 = 9, 9/2 = 4
    }

    #[test]
    fn test_closure_memoize_clone_behavior() {
        // Test that memoize caches the cloned value
        let mut memoized = (|| vec![1, 2, 3]).memoize();

        let result1 = memoized.get();
        let result2 = memoized.get();

        assert_eq!(result1, vec![1, 2, 3]);
        assert_eq!(result2, vec![1, 2, 3]);
        // Verify they are separate clones
        assert_eq!(result1, result2);
    }
}

#[cfg(test)]
mod test_custom_clone_stateful_supplier {
    use super::*;

    #[derive(Clone)]
    struct CustomStatefulSupplier {
        value: i32,
    }

    impl StatefulSupplier<i32> for CustomStatefulSupplier {
        fn get(&mut self) -> i32 {
            self.value
        }
    }

    #[test]
    fn test_default_to_box() {
        let supplier = CustomStatefulSupplier { value: 10 };
        let mut boxed = supplier.to_box();
        assert_eq!(boxed.get(), 10);
    }

    #[test]
    fn test_default_to_rc() {
        let supplier = CustomStatefulSupplier { value: 11 };
        let mut rc = supplier.to_rc();
        assert_eq!(rc.get(), 11);
    }

    #[test]
    fn test_default_to_arc() {
        let supplier = CustomStatefulSupplier { value: 12 };
        let mut arc = supplier.to_arc();
        assert_eq!(arc.get(), 12);
    }

    #[test]
    fn test_default_to_fn() {
        let supplier = CustomStatefulSupplier { value: 13 };
        let mut f = supplier.to_fn();
        assert_eq!(f(), 13);
    }
}
