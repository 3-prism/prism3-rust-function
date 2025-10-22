/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for SupplierOnce types

use prism3_function::{ArcSupplier, BoxSupplier, BoxSupplierOnce, RcSupplier, SupplierOnce};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

// ==========================================================================
// SupplierOnce Trait Tests (for closures)
// ==========================================================================

#[cfg(test)]
mod test_supplier_once_trait {
    use super::*;

    #[test]
    fn test_closure_implements_supplier_once() {
        let closure = || 42;
        let boxed = closure.into_box_once();
        assert_eq!(boxed.get_once(), 42);
    }

    #[test]
    fn test_closure_move_capture() {
        let data = String::from("hello");
        let closure = move || data;
        let boxed = closure.into_box_once();
        assert_eq!(boxed.get_once(), "hello");
    }

    #[test]
    fn test_into_box_once() {
        let closure = || 42;
        let boxed = closure.into_box_once();
        assert_eq!(boxed.get_once(), 42);
    }

    #[test]
    fn test_closure_get_direct() {
        let closure = || 42;
        assert_eq!(closure.get_once(), 42);
    }

    #[test]
    fn test_closure_get_with_move() {
        let data = String::from("hello");
        let closure = move || data;
        assert_eq!(closure.get_once(), "hello");
    }

    #[test]
    fn test_closure_get_with_complex_type() {
        let closure = || vec![1, 2, 3];
        assert_eq!(closure.get_once(), vec![1, 2, 3]);
    }

    #[test]
    fn test_into_fn() {
        let closure = || 42;
        let fn_once = closure.into_fn_once();
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_into_fn_with_move() {
        let data = String::from("hello");
        let closure = move || data;
        let fn_once = closure.into_fn_once();
        assert_eq!(fn_once(), "hello");
    }

    #[test]
    fn test_into_fn_with_vec() {
        let closure = || vec![1, 2, 3];
        let fn_once = closure.into_fn_once();
        assert_eq!(fn_once(), vec![1, 2, 3]);
    }

    #[test]
    fn test_into_fn_with_complex_computation() {
        let closure = || {
            let x = 10;
            let y = 32;
            x + y
        };
        let fn_once = closure.into_fn_once();
        assert_eq!(fn_once(), 42);
    }
}

// ==========================================================================
// BoxSupplierOnce Tests
// ==========================================================================

#[cfg(test)]
mod test_box_supplier_once {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_creates_supplier() {
            let once = BoxSupplierOnce::new(|| 42);
            assert_eq!(once.get_once(), 42);
        }

        #[test]
        fn test_with_string() {
            let once = BoxSupplierOnce::new(|| String::from("hello"));
            assert_eq!(once.get_once(), "hello");
        }

        #[test]
        fn test_with_vec() {
            let once = BoxSupplierOnce::new(|| vec![1, 2, 3]);
            assert_eq!(once.get_once(), vec![1, 2, 3]);
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn test_consumes_supplier() {
            let once = BoxSupplierOnce::new(|| 42);
            let value = once.get_once();
            assert_eq!(value, 42);
            // once is consumed here
        }

        #[test]
        fn test_with_move_closure() {
            let data = String::from("hello");
            let once = BoxSupplierOnce::new(move || data);
            assert_eq!(once.get_once(), "hello");
        }

        #[test]
        fn test_with_expensive_computation() {
            let once = BoxSupplierOnce::new(move || {
                // Expensive computation
                42
            });
            assert_eq!(once.get_once(), 42);
        }

        #[test]
        fn test_moves_captured_value() {
            let resource = vec![1, 2, 3];
            let once = BoxSupplierOnce::new(move || resource);
            let result = once.get_once();
            assert_eq!(result, vec![1, 2, 3]);
        }
    }

    mod test_into_box {
        use super::*;

