/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for Supplier types

use prism3_function::{ArcSupplier, BoxSupplier, FnSupplierOps, RcSupplier, Supplier};

// ==========================================================================
// BoxSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_box_supplier {
    use super::*;

    #[test]
    fn test_new() {
        let mut supplier = BoxSupplier::new(|| 42);
        assert_eq!(supplier.get(), 42);
    }

    #[test]
    fn test_with_different_types() {
        // i32
        let mut int_supplier = BoxSupplier::new(|| 42);
        assert_eq!(int_supplier.get(), 42);

        // String
        let mut string_supplier = BoxSupplier::new(|| String::from("hello"));
        assert_eq!(string_supplier.get(), "hello");

        // Vec
        let mut vec_supplier = BoxSupplier::new(|| vec![1, 2, 3]);
        assert_eq!(vec_supplier.get(), vec![1, 2, 3]);

        // bool
        let mut bool_supplier = BoxSupplier::new(|| true);
        assert_eq!(bool_supplier.get(), true);
    }

    #[test]
    fn test_constant() {
        let mut constant = BoxSupplier::constant(42);
        assert_eq!(constant.get(), 42);
        assert_eq!(constant.get(), 42);
        assert_eq!(constant.get(), 42);
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

    #[test]
    fn test_map() {
        let mut mapped = BoxSupplier::new(|| 10).map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_multiple_chains() {
        let mut chained = BoxSupplier::new(|| 5).map(|x| x * 2).map(|x| x + 5);

        assert_eq!(chained.get(), 15); // (5 * 2) + 5
    }

    #[test]
    fn test_map_type_conversion() {
        let mut converted = BoxSupplier::new(|| 42).map(|x| x.to_string());
        assert_eq!(converted.get(), "42");
    }

    #[test]
    fn test_filter() {
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
    fn test_zip() {
        let first = BoxSupplier::new(|| 42);
        let second = BoxSupplier::new(|| "hello");
        let mut zipped = first.zip(second);

        assert_eq!(zipped.get(), (42, "hello"));
    }

    #[test]
    fn test_zip_with_stateful() {
        let mut count1 = 0;
        let mut count2 = 10;
        let first = BoxSupplier::new(move || {
            count1 += 1;
            count1
        });
        let second = BoxSupplier::new(move || {
            count2 += 10;
            count2
        });
        let mut zipped = first.zip(second);

        assert_eq!(zipped.get(), (1, 20));
        assert_eq!(zipped.get(), (2, 30));
    }

    #[test]
    fn test_memoize() {
        let mut _call_count = 0;
        let mut memoized = BoxSupplier::new(move || {
            _call_count += 1;
            42
        })
        .memoize();

        assert_eq!(memoized.get(), 42); // Calls underlying function
        assert_eq!(memoized.get(), 42); // Returns cached value
        assert_eq!(memoized.get(), 42); // Returns cached value
    }

    #[test]
    fn test_lazy() {
        let mut _factory_call_count = 0;
        let mut _supplier_call_count = 0;

        let mut lazy = BoxSupplier::lazy(move || {
            _factory_call_count += 1;
            BoxSupplier::new(move || {
                _supplier_call_count += 1;
                42
            })
        });

        // Factory hasn't been called yet
        assert_eq!(lazy.get(), 42); // Factory called, then Supplier called
        assert_eq!(lazy.get(), 42); // Only Supplier called
    }

    #[test]
    fn test_alternate() {
        let supplier1 = BoxSupplier::new(|| 1);
        let supplier2 = BoxSupplier::new(|| 2);
        let mut alternating = BoxSupplier::alternate(supplier1, supplier2);

        assert_eq!(alternating.get(), 1);
        assert_eq!(alternating.get(), 2);
        assert_eq!(alternating.get(), 1);
        assert_eq!(alternating.get(), 2);
    }

    #[test]
    fn test_or_else() {
        let primary = BoxSupplier::new(|| None);
        let mut supplier = primary.or_else(|| Some(42));
        assert_eq!(supplier.get(), Some(42));
    }

    #[test]
    fn test_or_else_with_some() {
        let primary = BoxSupplier::new(|| Some(10));
        let mut supplier = primary.or_else(|| Some(999));
        assert_eq!(supplier.get(), Some(10)); // Uses primary
    }

    #[test]
    fn test_or_else_supplier() {
        let primary = BoxSupplier::new(|| None);
        let fallback = BoxSupplier::new(|| Some(42));
        let mut supplier = primary.or_else_supplier(fallback);
        assert_eq!(supplier.get(), Some(42));
    }

    #[test]
    fn test_into_box() {
        let supplier = BoxSupplier::new(|| 42);
        let mut boxed = supplier.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_rc() {
        let supplier = BoxSupplier::new(|| 42);
        let mut rc = supplier.into_rc();
        assert_eq!(rc.get(), 42);
    }

    // Note: BoxSupplier cannot be safely converted to ArcSupplier
    // because the inner function may not be Send.
    // This test has been removed.
}

// ==========================================================================
// ArcSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_arc_supplier {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_new() {
        let supplier = ArcSupplier::new(|| 42);
        let mut s = supplier;
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcSupplier::constant(42);
        let mut s = constant;
        assert_eq!(s.get(), 42);
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_clone() {
        let supplier = ArcSupplier::new(|| 42);
        let clone1 = supplier.clone();
        let clone2 = supplier.clone();

        let mut s1 = clone1;
        assert_eq!(s1.get(), 42);

        let mut s2 = clone2;
        assert_eq!(s2.get(), 42);
    }

    #[test]
    fn test_stateful_shared() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        let mut s1 = supplier.clone();
        assert_eq!(s1.get(), 1);

        let mut s2 = supplier.clone();
        assert_eq!(s2.get(), 2);

        // Verify counter value
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_map() {
        let source = ArcSupplier::new(|| 10);
        let mapped = source.map(|x| x * 2);

        // source is still usable
        let mut s = mapped;
        assert_eq!(s.get(), 20);
    }

    #[test]
    fn test_map_multiple_chains() {
        let source = ArcSupplier::new(|| 5);
        let mapped = source.map(|x| x * 2).map(|x| x + 5);

        let mut s = mapped;
        assert_eq!(s.get(), 15);
    }

    #[test]
    fn test_filter() {
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
        assert_eq!(s.get(), None); // 3 is odd
        assert_eq!(s.get(), Some(4)); // 4 is even
    }

    #[test]
    fn test_zip() {
        let first = ArcSupplier::new(|| 42);
        let second = ArcSupplier::new(|| "hello");
        let zipped = first.zip(&second);

        // first and second are still usable
        let mut z = zipped;
        assert_eq!(z.get(), (42, "hello"));
    }

    #[test]
    fn test_memoize() {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = Arc::clone(&call_count);
        let source = ArcSupplier::new(move || {
            let mut c = call_count_clone.lock().unwrap();
            *c += 1;
            42
        });
        let memoized = source.memoize();

        let mut s = memoized;
        assert_eq!(s.get(), 42); // Calls underlying function
        assert_eq!(s.get(), 42); // Returns cached value
        assert_eq!(*call_count.lock().unwrap(), 1); // Only called once
    }

    #[test]
    fn test_thread_safety() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        let clone = supplier.clone();

        let handle = thread::spawn(move || {
            let mut s = clone;
            s.get()
        });

        let mut s = supplier;
        let value1 = s.get();

        let value2 = handle.join().unwrap();

        // Both threads should get different values
        assert!(value1 == 1 || value1 == 2);
        assert!(value2 == 1 || value2 == 2);
        assert_ne!(value1, value2);
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_or_else() {
        let primary = ArcSupplier::new(|| None);
        let supplier = primary.or_else(|| Some(42));
        let mut s = supplier;
        assert_eq!(s.get(), Some(42));
    }

    #[test]
    fn test_or_else_supplier() {
        let primary = ArcSupplier::new(|| None);
        let fallback = ArcSupplier::new(|| Some(42));
        let supplier = primary.or_else_supplier(&fallback);
        let mut s = supplier;
        assert_eq!(s.get(), Some(42));
    }

    #[test]
    fn test_into_box() {
        let supplier = ArcSupplier::new(|| 42);
        let mut boxed = supplier.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_rc() {
        let supplier = ArcSupplier::new(|| 42);
        let mut rc = supplier.into_rc();
        assert_eq!(rc.get(), 42);
    }

    #[test]
    fn test_into_arc() {
        let supplier = ArcSupplier::new(|| 42);
        let mut arc = supplier.into_arc();
        assert_eq!(arc.get(), 42);
    }
}

// ==========================================================================
// RcSupplier Tests
// ==========================================================================

#[cfg(test)]
mod test_rc_supplier {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_new() {
        let supplier = RcSupplier::new(|| 42);
        let mut s = supplier;
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcSupplier::constant(42);
        let mut s = constant;
        assert_eq!(s.get(), 42);
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_clone() {
        let supplier = RcSupplier::new(|| 42);
        let clone1 = supplier.clone();
        let clone2 = supplier.clone();

        let mut s1 = clone1;
        assert_eq!(s1.get(), 42);

        let mut s2 = clone2;
        assert_eq!(s2.get(), 42);
    }

    #[test]
    fn test_stateful_shared() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });

        let mut s1 = supplier.clone();
        assert_eq!(s1.get(), 1);

        let mut s2 = supplier.clone();
        assert_eq!(s2.get(), 2);

        // Verify counter value
        assert_eq!(*counter.borrow(), 2);
    }

    #[test]
    fn test_map() {
        let source = RcSupplier::new(|| 10);
        let mapped = source.map(|x| x * 2);

        // source is still usable
        let mut s = mapped;
        assert_eq!(s.get(), 20);
    }

    #[test]
    fn test_map_multiple_chains() {
        let source = RcSupplier::new(|| 5);
        let mapped = source.map(|x| x * 2).map(|x| x + 5);

        let mut s = mapped;
        assert_eq!(s.get(), 15);
    }

    #[test]
    fn test_filter() {
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
        assert_eq!(s.get(), None); // 3 is odd
        assert_eq!(s.get(), Some(4)); // 4 is even
    }

    #[test]
    fn test_zip() {
        let first = RcSupplier::new(|| 42);
        let second = RcSupplier::new(|| "hello");
        let zipped = first.zip(&second);

        // first and second are still usable
        let mut z = zipped;
        assert_eq!(z.get(), (42, "hello"));
    }

    #[test]
    fn test_memoize() {
        let call_count = Rc::new(RefCell::new(0));
        let call_count_clone = Rc::clone(&call_count);
        let source = RcSupplier::new(move || {
            let mut c = call_count_clone.borrow_mut();
            *c += 1;
            42
        });
        let memoized = source.memoize();

        let mut s = memoized;
        assert_eq!(s.get(), 42); // Calls underlying function
        assert_eq!(s.get(), 42); // Returns cached value
        assert_eq!(*call_count.borrow(), 1); // Only called once
    }

    #[test]
    fn test_or_else() {
        let primary = RcSupplier::new(|| None);
        let supplier = primary.or_else(|| Some(42));
        let mut s = supplier;
        assert_eq!(s.get(), Some(42));
    }

    #[test]
    fn test_or_else_supplier() {
        let primary = RcSupplier::new(|| None);
        let fallback = RcSupplier::new(|| Some(42));
        let supplier = primary.or_else_supplier(&fallback);
        let mut s = supplier;
        assert_eq!(s.get(), Some(42));
    }

    #[test]
    fn test_into_box() {
        let supplier = RcSupplier::new(|| 42);
        let mut boxed = supplier.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_rc() {
        let supplier = RcSupplier::new(|| 42);
        let mut rc = supplier.into_rc();
        assert_eq!(rc.get(), 42);
    }

    // Note: RcSupplier cannot be converted to ArcSupplier because
    // it's not Send. This test has been removed.
}

// ==========================================================================
// Consumer Trait Tests
// ==========================================================================

#[cfg(test)]
mod test_supplier_trait {
    use super::*;

    #[test]
    fn test_trait_object_box() {
        fn use_any_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
            supplier.get()
        }

        let mut box_sup = BoxSupplier::new(|| 42);
        assert_eq!(use_any_supplier(&mut box_sup), 42);
    }

    #[test]
    fn test_trait_object_arc() {
        fn use_any_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
            supplier.get()
        }

        let arc_sup = ArcSupplier::new(|| 42);
        let mut s = arc_sup;
        assert_eq!(use_any_supplier(&mut s), 42);
    }

    #[test]
    fn test_trait_object_rc() {
        fn use_any_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
            supplier.get()
        }

        let rc_sup = RcSupplier::new(|| 42);
        let mut s = rc_sup;
        assert_eq!(use_any_supplier(&mut s), 42);
    }

    #[test]
    fn test_closure_implements_supplier() {
        fn use_any_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
            supplier.get()
        }

        let mut closure = || 42;
        assert_eq!(use_any_supplier(&mut closure), 42);
    }

    #[test]
    fn test_closure_into_box() {
        let closure = || 42;
        let mut box_supplier = closure.into_box();
        assert_eq!(box_supplier.get(), 42);
    }

    #[test]
    fn test_closure_into_rc() {
        let closure = || 42;
        let mut rc_supplier = closure.into_rc();
        assert_eq!(rc_supplier.get(), 42);
    }

    #[test]
    fn test_closure_into_arc() {
        let closure = || 42;
        let mut arc_supplier = closure.into_arc();
        assert_eq!(arc_supplier.get(), 42);
    }
}

