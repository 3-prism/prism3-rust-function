/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for SupplierOnce types

use prism3_function::{BoxSupplierOnce, SupplierOnce};

// ==========================================================================
// SupplierOnce Trait Tests (for closures)
// ==========================================================================

#[cfg(test)]
mod test_supplier_once_trait {
    use super::*;

    #[test]
    fn test_closure_implements_supplier_once() {
        let closure = || 42;
        let boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_move_capture() {
        let data = String::from("hello");
        let closure = move || data;
        let boxed = closure.into_box();
        assert_eq!(boxed.get(), "hello");
    }

    #[test]
    fn test_into_box_once() {
        let closure = || 42;
        let boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_get_direct() {
        let closure = || 42;
        assert_eq!(closure.get(), 42);
    }

    #[test]
    fn test_closure_get_with_move() {
        let data = String::from("hello");
        let closure = move || data;
        assert_eq!(closure.get(), "hello");
    }

    #[test]
    fn test_closure_get_with_complex_type() {
        let closure = || vec![1, 2, 3];
        assert_eq!(closure.get(), vec![1, 2, 3]);
    }

    #[test]
    fn test_into_fn() {
        let closure = || 42;
        let fn_once = closure.into_fn();
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_into_fn_with_move() {
        let data = String::from("hello");
        let closure = move || data;
        let fn_once = closure.into_fn();
        assert_eq!(fn_once(), "hello");
    }

    #[test]
    fn test_into_fn_with_vec() {
        let closure = || vec![1, 2, 3];
        let fn_once = closure.into_fn();
        assert_eq!(fn_once(), vec![1, 2, 3]);
    }

    #[test]
    fn test_into_fn_with_complex_computation() {
        let closure = || {
            let x = 10;
            let y = 32;
            x + y
        };
        let fn_once = closure.into_fn();
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
            assert_eq!(once.get(), 42);
        }

        #[test]
        fn test_with_string() {
            let once = BoxSupplierOnce::new(|| String::from("hello"));
            assert_eq!(once.get(), "hello");
        }

        #[test]
        fn test_with_vec() {
            let once = BoxSupplierOnce::new(|| vec![1, 2, 3]);
            assert_eq!(once.get(), vec![1, 2, 3]);
        }
    }

    mod test_get {
        use super::*;

        #[test]
        fn test_consumes_supplier() {
            let once = BoxSupplierOnce::new(|| 42);
            let value = once.get();
            assert_eq!(value, 42);
            // once is consumed here
        }

        #[test]
        fn test_with_move_closure() {
            let data = String::from("hello");
            let once = BoxSupplierOnce::new(move || data);
            assert_eq!(once.get(), "hello");
        }

        #[test]
        fn test_with_expensive_computation() {
            let once = BoxSupplierOnce::new(move || {
                // Expensive computation
                42
            });
            assert_eq!(once.get(), 42);
        }

        #[test]
        fn test_moves_captured_value() {
            let resource = vec![1, 2, 3];
            let once = BoxSupplierOnce::new(move || resource);
            let result = once.get();
            assert_eq!(result, vec![1, 2, 3]);
        }
    }

    mod test_into_box {
        use super::*;

        #[test]
        fn test_returns_self() {
            let once = BoxSupplierOnce::new(|| 42);
            let boxed = once.into_box();
            assert_eq!(boxed.get(), 42);
        }
    }

    mod test_into_fn {
        use super::*;

        #[test]
        fn test_basic_conversion() {
            let once = BoxSupplierOnce::new(|| 42);
            let fn_once = once.into_fn();
            assert_eq!(fn_once(), 42);
        }

        #[test]
        fn test_with_string() {
            let once = BoxSupplierOnce::new(|| String::from("hello"));
            let fn_once = once.into_fn();
            assert_eq!(fn_once(), "hello");
        }

        #[test]
        fn test_with_move_closure() {
            let data = String::from("captured");
            let once = BoxSupplierOnce::new(move || data);
            let fn_once = once.into_fn();
            assert_eq!(fn_once(), "captured");
        }

        #[test]
        fn test_with_vec() {
            let once = BoxSupplierOnce::new(|| vec![1, 2, 3]);
            let fn_once = once.into_fn();
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
            let value = once.get();
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

            let result = once.get();
            assert_eq!(result, "important data");
        }

        #[test]
        fn test_with_non_cloneable_type() {
            use std::rc::Rc;

            let data = Rc::new(vec![1, 2, 3]);
            let once = BoxSupplierOnce::new(move || data);

            let result = once.get();
            assert_eq!(*result, vec![1, 2, 3]);
        }
    }

    mod test_into_box_conversion {
        use super::*;

        #[test]
        fn test_returns_self() {
            let once = BoxSupplierOnce::new(|| 42);
            let boxed = once.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_closure_into_box() {
            let closure = || 42;
            let boxed = closure.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_closure_with_move() {
            let data = String::from("hello");
            let closure = move || data;
            let boxed = closure.into_box();
            assert_eq!(boxed.get(), "hello");
        }
    }

    mod test_edge_cases {
        use super::*;

        #[test]
        fn test_with_unit_type() {
            let once = BoxSupplierOnce::new(|| ());
            once.get();
            // Unit type always succeeds, no assertion needed
        }

        #[test]
        fn test_with_tuple() {
            let once = BoxSupplierOnce::new(|| (1, "hello", true));
            assert_eq!(once.get(), (1, "hello", true));
        }

        #[test]
        fn test_with_option_some() {
            let once = BoxSupplierOnce::new(|| Some(42));
            assert_eq!(once.get(), Some(42));
        }

        #[test]
        fn test_with_option_none() {
            let once = BoxSupplierOnce::new(|| None::<i32>);
            assert_eq!(once.get(), None);
        }

        #[test]
        fn test_with_result_ok() {
            let once = BoxSupplierOnce::new(|| Ok::<i32, String>(42));
            assert_eq!(once.get(), Ok(42));
        }

        #[test]
        fn test_with_result_err() {
            let once = BoxSupplierOnce::new(|| Err::<i32, String>(String::from("error")));
            assert_eq!(once.get(), Err(String::from("error")));
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
        fn get(mut self) -> T {
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
        assert_eq!(custom.get(), 42);
    }

    #[test]
    fn test_custom_type_into_box_default_impl() {
        let custom = CustomSupplierOnce::new(42);
        let boxed = custom.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_custom_type_with_string() {
        let custom = CustomSupplierOnce::new(String::from("hello"));
        let boxed = custom.into_box();
        assert_eq!(boxed.get(), "hello");
    }

    #[test]
    fn test_custom_type_with_vec() {
        let custom = CustomSupplierOnce::new(vec![1, 2, 3]);
        let boxed = custom.into_box();
        assert_eq!(boxed.get(), vec![1, 2, 3]);
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
        let boxed = custom.into_box();
        let result = boxed.get();
        assert_eq!(result.id, 1);
        assert_eq!(result.name, "test");
    }

    #[test]
    fn test_custom_type_with_option() {
        let custom = CustomSupplierOnce::new(Some(42));
        let boxed = custom.into_box();
        assert_eq!(boxed.get(), Some(42));
    }

    #[test]
    fn test_custom_type_with_result() {
        let custom = CustomSupplierOnce::new(Ok::<i32, String>(42));
        let boxed = custom.into_box();
        assert_eq!(boxed.get(), Ok(42));
    }

    #[test]
    fn test_custom_type_into_fn_default_impl() {
        let custom = CustomSupplierOnce::new(42);
        let fn_once = custom.into_fn();
        assert_eq!(fn_once(), 42);
    }

    #[test]
    fn test_custom_type_into_fn_with_string() {
        let custom = CustomSupplierOnce::new(String::from("hello"));
        let fn_once = custom.into_fn();
        assert_eq!(fn_once(), "hello");
    }

    #[test]
    fn test_custom_type_into_fn_with_vec() {
        let custom = CustomSupplierOnce::new(vec![1, 2, 3]);
        let fn_once = custom.into_fn();
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
        fn get(self) -> i32 {
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
        let fn_once = supplier.to_fn();
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
        let boxed = supplier.to_box();
        // The original supplier is not consumed
        assert!(supplier.value.lock().unwrap().is_some());
        // The returned BoxSupplierOnce can be consumed
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_specialized_to_fn_for_cloneable_closure() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let closure = move || {
            *counter_clone.lock().unwrap() += 1;
            42
        };
        let fn_once = closure.to_fn();
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
        let boxed = closure.to_box();
        boxed.get();
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_box_supplier_once_cannot_use_to_box() {
        // This test demonstrates that `to_box` cannot be called on a
        // `BoxSupplierOnce` because it does not implement `Clone`.
        // The following code will fail to compile, which is the expected
        // behavior.

        // let once = BoxSupplierOnce::new(|| 42);
        // let _boxed = once.to_box(); // COMPILE ERROR: `BoxSupplierOnce` is not `Clone`
    }

    #[test]
    fn test_box_supplier_once_cannot_use_to_fn() {
        // Similar to `to_box`, `to_fn` cannot be called on a `BoxSupplierOnce`
        // because of the `Clone` trait bound. The following code is commented
        // out because it would prevent the test suite from compiling.

        // let once = BoxSupplierOnce::new(|| 42);
        // let _fn_once = once.to_fn(); // COMPILE ERROR: `BoxSupplierOnce` is not `Clone`
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
        // let _boxed = _closure.to_box();
        // let _fn_once = _closure.to_fn();
    }
}