        #[test]
        fn test_returns_self() {
            let once = BoxSupplierOnce::new(|| 42);
            let boxed = once.into_box_once();
            assert_eq!(boxed.get_once(), 42);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_basic_conversion() {
            let once = BoxSupplierOnce::new(|| 42);
            let fn_once = once.into_fn_once();
            assert_eq!(fn_once(), 42);
        }

        #[test]
        fn test_with_string() {
            let once = BoxSupplierOnce::new(|| String::from("hello"));
            let fn_once = once.into_fn_once();
            assert_eq!(fn_once(), "hello");
        }

        #[test]
        fn test_with_move_closure() {
            let data = String::from("captured");
            let once = BoxSupplierOnce::new(move || data);
            let fn_once = once.into_fn_once();
            assert_eq!(fn_once(), "captured");
        }

        #[test]
        fn test_with_vec() {
            let once = BoxSupplierOnce::new(|| vec![1, 2, 3]);
            let fn_once = once.into_fn_once();
            assert_eq!(fn_once(), vec![1, 2, 3]);
        }
    }

    mod test_use_cases {
        use super::*;

        #[test]
        fn test_lazy_initialization() {
            let once = BoxSupplierOnce::new(|| {
                // Simulating expensive initialization
                std::thread::sleep(std::time::Duration::from_millis(1));
                42
            });

            // Initialization only happens when get() is called
            let value = once.get_once();
            assert_eq!(value, 42);
        }

        #[test]
        fn test_resource_consumption() {
            struct Resource {
                data: String,
            }

            let resource = Resource {
                data: String::from("important data"),
            };

            let once = BoxSupplierOnce::new(move || {
                // Consume the resource
                resource.data
            });

            let result = once.get_once();
            assert_eq!(result, "important data");
        }

        #[test]
        fn test_with_non_cloneable_type() {
            use std::rc::Rc;

            let data = Rc::new(vec![1, 2, 3]);
            let once = BoxSupplierOnce::new(move || data);

            let result = once.get_once();
            assert_eq!(*result, vec![1, 2, 3]);
        }
    }

    mod test_into_box_conversion {
        use super::*;

        #[test]
        fn test_returns_self() {
            let once = BoxSupplierOnce::new(|| 42);
            let boxed = once.into_box_once();
            assert_eq!(boxed.get_once(), 42);
        }

        #[test]
        fn test_closure_into_box() {
            let closure = || 42;
            let boxed = closure.into_box_once();
            assert_eq!(boxed.get_once(), 42);
        }

        #[test]
        fn test_closure_with_move() {
            let data = String::from("hello");
            let closure = move || data;
            let boxed = closure.into_box_once();
            assert_eq!(boxed.get_once(), "hello");
        }
    }

    mod test_edge_cases {
        use super::*;

        #[test]
        fn test_with_unit_type() {
            let once = BoxSupplierOnce::new(|| ());
            once.get_once();
            // Unit type always succeeds, no assertion needed
        }

        #[test]
        fn test_with_tuple() {
            let once = BoxSupplierOnce::new(|| (1, "hello", true));
            assert_eq!(once.get_once(), (1, "hello", true));
        }

        #[test]
        fn test_with_option_some() {
            let once = BoxSupplierOnce::new(|| Some(42));
            assert_eq!(once.get_once(), Some(42));
        }

        #[test]
        fn test_with_option_none() {
            let once = BoxSupplierOnce::new(|| None::<i32>);
            assert_eq!(once.get_once(), None);
        }

        #[test]
        fn test_with_result_ok() {
            let once = BoxSupplierOnce::new(|| Ok::<i32, String>(42));
            assert_eq!(once.get_once(), Ok(42));
        }

        #[test]
        fn test_with_result_err() {
            let once = BoxSupplierOnce::new(|| Err::<i32, String>(String::from("error")));
            assert_eq!(once.get_once(), Err(String::from("error")));
        }
    }
}

// ==========================================================================
// Test Custom Type with Default into_box Implementation
// ==========================================================================

#[cfg(test)]
mod test_custom_supplier_once_default_implementation {
    use super::*;

    // A custom type that implements SupplierOnce by only providing
    // the core get() method. The into_box() method will use
    // the default implementation from the trait.
    struct CustomSupplierOnce<T> {
        value: Option<T>,
    }

    impl<T> CustomSupplierOnce<T> {
        fn new(value: T) -> Self {
            CustomSupplierOnce { value: Some(value) }
        }
    }

