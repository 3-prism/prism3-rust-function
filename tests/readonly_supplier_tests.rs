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
    ArcSupplier,
    ArcTransformer,
    BoxSupplier,
    BoxTransformer,
    RcSupplier,
    RcTransformer,
    Supplier,
};
use std::sync::Arc;
use std::thread;

// ======================================================================
// Supplier Trait Tests (for closures)
// ======================================================================

#[cfg(test)]
mod test_readonly_supplier_trait {
    use super::*;

    #[test]
    fn test_closure_implements_readonly_supplier() {
        // Test that closure implements Supplier trait
        let closure = || 42;
        let boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_stateless() {
        // Test stateless closure (always returns same value)
        let boxed = BoxSupplier::new(|| 42);
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_box() {
        // Test conversion to BoxSupplier
        let closure = || 42;
        let boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_rc() {
        // Test conversion to RcSupplier
        let closure = || 42;
        let rc = closure.into_rc();
        assert_eq!(rc.get(), 42);
    }

    #[test]
    fn test_into_arc() {
        // Test conversion to ArcSupplier
        let closure = || 42;
        let arc = closure.into_arc();
        assert_eq!(arc.get(), 42);
    }

    #[test]
    fn test_closure_get() {
        // Test the get method in impl<T, F> Supplier<T>
        // for F
        let closure = || 42;
        assert_eq!(closure.get(), 42);
        assert_eq!(closure.get(), 42);
    }

    #[test]
    fn test_closure_get_stateless() {
        // Test stateless closure (doesn't modify captured
        // variables)
        let value = 100;
        let closure = move || value * 2;
        assert_eq!(closure.get(), 200);
        assert_eq!(closure.get(), 200);
        assert_eq!(closure.get(), 200);
    }

    #[test]
    fn test_into_fn() {
        // Test conversion to FnMut closure
        let closure = || 42;
        let mut fn_mut = closure.into_fn();
        assert_eq!(fn_mut(), 42);
        assert_eq!(fn_mut(), 42);
    }

    #[test]
    fn test_into_fn_with_captured_value() {
        // Test into_fn with captured value
        let value = 100;
        let closure = move || value * 2;
        let mut fn_mut = closure.into_fn();
        assert_eq!(fn_mut(), 200);
        assert_eq!(fn_mut(), 200);
    }

    #[test]
    fn test_into_fn_returns_different_types() {
        // Test into_fn with different return types
        let closure_i32 = || 42i32;
        let mut fn_mut_i32 = closure_i32.into_fn();
        assert_eq!(fn_mut_i32(), 42i32);

        let closure_str = || "hello";
        let mut fn_mut_str = closure_str.into_fn();
        assert_eq!(fn_mut_str(), "hello");
    }
}

// ======================================================================
// BoxSupplier Tests
// ======================================================================

#[cfg(test)]
mod test_box_readonly_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_new_basic() {
            // Test creating a new BoxSupplier
            let supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_new_with_closure() {
            // Test with a closure that captures variables
            let value = 100;
            let supplier = BoxSupplier::new(move || value);
            assert_eq!(supplier.get(), 100);
        }

        #[test]
        fn test_new_returns_same_value() {
            // Test that successive calls return same value
            let supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
        }
    }

    mod test_constant {
        use super::*;

        #[test]
        fn test_constant_basic() {
            // Test constant supplier
            let constant = BoxSupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_constant_string() {
            // Test constant with String type
            let constant = BoxSupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }

        #[test]
        fn test_constant_vec() {
            // Test constant with Vec type
            let constant = BoxSupplier::constant(vec![1, 2, 3]);
            assert_eq!(constant.get(), vec![1, 2, 3]);
            assert_eq!(constant.get(), vec![1, 2, 3]);
        }
    }

    mod test_map {
        use super::*;

        #[test]
        fn test_map_basic() {
            // Test map transformation
            let mapped = BoxSupplier::new(|| 10).map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_map_chain() {
            // Test chained map operations
            let pipeline = BoxSupplier::new(|| 10).map(|x| x * 2).map(|x| x + 5);
            assert_eq!(pipeline.get(), 25);
        }

        #[test]
        fn test_map_type_conversion() {
            // Test map with type conversion
            let mapped = BoxSupplier::new(|| 42).map(|x: i32| x.to_string());
            assert_eq!(mapped.get(), "42");
        }
    }

    mod test_filter {
        use super::*;

        #[test]
        fn test_filter_passes() {
            // Test filter that passes
            let filtered = BoxSupplier::new(|| 42).filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let filtered = BoxSupplier::new(|| 43).filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }

        #[test]
        fn test_filter_with_map() {
            // Test combining filter and map
            let pipeline = BoxSupplier::new(|| 10).map(|x| x * 2).filter(|x| *x > 15);
            assert_eq!(pipeline.get(), Some(20));
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = BoxSupplier::new(|| 42);
            let second = BoxSupplier::new(|| "hello");
            let zipped = first.zip(second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_different_types() {
            // Test zipping suppliers of different types
            let first = BoxSupplier::new(|| 100);
            let second = BoxSupplier::new(|| vec![1, 2, 3]);
            let zipped = first.zip(second);
            assert_eq!(zipped.get(), (100, vec![1, 2, 3]));
        }
    }

    mod test_trait_methods {
        use super::*;

        #[test]
        fn test_get() {
            // Test Supplier::get method
            let supplier = BoxSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_into_box() {
            // Test into_box (should return self)
            let supplier = BoxSupplier::new(|| 42);
            let boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_into_rc() {
            // Test conversion to RcSupplier
            let supplier = BoxSupplier::new(|| 42);
            let rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_into_fn() {
            // Test conversion to FnMut closure
            let supplier = BoxSupplier::new(|| 42);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);
        }

        #[test]
        fn test_into_fn_with_captured_value() {
            // Test into_fn with captured value
            let value = 100;
            let supplier = BoxSupplier::new(move || value * 2);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 200);
            assert_eq!(fn_mut(), 200);
        }

        #[test]
        fn test_into_fn_with_string() {
            // Test into_fn with String type
            let supplier = BoxSupplier::new(|| String::from("hello"));
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), "hello");
            assert_eq!(fn_mut(), "hello");
        }

        // Note: test_into_arc is not included here because
        // BoxSupplier cannot be converted to
        // ArcSupplier (inner function may not be Send +
        // Sync). This is enforced at compile time by trait bounds.
    }
}

// ======================================================================
// ArcSupplier Tests
// ======================================================================

#[cfg(test)]
mod test_arc_readonly_supplier {
    use super::*;

    mod test_new {
        use super::*;

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
        use super::*;

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
        use super::*;

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
        use super::*;

        #[test]
        fn test_filter_passes() {
            // Test filter that passes
            let source = ArcSupplier::new(|| 42);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let source = ArcSupplier::new(|| 43);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = ArcSupplier::new(|| 42);
            let second = ArcSupplier::new(|| "hello");
            let zipped = first.zip(&second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_preserves_originals() {
            // Test that zip doesn't consume originals
            let first = ArcSupplier::new(|| 42);
            let second = ArcSupplier::new(|| "hello");
            let _zipped = first.zip(&second);
            // Both are still usable
            assert_eq!(first.get(), 42);
            assert_eq!(second.get(), "hello");
        }
    }

    mod test_clone {
        use super::*;

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
        use super::*;

        #[test]
        fn test_send_between_threads() {
            // Test that supplier can be sent between threads
            let supplier = ArcSupplier::new(|| 42);
            let handle = thread::spawn(move || supplier.get());
            assert_eq!(handle.join().unwrap(), 42);
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
                assert_eq!(h.join().unwrap(), "Hello, World!");
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
                assert_eq!(h.join().unwrap(), 100);
            }
        }
    }

    mod test_trait_methods {
        use super::*;

        #[test]
        fn test_get() {
            // Test Supplier::get method
            let supplier = ArcSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_into_box() {
            // Test conversion to BoxSupplier
            let supplier = ArcSupplier::new(|| 42);
            let boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_into_rc() {
            // Test conversion to RcSupplier
            let supplier = ArcSupplier::new(|| 42);
            let rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_into_arc() {
            // Test into_arc (should return self)
            let supplier = ArcSupplier::new(|| 42);
            let arc = supplier.into_arc();
            assert_eq!(arc.get(), 42);
        }

        #[test]
        fn test_into_fn() {
            // Test conversion to FnMut closure
            let supplier = ArcSupplier::new(|| 42);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);
        }

        #[test]
        fn test_into_fn_with_captured_value() {
            // Test into_fn with captured value
            let value = 100;
            let supplier = ArcSupplier::new(move || value * 2);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 200);
            assert_eq!(fn_mut(), 200);
        }

        #[test]
        fn test_into_fn_with_string() {
            // Test into_fn with String type
            let supplier = ArcSupplier::new(|| String::from("hello"));
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), "hello");
            assert_eq!(fn_mut(), "hello");
        }

        #[test]
        fn test_into_fn_thread_safe() {
            // Test that into_fn result can be sent to another thread
            let supplier = ArcSupplier::new(|| 42);
            let func = supplier.into_fn();
            let handle = thread::spawn(func);
            assert_eq!(handle.join().unwrap(), 42);
        }
    }
}

// ======================================================================
// RcSupplier Tests
// ======================================================================

#[cfg(test)]
mod test_rc_readonly_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_new_basic() {
            // Test creating a new RcSupplier
            let supplier = RcSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_new_with_closure() {
            // Test with a closure that captures variables
            let value = 100;
            let supplier = RcSupplier::new(move || value);
            assert_eq!(supplier.get(), 100);
        }

        #[test]
        fn test_new_returns_same_value() {
            // Test that successive calls return same value
            let supplier = RcSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
            assert_eq!(supplier.get(), 42);
        }
    }

    mod test_constant {
        use super::*;

        #[test]
        fn test_constant_basic() {
            // Test constant supplier
            let constant = RcSupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_constant_string() {
            // Test constant with String type
            let constant = RcSupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }
    }

    mod test_map {
        use super::*;

        #[test]
        fn test_map_basic() {
            // Test map transformation
            let source = RcSupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_map_chain() {
            // Test chained map operations
            let source = RcSupplier::new(|| 10);
            let pipeline = source.map(|x| x * 2).map(|x| x + 5);
            assert_eq!(pipeline.get(), 25);
        }

        #[test]
        fn test_map_preserves_original() {
            // Test that mapping doesn't consume original
            let source = RcSupplier::new(|| 10);
            let _mapped = source.map(|x| x * 2);
            // source is still usable
            assert_eq!(source.get(), 10);
        }
    }

    mod test_filter {
        use super::*;

        #[test]
        fn test_filter_passes() {
            // Test filter that passes
            let source = RcSupplier::new(|| 42);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let source = RcSupplier::new(|| 43);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = RcSupplier::new(|| 42);
            let second = RcSupplier::new(|| "hello");
            let zipped = first.zip(&second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_preserves_originals() {
            // Test that zip doesn't consume originals
            let first = RcSupplier::new(|| 42);
            let second = RcSupplier::new(|| "hello");
            let _zipped = first.zip(&second);
            // Both are still usable
            assert_eq!(first.get(), 42);
            assert_eq!(second.get(), "hello");
        }
    }

    mod test_clone {
        use super::*;

        #[test]
        fn test_clone_basic() {
            // Test cloning supplier
            let original = RcSupplier::new(|| 42);
            let cloned = original.clone();
            assert_eq!(original.get(), 42);
            assert_eq!(cloned.get(), 42);
        }

        #[test]
        fn test_clone_shares_function() {
            // Test that clone shares the underlying function
            let original = RcSupplier::new(|| String::from("hello"));
            let cloned = original.clone();
            assert_eq!(original.get(), cloned.get());
        }
    }

    mod test_trait_methods {
        use super::*;

        #[test]
        fn test_get() {
            // Test Supplier::get method
            let supplier = RcSupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_into_box() {
            // Test conversion to BoxSupplier
            let supplier = RcSupplier::new(|| 42);
            let boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_into_rc() {
            // Test into_rc (should return self)
            let supplier = RcSupplier::new(|| 42);
            let rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_into_fn() {
            // Test conversion to FnMut closure
            let supplier = RcSupplier::new(|| 42);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);
        }

        #[test]
        fn test_into_fn_with_captured_value() {
            // Test into_fn with captured value
            let value = 100;
            let supplier = RcSupplier::new(move || value * 2);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 200);
            assert_eq!(fn_mut(), 200);
        }

        #[test]
        fn test_into_fn_with_string() {
            // Test into_fn with String type
            let supplier = RcSupplier::new(|| String::from("hello"));
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), "hello");
            assert_eq!(fn_mut(), "hello");
        }

        // Note: test_into_arc is not included here because
        // RcSupplier cannot be converted to
        // ArcSupplier (Rc is not Send + Sync). This is
        // enforced at compile time by trait bounds.
    }
}

// ======================================================================
// Integration Tests
// ======================================================================

#[cfg(test)]
mod test_integration {
    use super::*;

    #[test]
    fn test_usage_in_read_only_context() {
        // Test using supplier in read-only struct methods
        struct Executor {
            error_supplier: ArcSupplier<String>,
        }

        impl Executor {
            fn execute(&self) -> Result<(), String> {
                // Can call supplier in &self method!
                Err(self.error_supplier.get())
            }
        }

        let executor = Executor {
            error_supplier: ArcSupplier::new(|| String::from("Error occurred")),
        };

        assert_eq!(executor.execute(), Err(String::from("Error occurred")));
    }

    #[test]
    fn test_factory_pattern() {
        // Test using as a factory for creating instances
        #[derive(Debug, PartialEq)]
        struct Config {
            timeout: u64,
        }

        let factory = BoxSupplier::new(|| Config { timeout: 30 });

        let config1 = factory.get();
        let config2 = factory.get();

        assert_eq!(config1, Config { timeout: 30 });
        assert_eq!(config2, Config { timeout: 30 });
    }

    #[test]
    fn test_concurrent_factory() {
        // Test using as factory in concurrent context
        let factory = Arc::new(ArcSupplier::new(|| vec![1, 2, 3, 4, 5]));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let f = Arc::clone(&factory);
                thread::spawn(move || f.get())
            })
            .collect();

        for h in handles {
            assert_eq!(h.join().unwrap(), vec![1, 2, 3, 4, 5]);
        }
    }

    #[test]
    fn test_mixed_transformations() {
        // Test combining multiple transformation methods
        let pipeline = BoxSupplier::new(|| 10)
            .map(|x| x * 2)
            .filter(|x| *x > 15)
            .map(|opt: Option<i32>| opt.map(|x| x.to_string()));

        assert_eq!(pipeline.get(), Some(String::from("20")));
    }

    #[test]
    fn test_conversion_chain() {
        // Test converting between different supplier types
        let closure = || 42;
        let boxed = closure.into_box();
        let rc = boxed.into_rc();
        assert_eq!(rc.get(), 42);
    }
}

// ======================================================================
// Map with Transformer Tests - BoxSupplier
// ======================================================================

#[cfg(test)]
mod test_box_readonly_supplier_map_with_transformer {
    use super::*;

    // Helper function pointers
    fn double(x: i32) -> i32 {
        x * 2
    }

    fn to_string(x: i32) -> String {
        x.to_string()
    }

    #[test]
    fn test_map_with_closure() {
        // Test map accepts closure
        let supplier = BoxSupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_function_pointer() {
        // Test map accepts function pointer
        let supplier = BoxSupplier::new(|| 10);
        let mapped = supplier.map(double);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_box_transformer() {
        // Test map accepts BoxTransformer object
        let supplier = BoxSupplier::new(|| 10);
        let transformer = BoxTransformer::new(|x| x * 3);
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_chain_with_different_types() {
        // Test chained calls, each map uses different type of transformer
        let supplier = BoxSupplier::new(|| 10);
        let step1 = supplier.map(|x| x * 2); // closure
        let step2 = step1.map(double); // function pointer
        let step3 = step2.map(BoxTransformer::new(|x| x + 5)); // BoxTransformer
        assert_eq!(step3.get(), 45); // (10 * 2) * 2 + 5 = 45
    }

    #[test]
    fn test_map_with_closure_capturing_variables() {
        // Test map uses closure capturing variables
        let multiplier = 3;
        let supplier = BoxSupplier::new(|| 10);
        let mapped = supplier.map(move |x| x * multiplier);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_with_type_conversion() {
        // Test map performs type conversion
        let supplier = BoxSupplier::new(|| 42);

        // Use closure to convert type
        let mapped1 = supplier.map(|x: i32| x.to_string());
        assert_eq!(mapped1.get(), "42");

        // Use BoxTransformer to convert type
        let supplier2 = BoxSupplier::new(|| 42);
        let transformer = BoxTransformer::new(to_string);
        let mapped2 = supplier2.map(transformer);
        assert_eq!(mapped2.get(), "42");
    }

    #[test]
    fn test_map_with_complex_transformer() {
        // Test map uses complex Transformer
        #[derive(Debug, PartialEq)]
        struct Data {
            value: i32,
        }

        let supplier = BoxSupplier::new(|| 10);
        let transformer = BoxTransformer::new(|x| Data { value: x * 2 });
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), Data { value: 20 });
    }
}

// ======================================================================
// Map with Transformer Tests - ArcSupplier
// ======================================================================

#[cfg(test)]
mod test_arc_readonly_supplier_map_with_transformer {
    use super::*;

    // Helper function pointers
    fn double(x: i32) -> i32 {
        x * 2
    }

    fn to_string(x: i32) -> String {
        x.to_string()
    }

    #[test]
    fn test_map_with_closure() {
        // Test map accepts closure
        let supplier = ArcSupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_function_pointer() {
        // Test map accepts function pointer
        let supplier = ArcSupplier::new(|| 10);
        let mapped = supplier.map(double);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_arc_transformer() {
        // Test map accepts ArcTransformer object
        let supplier = ArcSupplier::new(|| 10);
        let transformer = ArcTransformer::new(|x| x * 3);
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_chain_with_different_types() {
        // Test chained calls, each map uses different type of transformer
        let supplier = ArcSupplier::new(|| 10);
        let step1 = supplier.map(|x| x * 2); // closure
        let step2 = step1.map(double); // function pointer
        let step3 = step2.map(ArcTransformer::new(|x| x + 5)); // ArcTransformer
        assert_eq!(step3.get(), 45); // (10 * 2) * 2 + 5 = 45
    }

    #[test]
    fn test_map_with_closure_capturing_variables() {
        // Test map uses closure capturing variables
        let multiplier = 3;
        let supplier = ArcSupplier::new(|| 10);
        let mapped = supplier.map(move |x| x * multiplier);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_preserves_original_with_transformer() {
        // Test original supplier still usable after using transformer
        let supplier = ArcSupplier::new(|| 10);
        let transformer = ArcTransformer::new(|x| x * 2);
        let mapped = supplier.map(transformer);

        // Original supplier still usable
        assert_eq!(supplier.get(), 10);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_thread_safety_with_transformer() {
        // Test map with transformer in multi-threaded environment
        let supplier = ArcSupplier::new(|| 10);
        let transformer = ArcTransformer::new(|x| x * 2);
        let mapped = supplier.map(transformer);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let m = mapped.clone();
                thread::spawn(move || m.get())
            })
            .collect();

        for h in handles {
            assert_eq!(h.join().unwrap(), 20);
        }
    }

    #[test]
    fn test_map_with_type_conversion() {
        // Test map performs type conversion
        let supplier = ArcSupplier::new(|| 42);

        // Use closure to convert type
        let mapped1 = supplier.map(|x: i32| x.to_string());
        assert_eq!(mapped1.get(), "42");

        // Use ArcTransformer to convert type
        let transformer = ArcTransformer::new(to_string);
        let mapped2 = supplier.map(transformer);
        assert_eq!(mapped2.get(), "42");
    }

    #[test]
    fn test_map_with_shared_transformer() {
        // Test multiple suppliers sharing the same transformer
        let supplier1 = ArcSupplier::new(|| 10);
        let supplier2 = ArcSupplier::new(|| 20);

        let transformer = ArcTransformer::new(|x| x * 2);
        let mapped1 = supplier1.map(transformer.clone());
        let mapped2 = supplier2.map(transformer);

        assert_eq!(mapped1.get(), 20);
        assert_eq!(mapped2.get(), 40);
    }
}

// ======================================================================
// Map with Transformer Tests - RcSupplier
// ======================================================================

#[cfg(test)]
mod test_rc_readonly_supplier_map_with_transformer {
    use super::*;

    // Helper function pointers
    fn double(x: i32) -> i32 {
        x * 2
    }

    fn to_string(x: i32) -> String {
        x.to_string()
    }

    #[test]
    fn test_map_with_closure() {
        // Test map accepts closure
        let supplier = RcSupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_function_pointer() {
        // Test map accepts function pointer
        let supplier = RcSupplier::new(|| 10);
        let mapped = supplier.map(double);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_rc_transformer() {
        // Test map accepts RcTransformer object
        let supplier = RcSupplier::new(|| 10);
        let transformer = RcTransformer::new(|x| x * 3);
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_chain_with_different_types() {
        // Test chained calls, each map uses different type of transformer
        let supplier = RcSupplier::new(|| 10);
        let step1 = supplier.map(|x| x * 2); // closure
        let step2 = step1.map(double); // function pointer
        let step3 = step2.map(RcTransformer::new(|x| x + 5)); // RcTransformer
        assert_eq!(step3.get(), 45); // (10 * 2) * 2 + 5 = 45
    }

    #[test]
    fn test_map_with_closure_capturing_variables() {
        // Test map uses closure capturing variables
        let multiplier = 3;
        let supplier = RcSupplier::new(|| 10);
        let mapped = supplier.map(move |x| x * multiplier);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_preserves_original_with_transformer() {
        // Test original supplier still usable after using transformer
        let supplier = RcSupplier::new(|| 10);
        let transformer = RcTransformer::new(|x| x * 2);
        let mapped = supplier.map(transformer);

        // Original supplier still usable
        assert_eq!(supplier.get(), 10);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_type_conversion() {
        // Test map performs type conversion
        let supplier = RcSupplier::new(|| 42);

        // Use closure to convert type
        let mapped1 = supplier.map(|x: i32| x.to_string());
        assert_eq!(mapped1.get(), "42");

        // Use RcTransformer to convert type
        let transformer = RcTransformer::new(to_string);
        let mapped2 = supplier.map(transformer);
        assert_eq!(mapped2.get(), "42");
    }

    #[test]
    fn test_map_with_shared_transformer() {
        // Test multiple suppliers sharing the same transformer
        let supplier1 = RcSupplier::new(|| 10);
        let supplier2 = RcSupplier::new(|| 20);

        let transformer = RcTransformer::new(|x| x * 2);
        let mapped1 = supplier1.map(transformer.clone());
        let mapped2 = supplier2.map(transformer);

        assert_eq!(mapped1.get(), 20);
        assert_eq!(mapped2.get(), 40);
    }
}

// ======================================================================
// Integration Tests for Map with Transformer
// ======================================================================

#[cfg(test)]
mod test_map_transformer_integration {
    use super::*;

    #[test]
    fn test_mixed_transformer_types_in_pipeline() {
        // Test mixing different types of transformers in pipeline
        let supplier = BoxSupplier::new(|| 5);

        let pipeline = supplier
            .map(|x| x * 2) // closure
            .map(|x: i32| -> i32 { x + 3 }) // closure with explicit type annotation
            .map(|x: i32| x.to_string()); // type conversion closure

        assert_eq!(pipeline.get(), "13");
    }

    #[test]
    fn test_transformer_with_complex_logic() {
        // Test transformer with complex logic
        #[derive(Debug, PartialEq)]
        struct Result {
            doubled: i32,
            squared: i32,
        }

        let supplier = ArcSupplier::new(|| 5);
        let transformer = ArcTransformer::new(|x| Result {
            doubled: x * 2,
            squared: x * x,
        });

        let mapped = supplier.map(transformer);
        assert_eq!(
            mapped.get(),
            Result {
                doubled: 10,
                squared: 25
            }
        );
    }

    #[test]
    fn test_function_pointer_with_generic_supplier() {
        // Test function pointer with generic supplier
        fn process(x: i32) -> String {
            format!("Value: {}", x * 2)
        }

        let supplier = ArcSupplier::new(|| 21);
        let mapped = supplier.map(process);
        assert_eq!(mapped.get(), "Value: 42");
    }

    #[test]
    fn test_transformer_reusability() {
        // Test reusability of Transformer
        let transformer = ArcTransformer::new(|x: i32| x * 10);

        let supplier1 = ArcSupplier::new(|| 1);
        let supplier2 = ArcSupplier::new(|| 2);
        let supplier3 = ArcSupplier::new(|| 3);

        let mapped1 = supplier1.map(transformer.clone());
        let mapped2 = supplier2.map(transformer.clone());
        let mapped3 = supplier3.map(transformer);

        assert_eq!(mapped1.get(), 10);
        assert_eq!(mapped2.get(), 20);
        assert_eq!(mapped3.get(), 30);
    }
}

// ======================================================================
// Default Implementation Tests for Custom Types
// ======================================================================

#[cfg(test)]
mod test_custom_readonly_supplier_default_impl {
    use super::*;

    /// A simple custom type that implements Supplier with
    /// only the core `get` method, relying on default
    /// implementations for `into_box`, `into_rc`, and `into_arc`.
    struct CounterSupplier {
        /// The value to return each time `get` is called.
        value: i32,
    }

    impl CounterSupplier {
        /// Creates a new CounterSupplier with the given value.
        fn new(value: i32) -> Self {
            Self { value }
        }
    }

    impl Supplier<i32> for CounterSupplier {
        fn get(&self) -> i32 {
            self.value
        }

        // All into_xxx methods use default implementations
    }

    #[test]
    fn test_custom_supplier_get() {
        // Test that the custom supplier correctly implements the
        // core get method
        let supplier = CounterSupplier::new(42);
        assert_eq!(supplier.get(), 42);
        assert_eq!(supplier.get(), 42);
    }

    #[test]
    fn test_custom_supplier_into_box_default() {
        // Test that the default implementation of into_box works
        // correctly for custom types
        let supplier = CounterSupplier::new(100);
        let boxed = supplier.into_box();

        assert_eq!(boxed.get(), 100);
        assert_eq!(boxed.get(), 100);
    }

    #[test]
    fn test_custom_supplier_into_rc_default() {
        // Test that the default implementation of into_rc works
        // correctly for custom types
        let supplier = CounterSupplier::new(200);
        let rc = supplier.into_rc();

        assert_eq!(rc.get(), 200);
        assert_eq!(rc.get(), 200);

        // Verify that Rc can be cloned
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.get(), 200);
    }

    #[test]
    fn test_custom_supplier_into_arc_default() {
        // Test that the default implementation of into_arc works
        // correctly for custom types
        let supplier = CounterSupplier::new(300);
        let arc = supplier.into_arc();

        assert_eq!(arc.get(), 300);
        assert_eq!(arc.get(), 300);

        // Verify that Arc can be cloned
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.get(), 300);
    }

    #[test]
    fn test_custom_supplier_arc_thread_safety() {
        // Test that the Arc variant created from custom supplier
        // using default implementation is thread-safe
        let supplier = CounterSupplier::new(999);
        let arc = supplier.into_arc();

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let a = arc.clone();
                thread::spawn(move || a.get())
            })
            .collect();

        for h in handles {
            assert_eq!(h.join().unwrap(), 999);
        }
    }

    #[test]
    fn test_custom_supplier_conversion_chain() {
        // Test chaining conversions using default implementations
        let supplier = CounterSupplier::new(50);
        let boxed = supplier.into_box();
        let rc = boxed.into_rc();

        assert_eq!(rc.get(), 50);
    }

    #[test]
    fn test_custom_supplier_with_transformations() {
        // Test that converted suppliers work with map operations
        let supplier = CounterSupplier::new(10);
        let arc = supplier.into_arc();
        let mapped = arc.map(|x| x * 3);

        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_custom_supplier_multiple_conversions() {
        // Test that we can create different wrapper types from the
        // same custom supplier instance
        let supplier1 = CounterSupplier::new(77);
        let supplier2 = CounterSupplier::new(77);
        let supplier3 = CounterSupplier::new(77);

        let boxed = supplier1.into_box();
        let rc = supplier2.into_rc();
        let arc = supplier3.into_arc();

        assert_eq!(boxed.get(), 77);
        assert_eq!(rc.get(), 77);
        assert_eq!(arc.get(), 77);
    }

    #[test]
    fn test_custom_supplier_into_fn_default() {
        // Test that the default implementation of into_fn works
        // correctly for custom types
        let supplier = CounterSupplier::new(42);
        let mut fn_mut = supplier.into_fn();

        assert_eq!(fn_mut(), 42);
        assert_eq!(fn_mut(), 42);
        assert_eq!(fn_mut(), 42);
    }

    #[test]
    fn test_custom_supplier_into_fn_with_different_values() {
        // Test into_fn with different values
        let supplier1 = CounterSupplier::new(100);
        let mut fn_mut1 = supplier1.into_fn();
        assert_eq!(fn_mut1(), 100);

        let supplier2 = CounterSupplier::new(200);
        let mut fn_mut2 = supplier2.into_fn();
        assert_eq!(fn_mut2(), 200);
    }

    #[test]
    fn test_custom_supplier_into_fn_multiple_calls() {
        // Test that into_fn result can be called multiple times
        let supplier = CounterSupplier::new(999);
        let mut fn_mut = supplier.into_fn();

        for _ in 0..10 {
            assert_eq!(fn_mut(), 999);
        }
    }

    #[test]
    fn test_custom_supplier_to_box_default() {
        // Test that the default implementation of to_box works
        // correctly for custom Clone types
        let supplier = CounterSupplier::new(100);
        let boxed = supplier.to_box();

        assert_eq!(boxed.get(), 100);
        assert_eq!(boxed.get(), 100);
    }

    #[test]
    fn test_custom_supplier_to_rc_default() {
        // Test that the default implementation of to_rc works
        // correctly for custom Clone types
        let supplier = CounterSupplier::new(200);
        let rc = supplier.to_rc();

        assert_eq!(rc.get(), 200);
        assert_eq!(rc.get(), 200);

        // Verify that Rc can be cloned
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.get(), 200);
    }

    #[test]
    fn test_custom_supplier_to_arc_default() {
        // Test that the default implementation of to_arc works
        // correctly for custom Clone types
        let supplier = CounterSupplier::new(300);
        let arc = supplier.to_arc();

        assert_eq!(arc.get(), 300);
        assert_eq!(arc.get(), 300);

        // Verify that Arc can be cloned
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.get(), 300);
    }

    #[test]
    fn test_custom_supplier_to_fn_default() {
        // Test that the default implementation of to_fn works
        // correctly for custom Clone types
        let supplier = CounterSupplier::new(42);
        let mut fn_mut = supplier.to_fn();

        assert_eq!(fn_mut(), 42);
        assert_eq!(fn_mut(), 42);
        assert_eq!(fn_mut(), 42);
    }

    #[test]
    fn test_custom_supplier_to_arc_thread_safety() {
        // Test that the Arc variant created from custom supplier
        // using to_arc is thread-safe
        let supplier = CounterSupplier::new(999);
        let arc = supplier.to_arc();

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let a = arc.clone();
                thread::spawn(move || a.get())
            })
            .collect();

        for h in handles {
            assert_eq!(h.join().unwrap(), 999);
        }
    }

    // Implement Clone for CounterSupplier to enable to_* methods
    impl Clone for CounterSupplier {
        fn clone(&self) -> Self {
            Self { value: self.value }
        }
    }
}

// ======================================================================
// Tests for to_* Methods
// ======================================================================

#[cfg(test)]
mod test_to_methods {
    use super::*;

    // ============================================================
    // Tests for ArcSupplier to_* methods
    // ============================================================

    mod test_arc_readonly_supplier_to_methods {
        use super::*;

        #[test]
        fn test_arc_to_box() {
            // Test ArcSupplier::to_box
            let arc = ArcSupplier::new(|| 42);
            let boxed = arc.to_box();

            assert_eq!(boxed.get(), 42);
            assert_eq!(boxed.get(), 42);

            // Original arc is still usable
            assert_eq!(arc.get(), 42);
        }

        #[test]
        fn test_arc_to_rc() {
            // Test ArcSupplier::to_rc
            let arc = ArcSupplier::new(|| 100);
            let rc = arc.to_rc();

            assert_eq!(rc.get(), 100);
            assert_eq!(rc.get(), 100);

            // Original arc is still usable
            assert_eq!(arc.get(), 100);
        }

        #[test]
        fn test_arc_to_arc() {
            // Test ArcSupplier::to_arc (optimized clone)
            let arc1 = ArcSupplier::new(|| 200);
            let arc2 = arc1.to_arc();

            assert_eq!(arc1.get(), 200);
            assert_eq!(arc2.get(), 200);

            // Both are still usable
            assert_eq!(arc1.get(), 200);
            assert_eq!(arc2.get(), 200);
        }

        #[test]
        fn test_arc_to_fn() {
            // Test ArcSupplier::to_fn
            let arc = ArcSupplier::new(|| 42);
            let mut fn_mut = arc.to_fn();

            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);

            // Original arc is still usable
            assert_eq!(arc.get(), 42);
        }

        #[test]
        fn test_arc_to_methods_with_string() {
            // Test to_* methods with String type
            let arc = ArcSupplier::new(|| String::from("Hello"));

            let boxed = arc.to_box();
            assert_eq!(boxed.get(), "Hello");

            let rc = arc.to_rc();
            assert_eq!(rc.get(), "Hello");

            let arc2 = arc.to_arc();
            assert_eq!(arc2.get(), "Hello");

            let mut fn_mut = arc.to_fn();
            assert_eq!(fn_mut(), "Hello");

            // Original arc is still usable
            assert_eq!(arc.get(), "Hello");
        }

        #[test]
        fn test_arc_to_arc_thread_safety() {
            // Test that to_arc result is thread-safe
            let arc1 = ArcSupplier::new(|| 999);
            let arc2 = arc1.to_arc();

            let handles: Vec<_> = (0..5)
                .map(|_| {
                    let a = arc2.clone();
                    thread::spawn(move || a.get())
                })
                .collect();

            for h in handles {
                assert_eq!(h.join().unwrap(), 999);
            }
        }
    }

    // ============================================================
    // Tests for RcSupplier to_* methods
    // ============================================================

    mod test_rc_readonly_supplier_to_methods {
        use super::*;

        #[test]
        fn test_rc_to_box() {
            // Test RcSupplier::to_box
            let rc = RcSupplier::new(|| 42);
            let boxed = rc.to_box();

            assert_eq!(boxed.get(), 42);
            assert_eq!(boxed.get(), 42);

            // Original rc is still usable
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_rc_to_rc() {
            // Test RcSupplier::to_rc (optimized clone)
            let rc1 = RcSupplier::new(|| 100);
            let rc2 = rc1.to_rc();

            assert_eq!(rc1.get(), 100);
            assert_eq!(rc2.get(), 100);

            // Both are still usable
            assert_eq!(rc1.get(), 100);
            assert_eq!(rc2.get(), 100);
        }

        #[test]
        fn test_rc_to_fn() {
            // Test RcSupplier::to_fn
            let rc = RcSupplier::new(|| 42);
            let mut fn_mut = rc.to_fn();

            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);

            // Original rc is still usable
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_rc_to_methods_with_string() {
            // Test to_* methods with String type
            let rc = RcSupplier::new(|| String::from("Hello"));

            let boxed = rc.to_box();
            assert_eq!(boxed.get(), "Hello");

            let rc2 = rc.to_rc();
            assert_eq!(rc2.get(), "Hello");

            let mut fn_mut = rc.to_fn();
            assert_eq!(fn_mut(), "Hello");

            // Original rc is still usable
            assert_eq!(rc.get(), "Hello");
        }

        // Note: to_arc is not implemented for RcSupplier
        // because Rc is not Send + Sync. If you try to call it,
        // the compiler will fail with a trait bound error.
    }

    // ============================================================
    // Tests for Closure to_* methods
    // ============================================================

    mod test_closure_to_methods {
        use super::*;

        #[test]
        fn test_closure_to_box() {
            // Test closure to_box
            let closure = || 42;
            let boxed = closure.to_box();

            assert_eq!(boxed.get(), 42);
            assert_eq!(boxed.get(), 42);

            // Original closure is still usable
            assert_eq!(closure.get(), 42);
        }

        #[test]
        fn test_closure_to_rc() {
            // Test closure to_rc
            let closure = || 100;
            let rc = closure.to_rc();

            assert_eq!(rc.get(), 100);
            assert_eq!(rc.get(), 100);

            // Original closure is still usable
            assert_eq!(closure.get(), 100);
        }

        #[test]
        fn test_closure_to_arc() {
            // Test closure to_arc
            let closure = || 200;
            let arc = closure.to_arc();

            assert_eq!(arc.get(), 200);
            assert_eq!(arc.get(), 200);

            // Original closure is still usable
            assert_eq!(closure.get(), 200);
        }

        #[test]
        fn test_closure_to_fn() {
            // Test closure to_fn
            let closure = || 42;
            let mut fn_mut = closure.to_fn();

            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);

            // Original closure is still usable
            assert_eq!(closure.get(), 42);
        }

        #[test]
        fn test_closure_to_methods_with_captured_value() {
            // Test to_* methods with captured value
            let value = 100;
            let closure = move || value * 2;

            let boxed = closure.to_box();
            assert_eq!(boxed.get(), 200);

            let rc = closure.to_rc();
            assert_eq!(rc.get(), 200);

            let arc = closure.to_arc();
            assert_eq!(arc.get(), 200);

            let mut fn_mut = closure.to_fn();
            assert_eq!(fn_mut(), 200);

            // Original closure is still usable
            assert_eq!(closure.get(), 200);
        }

        #[test]
        fn test_closure_to_arc_thread_safety() {
            // Test that to_arc result is thread-safe
            let closure = || 999;
            let arc = closure.to_arc();

            let handles: Vec<_> = (0..5)
                .map(|_| {
                    let a = arc.clone();
                    thread::spawn(move || a.get())
                })
                .collect();

            for h in handles {
                assert_eq!(h.join().unwrap(), 999);
            }
        }
    }

    // ============================================================
    // Note: BoxSupplier does not implement to_* methods
    // ============================================================
    //
    // BoxSupplier cannot implement to_* methods because
    // it does not implement Clone. Box provides unique ownership
    // and cannot be cloned unless the inner type implements Clone,
    // which dyn Fn() -> T does not.
    //
    // If you try to call to_box, to_rc, to_arc, or to_fn on
    // BoxSupplier, the compiler will fail with an error
    // indicating that BoxSupplier<T> does not implement
    // Clone, which is required by the default implementations.
}