// ==========================================================================
// FnSupplierOps Tests
// ==========================================================================

#[cfg(test)]
mod test_fn_supplier_ops {
    use super::*;

    #[test]
    fn test_closure_map() {
        let mapped = (|| 10).map(|x| x * 2);
        let mut result = mapped;
        assert_eq!(result.get(), 20);
    }

    #[test]
    fn test_closure_map_chain() {
        let mapped = (|| 5).map(|x| x * 2).map(|x| x + 5);
        let mut result = mapped;
        assert_eq!(result.get(), 15);
    }

    #[test]
    fn test_closure_filter() {
        let mut counter = 0;
        let filtered = (move || {
            counter += 1;
            counter
        })
        .filter(|x| x % 2 == 0);

        let mut result = filtered;
        assert_eq!(result.get(), None); // 1 is odd
        assert_eq!(result.get(), Some(2)); // 2 is even
    }
}

// ==========================================================================
// into_fn Tests
// ==========================================================================

#[cfg(test)]
mod test_into_fn {
    use super::*;
    use std::iter::repeat_with;

    // BoxSupplier into_fn tests
    #[test]
    fn test_box_supplier_into_fn_basic() {
        let supplier = BoxSupplier::new(|| 42);
        let mut func = supplier.into_fn();
        assert_eq!(func(), 42);
        assert_eq!(func(), 42);
    }