    impl<T> SupplierOnce<T> for CustomSupplierOnce<T> {
        fn get_once(mut self) -> T {
            self.value
                .take()
                .expect("CustomSupplierOnce already consumed")
        }
        // Note: into_box() is NOT implemented here, so the
        // default implementation from the trait will be used
    }

    #[test]
    fn test_custom_type_get_method() {
        let custom = CustomSupplierOnce::new(42);
        assert_eq!(custom.get_once(), 42);
    }

    #[test]
    fn test_custom_type_into_box_default_impl() {
        let custom = CustomSupplierOnce::new(42);
        let boxed = custom.into_box_once();
        assert_eq!(boxed.get_once(), 42);
    }

    #[test]
    fn test_custom_type_with_string() {
        let custom = CustomSupplierOnce::new(String::from("hello"));
        let boxed = custom.into_box_once();
        assert_eq!(boxed.get_once(), "hello");
    }

    #[test]
    fn test_custom_type_with_vec() {
        let custom = CustomSupplierOnce::new(vec![1, 2, 3]);
        let boxed = custom.into_box_once();
        assert_eq!(boxed.get_once(), vec![1, 2, 3]);
    }

    #[test]
    fn test_custom_type_with_complex_type() {
        struct Data {
            id: i32,
            name: String,
        }

        let data = Data {
            id: 1,
            name: String::from("test"),
        };
        let custom = CustomSupplierOnce::new(data);
        let boxed = custom.into_box_once();
        let result = boxed.get_once();
        assert_eq!(result.id, 1);
        assert_eq!(result.name, "test");
    }

    #[test]
    fn test_custom_type_with_option() {
        let custom = CustomSupplierOnce::new(Some(42));
        let boxed = custom.into_box_once();
        assert_eq!(boxed.get_once(), Some(42));
    }

    #[test]
    fn test_custom_type_with_result() {
        let custom = CustomSupplierOnce::new(Ok::<i32, String>(42));
        let boxed = custom.into_box_once();
        assert_eq!(boxed.get_once(), Ok(42));
    }

    #[test]
    fn test_custom_type_into_fn_default_impl() {
        let custom = CustomSupplierOnce::new(42);
        let fn_once = custom.into_fn_once();
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_custom_type_into_fn_with_string() {
        let custom = CustomSupplierOnce::new(String::from("hello"));
        let fn_once = custom.into_fn_once();
        assert_eq!(fn_once(), "hello");
    }

    #[test]
    fn test_custom_type_into_fn_with_vec() {
        let custom = CustomSupplierOnce::new(vec![1, 2, 3]);
        let fn_once = custom.into_fn_once();
        assert_eq!(fn_once(), vec![1, 2, 3]);
    }
}

// ==========================================================================
// Tests for to_box and to_fn
// ==========================================================================

#[cfg(test)]
mod test_to_box_and_to_fn {
    use super::*;
    use std::sync::{Arc, Mutex};

    // A custom cloneable supplier to test the default `to_box` and `to_fn`
    // implementations.
    #[derive(Clone)]
    struct CloneableSupplier {
        value: Arc<Mutex<Option<i32>>>,
    }

    impl SupplierOnce<i32> for CloneableSupplier {
        fn get_once(self) -> i32 {
            self.value
                .lock()
                .unwrap()
                .take()
                .expect("CloneableSupplier already consumed")
        }
    }

