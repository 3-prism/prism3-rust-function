// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for SupplierOnce types

use qubit_function::BoxSupplierOnce;
use qubit_function::SupplierOnce;

// ==========================================================================
// SupplierOnce Trait Tests (for closures)
// ==========================================================================

#[cfg(test)]
mod test_supplier_once_trait {
    use super::SupplierOnce;

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
}

// ==========================================================================
// BoxSupplierOnce Tests
// ==========================================================================

#[cfg(test)]
mod test_box_supplier_once {
    use super::BoxSupplierOnce;
    use super::SupplierOnce;

    mod test_new {
        use super::BoxSupplierOnce;
        use super::SupplierOnce;

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
        use super::BoxSupplierOnce;
        use super::SupplierOnce;

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

    mod test_use_cases {
        use super::BoxSupplierOnce;
        use super::SupplierOnce;

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

    mod test_edge_cases {
        use super::BoxSupplierOnce;
        use super::SupplierOnce;

        #[test]
        fn test_with_unit_type() {
            let once = BoxSupplierOnce::new(|| ());
            let _: () = once.get();
            std::hint::black_box(());
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
    use super::SupplierOnce;

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
            self.value.take().expect("CustomSupplierOnce already consumed")
        }
        // Note: into_box() is NOT implemented here, so the
        // default implementation from the trait will be used
    }

    #[test]
    fn test_custom_type_get_method() {
        let custom = CustomSupplierOnce::new(42);
        assert_eq!(custom.get(), 42);
    }
}
// ======================================================================
// Debug and Display Trait Tests
// ======================================================================

#[cfg(test)]
mod test_supplier_once_debug_display {
    use super::BoxSupplierOnce;

    // ============================================================
    // BoxSupplierOnce Debug and Display Tests
    // ============================================================

    mod test_box_supplier_once_debug_display {
        use super::BoxSupplierOnce;

        #[test]
        fn test_debug_without_name() {
            // Test Debug formatting for BoxSupplierOnce without name
            let supplier = BoxSupplierOnce::new(|| 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("BoxSupplierOnce"));
            assert!(debug_str.contains("name: None"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_debug_with_name() {
            // Test Debug formatting for BoxSupplierOnce with name
            let supplier = BoxSupplierOnce::new_with_name("test_supplier", || 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("BoxSupplierOnce"));
            assert!(debug_str.contains("name: Some(\"test_supplier\")"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_display_without_name() {
            // Test Display formatting for BoxSupplierOnce without name
            let supplier = BoxSupplierOnce::new(|| 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "BoxSupplierOnce");
        }

        #[test]
        fn test_display_with_name() {
            // Test Display formatting for BoxSupplierOnce with name
            let supplier = BoxSupplierOnce::new_with_name("test_supplier", || 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "BoxSupplierOnce(test_supplier)");
        }
    }
}
