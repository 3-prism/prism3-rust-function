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
        let boxed = closure.into_box_once();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_move_capture() {
        let data = String::from("hello");
        let closure = move || data;
        let boxed = closure.into_box_once();
        assert_eq!(boxed.get(), "hello");
    }

    #[test]
    fn test_into_box_once() {
        let closure = || 42;
        let boxed = closure.into_box_once();
        assert_eq!(boxed.get(), 42);
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

    mod test_into_box_once {
        use super::*;

        #[test]
        fn test_returns_self() {
            let once = BoxSupplierOnce::new(|| 42);
            let boxed = once.into_box_once();
            assert_eq!(boxed.get(), 42);
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
}