    #[test]
    fn test_default_to_fn_with_custom_cloneable_supplier() {
        let supplier = CloneableSupplier {
            value: Arc::new(Mutex::new(Some(42))),
        };
        let fn_once = supplier.to_fn_once();
        // The original supplier is not consumed
        assert!(supplier.value.lock().unwrap().is_some());
        // The returned FnOnce can be called
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_default_to_box_with_custom_cloneable_supplier() {
        let supplier = CloneableSupplier {
            value: Arc::new(Mutex::new(Some(42))),
        };
        let boxed = supplier.to_box_once();
        // The original supplier is not consumed
        assert!(supplier.value.lock().unwrap().is_some());
        // The returned BoxSupplierOnce can be consumed
        assert_eq!(boxed.get_once(), 42);
    }

    #[test]
    fn test_specialized_to_fn_for_cloneable_closure() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let closure = move || {
            *counter_clone.lock().unwrap() += 1;
            42
        };
        let fn_once = closure.to_fn_once();
        fn_once();
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_specialized_to_box_for_cloneable_closure() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let closure = move || {
            *counter_clone.lock().unwrap() += 1;
            42
        };
        let boxed = closure.to_box_once();
        boxed.get_once();
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_box_supplier_once_cannot_use_to_box() {
        // This test demonstrates that `to_box` cannot be called on a
        // `BoxSupplierOnce` because it does not implement `Clone`.
        // The following code will fail to compile, which is the expected
        // behavior.

        // let once = BoxSupplierOnce::new(|| 42);
        // let _boxed = once.to_box_once(); // COMPILE ERROR: `BoxSupplierOnce` is not `Clone`
    }

    #[test]
    fn test_box_supplier_once_cannot_use_to_fn() {
        // Similar to `to_box`, `to_fn` cannot be called on a `BoxSupplierOnce`
        // because of the `Clone` trait bound. The following code is commented
        // out because it would prevent the test suite from compiling.

        // let once = BoxSupplierOnce::new(|| 42);
        // let _fn_once = once.to_fn_once(); // COMPILE ERROR: `BoxSupplierOnce` is not `Clone`
    }

    #[test]
    fn test_non_cloneable_closure_cannot_use_to_box() {
        // A closure that moves a non-cloneable value cannot be cloned.
        // Therefore, `to_box` and `to_fn` cannot be called on it.
        struct NonCloneable(i32);
        let data = NonCloneable(42);
        let _closure = move || data.0;

        // The following lines would fail to compile because the closure is not
        // `Clone`.
        // let _boxed = _closure.to_box_once();
        // let _fn_once = _closure.to_fn_once();
    }
}

// ==========================================================================
// BoxSupplier SupplierOnce Implementation Tests
// ==========================================================================

#[cfg(test)]
mod test_box_supplier_supplier_once {
    use super::*;

    #[test]
    fn test_get_consumes_supplier() {
        let supplier = BoxSupplier::new(|| 42);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 42);
        // supplier is consumed here
    }

    #[test]
    fn test_get_with_string() {
        let supplier = BoxSupplier::new(|| String::from("hello"));
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_get_with_vec() {
        let supplier = BoxSupplier::new(|| vec![1, 2, 3]);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, vec![1, 2, 3]);
    }

