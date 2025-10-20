/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for Supplier types

use prism3_function::{
    ArcMapper, ArcSupplier, BoxMapper, BoxSupplier, RcMapper, RcSupplier, Supplier,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

// ==========================================================================
// Supplier Trait Tests (for closures)
// ==========================================================================

#[cfg(test)]
mod test_supplier_trait {
    use super::*;

    #[test]
    fn test_closure_implements_supplier() {
        let closure = || 42;
        let mut boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_stateful() {
        let mut counter = 0;
        let mut boxed = BoxSupplier::new(move || {
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
        // Test the get method in impl<T, F> Supplier<T> for F
        let mut closure = || 42;
        assert_eq!(closure.get(), 42);
        assert_eq!(closure.get(), 42);
    }

    #[test]
    fn test_closure_get_stateful() {
        // Test stateful closure
        let mut counter = 0;
        let mut closure = move || {
            counter += 1;
            counter
        };
        assert_eq!(closure.get(), 1);
        assert_eq!(closure.get(), 2);
        assert_eq!(closure.get(), 3);
    }
}

// ==========================================================================
// BoxSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_box_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_creates_supplier() {
            let mut supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_with_i32() {
            let mut supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let mut supplier = BoxSupplier::new(|| String::from("hello"));
            assert_eq!(supplier.get(), "hello");
        }

        #[test]
        fn test_with_vec() {
            let mut supplier = BoxSupplier::new(|| vec![1, 2, 3]);
            assert_eq!(supplier.get(), vec![1, 2, 3]);
        }

        #[test]
        fn test_with_bool() {
            let mut supplier = BoxSupplier::new(|| true);
            assert!(supplier.get());
        }
    }

    mod test_constant {
        use super::*;

        #[test]
        fn test_returns_same_value() {
            let mut constant = BoxSupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let mut constant = BoxSupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn test_can_be_called_multiple_times() {
            let mut supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_stateful_counter() {
            let mut counter = 0;
            let mut supplier = BoxSupplier::new(move || {
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
            let mut mapped = BoxSupplier::new(|| 10).map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_multiple_chains() {
            let mut chained = BoxSupplier::new(|| 5).map(|x| x * 2).map(|x| x + 5);
            assert_eq!(chained.get(), 15);
        }

        #[test]
        fn test_type_conversion() {
            let mut converted = BoxSupplier::new(|| 42).map(|x: i32| x.to_string());
            assert_eq!(converted.get(), "42");
        }

        #[test]
        fn test_with_stateful_supplier() {
            let mut counter = 0;
            let mut mapped = BoxSupplier::new(move || {
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
            let mut mapped = BoxSupplier::new(|| 10).map(double);
            assert_eq!(mapped.get(), 20);
        }

        // Test with BoxMapper
        #[test]
        fn test_with_box_mapper() {
            let mapper = BoxMapper::new(|x: i32| x * 3);
            let mut supplier = BoxSupplier::new(|| 10).map(mapper);
            assert_eq!(supplier.get(), 30);
        }

        // Test with stateful BoxMapper
        #[test]
        fn test_with_stateful_box_mapper() {
            let mut counter = 0;
            let mapper = BoxMapper::new(move |x: i32| {
                counter += 1;
                x + counter
            });
            let mut supplier = BoxSupplier::new(|| 10).map(mapper);
            assert_eq!(supplier.get(), 11); // 10 + 1
            assert_eq!(supplier.get(), 12); // 10 + 2
            assert_eq!(supplier.get(), 13); // 10 + 3
        }

        // Test with RcMapper
        #[test]
        fn test_with_rc_mapper() {
            let mapper = RcMapper::new(|x: i32| x * 4);
            let mut supplier = BoxSupplier::new(|| 10).map(mapper);
            assert_eq!(supplier.get(), 40);
        }

        // Test chaining with different mapper types
        #[test]
        fn test_chain_with_mapper_and_closure() {
            let mapper = BoxMapper::new(|x: i32| x * 2);
            let mut chained = BoxSupplier::new(|| 5).map(mapper).map(|x| x + 10);
            assert_eq!(chained.get(), 20); // (5 * 2) + 10
        }
    }

    mod test_filter {
        use super::*;

        #[test]
        fn test_filters_even_numbers() {
            let mut counter = 0;
            let mut filtered = BoxSupplier::new(move || {
                counter += 1;
                counter
            })
            .filter(|x| x % 2 == 0);

            assert_eq!(filtered.get(), None); // 1 is odd
            assert_eq!(filtered.get(), Some(2)); // 2 is even
            assert_eq!(filtered.get(), None); // 3 is odd
            assert_eq!(filtered.get(), Some(4)); // 4 is even
        }

        #[test]
        fn test_with_constant_supplier() {
            let mut filtered = BoxSupplier::constant(5).filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), None); // 5 is odd
            assert_eq!(filtered.get(), None);
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_combines_two_suppliers() {
            let first = BoxSupplier::new(|| 42);
            let second = BoxSupplier::new(|| "hello");
            let mut zipped = first.zip(second);

            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_with_stateful_suppliers() {
            let mut counter1 = 0;
            let first = BoxSupplier::new(move || {
                counter1 += 1;
                counter1
            });
            let mut counter2 = 0;
            let second = BoxSupplier::new(move || {
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
            let mut memoized = BoxSupplier::new(move || {
                call_count.set(call_count.get() + 1);
                42
            })
            .memoize();

            assert_eq!(memoized.get(), 42);
            assert_eq!(memoized.get(), 42);
            assert_eq!(memoized.get(), 42);
        }

        #[test]
        fn test_with_stateful_supplier() {
            let mut counter = 0;
            let mut memoized = BoxSupplier::new(move || {
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
            let supplier = BoxSupplier::new(|| 42);
            let mut boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_into_rc {
        use super::*;

        #[test]
        fn test_converts_to_rc() {
            let supplier = BoxSupplier::new(|| 42);
            let mut rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }
    }

    // Note: BoxSupplier cannot be converted to ArcSupplier because
    // the inner function may not be Send. This is prevented at
    // compile time by the trait bound, so we don't test it.
}

// ==========================================================================
// ArcSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_arc_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_creates_supplier() {
            let supplier = ArcSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_i32() {
            let supplier = ArcSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = ArcSupplier::new(|| String::from("hello"));
            let mut s = supplier;
            assert_eq!(s.get(), "hello");
        }
    }

    mod test_constant {
        use super::*;

        #[test]
        fn test_returns_same_value() {
            let constant = ArcSupplier::constant(42);
            let mut s = constant;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn test_can_be_called_multiple_times() {
            let supplier = ArcSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_stateful_counter() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let supplier = ArcSupplier::new(move || {
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
            let supplier = ArcSupplier::new(|| 42);
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
            let supplier = ArcSupplier::new(move || {
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
            let source = ArcSupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            let mut s = mapped;
            assert_eq!(s.get(), 20);
        }

        #[test]
        fn test_original_remains_usable() {
            let source = ArcSupplier::new(|| 10);
            let _mapped = source.map(|x| x * 2);
            // source is still usable
            let mut s = source;
            assert_eq!(s.get(), 10);
        }

        #[test]
        fn test_multiple_maps_from_same_source() {
            let source = ArcSupplier::new(|| 10);
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
            let source = ArcSupplier::new(|| 10);
            let mapped = source.map(triple);
            let mut s = mapped;
            assert_eq!(s.get(), 30);
        }

        // Test with ArcMapper
        #[test]
        fn test_with_arc_mapper() {
            let mapper = ArcMapper::new(|x: i32| x * 4);
            let source = ArcSupplier::new(|| 10);
            let mut supplier = source.map(mapper);
            assert_eq!(supplier.get(), 40);
        }

        // Test with stateful ArcMapper
        #[test]
        fn test_with_stateful_arc_mapper() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let mapper = ArcMapper::new(move |x: i32| {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                x + *c
            });
            let source = ArcSupplier::new(|| 10);
            let mut supplier = source.map(mapper);
            assert_eq!(supplier.get(), 11); // 10 + 1
            assert_eq!(supplier.get(), 12); // 10 + 2
            assert_eq!(supplier.get(), 13); // 10 + 3
        }

        // Test with another ArcMapper
        #[test]
        fn test_with_multiple_arc_mappers() {
            let mapper = ArcMapper::new(|x: i32| x * 5);
            let source = ArcSupplier::new(|| 10);
            let mut supplier = source.map(mapper);
            assert_eq!(supplier.get(), 50);
        }

        // Test chaining with different mapper types
        #[test]
        fn test_chain_with_mapper_and_closure() {
            let mapper = ArcMapper::new(|x: i32| x * 2);
            let source = ArcSupplier::new(|| 5);
            let chained = source.map(mapper);
            let mut final_supplier = chained.map(|x| x + 10);
            assert_eq!(final_supplier.get(), 20); // (5 * 2) + 10
        }

        // Test thread safety with mapper
        #[test]
        fn test_thread_safety_with_mapper() {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);
            let source = ArcSupplier::new(move || {
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
            let source = ArcSupplier::new(move || {
                let mut c = counter_clone.lock().unwrap();
                *c += 1;
                *c
            });
            let filtered = source.filter(|x| x % 2 == 0);

            let mut s = filtered;
            assert_eq!(s.get(), None); // 1 is odd
            assert_eq!(s.get(), Some(2)); // 2 is even
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_combines_two_suppliers() {
            let first = ArcSupplier::new(|| 42);
            let second = ArcSupplier::new(|| "hello");
            let zipped = first.zip(&second);

            let mut z = zipped;
            assert_eq!(z.get(), (42, "hello"));
        }

        #[test]
        fn test_originals_remain_usable() {
            let first = ArcSupplier::new(|| 42);
            let second = ArcSupplier::new(|| "hello");
            let _zipped = first.zip(&second);

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
            let source = ArcSupplier::new(move || {
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
            let supplier = ArcSupplier::new(move || {
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
            let supplier = ArcSupplier::new(|| 42);
            let mut boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_into_rc {
        use super::*;

        #[test]
        fn test_converts_to_rc() {
            let supplier = ArcSupplier::new(|| 42);
            let mut rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }
    }

    mod test_into_arc {
        use super::*;

        #[test]
        fn test_returns_self() {
            let supplier = ArcSupplier::new(|| 42);
            let mut arc = supplier.into_arc();
            assert_eq!(arc.get(), 42);
        }
    }
}

// ==========================================================================
// RcSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_rc_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_creates_supplier() {
            let supplier = RcSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_i32() {
            let supplier = RcSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let supplier = RcSupplier::new(|| String::from("hello"));
            let mut s = supplier;
            assert_eq!(s.get(), "hello");
        }
    }

    mod test_constant {
        use super::*;

        #[test]
        fn test_returns_same_value() {
            let constant = RcSupplier::constant(42);
            let mut s = constant;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn test_can_be_called_multiple_times() {
            let supplier = RcSupplier::new(|| 42);
            let mut s = supplier;
            assert_eq!(s.get(), 42);
            assert_eq!(s.get(), 42);
        }

        #[test]
        fn test_stateful_counter() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let supplier = RcSupplier::new(move || {
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

    mod test_clone {
        use super::*;

        #[test]
        fn test_can_be_cloned() {
            let supplier = RcSupplier::new(|| 42);
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
            let supplier = RcSupplier::new(move || {
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
            let source = RcSupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            let mut s = mapped;
            assert_eq!(s.get(), 20);
        }

        #[test]
        fn test_original_remains_usable() {
            let source = RcSupplier::new(|| 10);
            let _mapped = source.map(|x| x * 2);
            // source is still usable
            let mut s = source;
            assert_eq!(s.get(), 10);
        }

        #[test]
        fn test_multiple_maps_from_same_source() {
            let source = RcSupplier::new(|| 10);
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
            let source = RcSupplier::new(|| 10);
            let mapped = source.map(quadruple);
            let mut s = mapped;
            assert_eq!(s.get(), 40);
        }

        // Test with RcMapper
        #[test]
        fn test_with_rc_mapper() {
            let mapper = RcMapper::new(|x: i32| x * 5);
            let source = RcSupplier::new(|| 10);
            let mut supplier = source.map(mapper);
            assert_eq!(supplier.get(), 50);
        }

        // Test with stateful RcMapper
        #[test]
        fn test_with_stateful_rc_mapper() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let mapper = RcMapper::new(move |x: i32| {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                x + *c
            });
            let source = RcSupplier::new(|| 10);
            let mut supplier = source.map(mapper);
            assert_eq!(supplier.get(), 11); // 10 + 1
            assert_eq!(supplier.get(), 12); // 10 + 2
            assert_eq!(supplier.get(), 13); // 10 + 3
        }

        // Test with BoxMapper
        #[test]
        fn test_with_box_mapper() {
            let mapper = BoxMapper::new(|x: i32| x * 6);
            let source = RcSupplier::new(|| 10);
            let mut supplier = source.map(mapper);
            assert_eq!(supplier.get(), 60);
        }

        // Test chaining with different mapper types
        #[test]
        fn test_chain_with_mapper_and_closure() {
            let mapper = RcMapper::new(|x: i32| x * 2);
            let source = RcSupplier::new(|| 5);
            let chained = source.map(mapper);
            let mut final_supplier = chained.map(|x| x + 10);
            assert_eq!(final_supplier.get(), 20); // (5 * 2) + 10
        }

        // Test shared state with cloned suppliers
        #[test]
        fn test_shared_state_with_mapper() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let source = RcSupplier::new(move || {
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
            let source = RcSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });
            let filtered = source.filter(|x| x % 2 == 0);

            let mut s = filtered;
            assert_eq!(s.get(), None); // 1 is odd
            assert_eq!(s.get(), Some(2)); // 2 is even
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_combines_two_suppliers() {
            let first = RcSupplier::new(|| 42);
            let second = RcSupplier::new(|| "hello");
            let zipped = first.zip(&second);

            let mut z = zipped;
            assert_eq!(z.get(), (42, "hello"));
        }

        #[test]
        fn test_originals_remain_usable() {
            let first = RcSupplier::new(|| 42);
            let second = RcSupplier::new(|| "hello");
            let _zipped = first.zip(&second);

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
            let source = RcSupplier::new(move || {
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
            let supplier = RcSupplier::new(|| 42);
            let mut boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_into_rc {
        use super::*;

        #[test]
        fn test_returns_self() {
            let supplier = RcSupplier::new(|| 42);
            let mut rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }
    }

    // Note: RcSupplier cannot be converted to ArcSupplier because
    // Rc is not Send. This is prevented at compile time by the
    // trait bound, so we don't test it.
}