    #[test]
    fn test_box_supplier_into_fn_stateful() {
        let mut counter = 0;
        let supplier = BoxSupplier::new(move || {
            counter += 1;
            counter
        });
        let mut func = supplier.into_fn();

        assert_eq!(func(), 1);
        assert_eq!(func(), 2);
        assert_eq!(func(), 3);
    }

    #[test]
    fn test_box_supplier_into_fn_with_repeat_with() {
        let mut counter = 0;
        let supplier = BoxSupplier::new(move || {
            counter += 1;
            counter
        });

        let values: Vec<i32> = repeat_with(supplier.into_fn()).take(5).collect();

        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_box_supplier_into_fn_with_custom_function() {
        fn call_n_times<F>(mut func: F, n: usize) -> Vec<i32>
        where
            F: FnMut() -> i32,
        {
            (0..n).map(|_| func()).collect()
        }

        let mut counter = 0;
        let supplier = BoxSupplier::new(move || {
            counter += 1;
            counter
        });

        let result = call_n_times(supplier.into_fn(), 5);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    // ArcSupplier into_fn tests
    #[test]
    fn test_arc_supplier_into_fn_basic() {
        let supplier = ArcSupplier::new(|| 42);
        let mut func = supplier.into_fn();
        assert_eq!(func(), 42);
        assert_eq!(func(), 42);
    }

    #[test]
    fn test_arc_supplier_into_fn_stateful() {
        use std::sync::{Arc, Mutex};

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        let mut func = supplier.into_fn();
        assert_eq!(func(), 1);
        assert_eq!(func(), 2);
        assert_eq!(func(), 3);
    }

    #[test]
    fn test_arc_supplier_into_fn_with_repeat_with() {
        use std::sync::{Arc, Mutex};

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        let values: Vec<i32> = repeat_with(supplier.into_fn()).take(5).collect();

        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    // RcSupplier into_fn tests
    #[test]
    fn test_rc_supplier_into_fn_basic() {
        let supplier = RcSupplier::new(|| 42);
        let mut func = supplier.into_fn();
        assert_eq!(func(), 42);
        assert_eq!(func(), 42);
    }

    #[test]
    fn test_rc_supplier_into_fn_stateful() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });

        let mut func = supplier.into_fn();
        assert_eq!(func(), 1);
        assert_eq!(func(), 2);
        assert_eq!(func(), 3);
    }