    #[test]
    fn test_get_with_stateful_closure() {
        let mut counter = 0;
        let supplier = BoxSupplier::new(move || {
            counter += 1;
            counter
        });
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_into_box_once() {
        let supplier = BoxSupplier::new(|| 42);
        let once = SupplierOnce::into_box_once(supplier);
        assert_eq!(once.get_once(), 42);
    }

    #[test]
    fn test_into_box_once_with_string() {
        let supplier = BoxSupplier::new(|| String::from("hello"));
        let once = SupplierOnce::into_box_once(supplier);
        assert_eq!(once.get_once(), "hello");
    }

    #[test]
    fn test_into_fn_once() {
        let supplier = BoxSupplier::new(|| 42);
        let fn_once = SupplierOnce::into_fn_once(supplier);
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_into_fn_once_with_string() {
        let supplier = BoxSupplier::new(|| String::from("hello"));
        let fn_once = SupplierOnce::into_fn_once(supplier);
        assert_eq!(fn_once(), "hello");
    }

    #[test]
    fn test_into_fn_once_with_move() {
        let data = String::from("captured");
        let supplier = BoxSupplier::new(move || data.clone());
        let fn_once = SupplierOnce::into_fn_once(supplier);
        assert_eq!(fn_once(), "captured");
    }

    #[test]
    fn test_with_constant_supplier() {
        let supplier = BoxSupplier::constant(42);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_mapped_supplier() {
        let supplier = BoxSupplier::new(|| 10).map(|x| x * 2);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_with_filtered_supplier() {
        let mut counter = 0;
        let supplier = BoxSupplier::new(move || {
            counter += 1;
            counter
        })
        .filter(|x| x % 2 == 0);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, None);
    }

    #[test]
    fn test_with_memoized_supplier() {
        let supplier = BoxSupplier::new(|| 42).memoize();
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 42);
    }
}

// ==========================================================================
// ArcSupplier SupplierOnce Implementation Tests
// ==========================================================================

#[cfg(test)]
mod test_arc_supplier_supplier_once {
    use super::*;

    #[test]
    fn test_get_consumes_supplier() {
        let supplier = ArcSupplier::new(|| 42);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 42);
        // supplier is consumed here
    }

    #[test]
    fn test_get_with_string() {
        let supplier = ArcSupplier::new(|| String::from("hello"));
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_get_with_vec() {
        let supplier = ArcSupplier::new(|| vec![1, 2, 3]);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, vec![1, 2, 3]);
    }

    #[test]
    fn test_get_with_stateful_closure() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_into_box_once() {
        let supplier = ArcSupplier::new(|| 42);
        let once = SupplierOnce::into_box_once(supplier);
        assert_eq!(once.get_once(), 42);
    }

    #[test]
    fn test_into_box_once_with_string() {
        let supplier = ArcSupplier::new(|| String::from("hello"));
        let once = SupplierOnce::into_box_once(supplier);
        assert_eq!(once.get_once(), "hello");
    }

    #[test]
    fn test_into_fn_once() {
        let supplier = ArcSupplier::new(|| 42);
        let fn_once = SupplierOnce::into_fn_once(supplier);
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_into_fn_once_with_string() {
        let supplier = ArcSupplier::new(|| String::from("hello"));
        let fn_once = SupplierOnce::into_fn_once(supplier);
        assert_eq!(fn_once(), "hello");
    }

    #[test]
    fn test_to_box_once_clones_supplier() {
        let supplier = ArcSupplier::new(|| 42);
        let once = supplier.to_box_once();
        // Original supplier still usable (clone it first)
        let s = supplier.clone();
        assert_eq!(s.get_once(), 42);
        // BoxSupplierOnce also works
        assert_eq!(once.get_once(), 42);
    }

    #[test]
    fn test_to_fn_once_clones_supplier() {
        let supplier = ArcSupplier::new(|| 42);
        let fn_once = supplier.to_fn_once();
        // Original supplier still usable (clone it first)
        let s = supplier.clone();
        assert_eq!(s.get_once(), 42);
        // FnOnce also works
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_with_constant_supplier() {
        let supplier = ArcSupplier::constant(42);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_mapped_supplier() {
        let supplier = ArcSupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        let value = SupplierOnce::get_once(mapped);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_with_filtered_supplier() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });
        let filtered = supplier.filter(|x| x % 2 == 0);
        let value = SupplierOnce::get_once(filtered);
        assert_eq!(value, None);
    }

    #[test]
    fn test_with_memoized_supplier() {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = Arc::clone(&call_count);
        let supplier = ArcSupplier::new(move || {
            let mut c = call_count_clone.lock().unwrap();
            *c += 1;
            42
        });
        let memoized = supplier.memoize();
        let value = SupplierOnce::get_once(memoized);
        assert_eq!(value, 42);
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

        // Clone supplier to use in different threads
        let s1 = supplier.clone();
        let s2 = supplier.clone();

        let h1 = thread::spawn(move || SupplierOnce::get_once(s1));
        let h2 = thread::spawn(move || SupplierOnce::get_once(s2));

        let v1 = h1.join().unwrap();
        let v2 = h2.join().unwrap();

        // Both should get different values
        assert!(v1 != v2);
        assert!((1..=2).contains(&v1));
        assert!((1..=2).contains(&v2));
    }

    #[test]
    fn test_to_box_once_shares_state() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        let once = supplier.to_box_once();
        assert_eq!(once.get_once(), 1);
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_to_fn_once_shares_state() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        let supplier = ArcSupplier::new(move || {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
            *c
        });

        let fn_once = supplier.to_fn_once();
        assert_eq!(fn_once(), 1);
        assert_eq!(*counter.lock().unwrap(), 1);
    }
}

// ==========================================================================
// RcSupplier SupplierOnce Implementation Tests
// ==========================================================================

#[cfg(test)]
mod test_rc_supplier_supplier_once {
    use super::*;