    #[test]
    fn test_rc_supplier_into_fn_with_repeat_with() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });

        let values: Vec<i32> = repeat_with(supplier.into_fn()).take(5).collect();

        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    // Closure into_fn tests
    #[test]
    fn test_closure_into_fn_basic() {
        let closure = || 42;
        let mut func = closure.into_fn();
        assert_eq!(func(), 42);
        assert_eq!(func(), 42);
    }

    #[test]
    fn test_closure_into_fn_stateful() {
        let mut counter = 0;
        let closure = move || {
            counter += 1;
            counter
        };
        let mut func = closure.into_fn();

        assert_eq!(func(), 1);
        assert_eq!(func(), 2);
        assert_eq!(func(), 3);
    }

    #[test]
    fn test_closure_into_fn_with_repeat_with() {
        let mut counter = 0;
        let closure = move || {
            counter += 1;
            counter
        };

        let values: Vec<i32> = repeat_with(closure.into_fn()).take(5).collect();

        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    // Test different types
    #[test]
    fn test_into_fn_different_types() {
        // String
        let supplier = BoxSupplier::new(|| String::from("hello"));
        let mut func = supplier.into_fn();
        assert_eq!(func(), "hello");

        // Vec
        let supplier = BoxSupplier::new(|| vec![1, 2, 3]);
        let mut func = supplier.into_fn();
        assert_eq!(func(), vec![1, 2, 3]);

        // bool
        let supplier = BoxSupplier::new(|| true);
        let mut func = supplier.into_fn();
        assert_eq!(func(), true);
    }

    // Test with mapped suppliers
    #[test]
    fn test_into_fn_with_map() {
        let supplier = BoxSupplier::new(|| 10).map(|x| x * 2);
        let mut func = supplier.into_fn();
        assert_eq!(func(), 20);
    }

    #[test]
    fn test_into_fn_with_filter() {
        let mut counter = 0;
        let supplier = BoxSupplier::new(move || {
            counter += 1;
            counter
        })
        .filter(|x| x % 2 == 0);

        let mut func = supplier.into_fn();
        assert_eq!(func(), None); // 1 is odd
        assert_eq!(func(), Some(2)); // 2 is even
        assert_eq!(func(), None); // 3 is odd
        assert_eq!(func(), Some(4)); // 4 is even
    }

    // Test ownership behavior
    #[test]
    fn test_into_fn_consumes_supplier() {
        let supplier = BoxSupplier::new(|| 42);
        let _func = supplier.into_fn();
        // supplier is no longer usable here (moved)
    }

    // Test clone before into_fn for ArcSupplier
    #[test]
    fn test_arc_supplier_clone_before_into_fn() {
        use std::sync::{Arc, Mutex};

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        // Clone before conversion to keep the original
        let mut func = supplier.clone().into_fn();
        assert_eq!(func(), 1);
        assert_eq!(func(), 2);
        assert_eq!(func(), 3);

        // Original supplier is still usable and shares the same state
        let mut s = supplier;
        assert_eq!(s.get(), 4); // Counter continues from where it left off
        assert_eq!(s.get(), 5);
    }

    // Test clone before into_fn for RcSupplier
    #[test]
    fn test_rc_supplier_clone_before_into_fn() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });

        // Clone before conversion to keep the original
        let mut func = supplier.clone().into_fn();
        assert_eq!(func(), 1);
        assert_eq!(func(), 2);
        assert_eq!(func(), 3);

        // Original supplier is still usable and shares the same state
        let mut s = supplier;
        assert_eq!(s.get(), 4); // Counter continues from where it left off
        assert_eq!(s.get(), 5);
    }

    // Test multiple clones with into_fn for ArcSupplier
    #[test]
    fn test_arc_supplier_multiple_clones_with_into_fn() {
        use std::sync::{Arc, Mutex};

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        // Create multiple functions from clones
        let mut func1 = supplier.clone().into_fn();
        let mut func2 = supplier.clone().into_fn();
        let mut func3 = supplier.clone().into_fn();

        // All functions share the same counter state
        assert_eq!(func1(), 1);
        assert_eq!(func2(), 2);
        assert_eq!(func3(), 3);
        assert_eq!(func1(), 4);

        // Original supplier is still usable
        let mut s = supplier;
        assert_eq!(s.get(), 5);
    }

    // Test multiple clones with into_fn for RcSupplier
    #[test]
    fn test_rc_supplier_multiple_clones_with_into_fn() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });

        // Create multiple functions from clones
        let mut func1 = supplier.clone().into_fn();
        let mut func2 = supplier.clone().into_fn();
        let mut func3 = supplier.clone().into_fn();

        // All functions share the same counter state
        assert_eq!(func1(), 1);
        assert_eq!(func2(), 2);
        assert_eq!(func3(), 3);
        assert_eq!(func1(), 4);

        // Original supplier is still usable
        let mut s = supplier;
        assert_eq!(s.get(), 5);
    }
}