    #[test]
    fn test_get_consumes_supplier() {
        let supplier = RcSupplier::new(|| 42);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 42);
        // supplier is consumed here
    }

    #[test]
    fn test_get_with_string() {
        let supplier = RcSupplier::new(|| String::from("hello"));
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_get_with_vec() {
        let supplier = RcSupplier::new(|| vec![1, 2, 3]);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, vec![1, 2, 3]);
    }

    #[test]
    fn test_get_with_stateful_closure() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_into_box_once() {
        let supplier = RcSupplier::new(|| 42);
        let once = SupplierOnce::into_box_once(supplier);
        assert_eq!(once.get_once(), 42);
    }

    #[test]
    fn test_into_box_once_with_string() {
        let supplier = RcSupplier::new(|| String::from("hello"));
        let once = SupplierOnce::into_box_once(supplier);
        assert_eq!(once.get_once(), "hello");
    }

    #[test]
    fn test_into_fn_once() {
        let supplier = RcSupplier::new(|| 42);
        let fn_once = SupplierOnce::into_fn_once(supplier);
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_into_fn_once_with_string() {
        let supplier = RcSupplier::new(|| String::from("hello"));
        let fn_once = SupplierOnce::into_fn_once(supplier);
        assert_eq!(fn_once(), "hello");
    }

    #[test]
    fn test_to_box_once_clones_supplier() {
        let supplier = RcSupplier::new(|| 42);
        let once = supplier.to_box_once();
        // Original supplier still usable (clone it first)
        let s = supplier.clone();
        assert_eq!(s.get_once(), 42);
        // BoxSupplierOnce also works
        assert_eq!(once.get_once(), 42);
    }

    #[test]
    fn test_to_fn_once_clones_supplier() {
        let supplier = RcSupplier::new(|| 42);
        let fn_once = supplier.to_fn_once();
        // Original supplier still usable (clone it first)
        let s = supplier.clone();
        assert_eq!(s.get_once(), 42);
        // FnOnce also works
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_with_constant_supplier() {
        let supplier = RcSupplier::constant(42);
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_mapped_supplier() {
        let supplier = RcSupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        let value = SupplierOnce::get_once(mapped);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_with_filtered_supplier() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });
        let filtered = supplier.filter(|x| x % 2 == 0);
        let value = SupplierOnce::get_once(filtered);
        assert_eq!(value, None);
    }

    #[test]
    fn test_with_memoized_supplier() {
        let call_count = Rc::new(RefCell::new(0));
        let call_count_clone = Rc::clone(&call_count);
        let supplier = RcSupplier::new(move || {
            let mut c = call_count_clone.borrow_mut();
            *c += 1;
            42
        });
        let memoized = supplier.memoize();
        let value = SupplierOnce::get_once(memoized);
        assert_eq!(value, 42);
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

        let s1 = supplier.clone();
        let s2 = supplier.clone();

        assert_eq!(SupplierOnce::get_once(s1), 1);
        assert_eq!(SupplierOnce::get_once(s2), 2);
    }

    #[test]
    fn test_to_box_once_shares_state() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });

        let once = supplier.to_box_once();
        assert_eq!(once.get_once(), 1);
        assert_eq!(*counter.borrow(), 1);
    }

    #[test]
    fn test_to_fn_once_shares_state() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let supplier = RcSupplier::new(move || {
            let mut c = counter_clone.borrow_mut();
            *c += 1;
            *c
        });

        let fn_once = supplier.to_fn_once();
        assert_eq!(fn_once(), 1);
        assert_eq!(*counter.borrow(), 1);
    }

    #[test]
    fn test_with_complex_types() {
        struct Data {
            id: i32,
            name: String,
        }

        let supplier = RcSupplier::new(|| Data {
            id: 1,
            name: String::from("test"),
        });
        let value = SupplierOnce::get_once(supplier);
        assert_eq!(value.id, 1);
        assert_eq!(value.name, "test");
    }
}
