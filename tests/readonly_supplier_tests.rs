/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for ReadonlySupplier types

use prism3_function::{
    ArcReadonlySupplier, ArcTransformer, BoxReadonlySupplier, BoxTransformer, RcReadonlySupplier,
    RcTransformer, ReadonlySupplier,
};
use std::sync::Arc;
use std::thread;

// ======================================================================
// ReadonlySupplier Trait Tests (for closures)
// ======================================================================

#[cfg(test)]
mod test_readonly_supplier_trait {
    use super::*;

    #[test]
    fn test_closure_implements_readonly_supplier() {
        // Test that closure implements ReadonlySupplier trait
        let closure = || 42;
        let boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_closure_stateless() {
        // Test stateless closure (always returns same value)
        let boxed = BoxReadonlySupplier::new(|| 42);
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_box() {
        // Test conversion to BoxReadonlySupplier
        let closure = || 42;
        let boxed = closure.into_box();
        assert_eq!(boxed.get(), 42);
    }

    #[test]
    fn test_into_rc() {
        // Test conversion to RcReadonlySupplier
        let closure = || 42;
        let rc = closure.into_rc();
        assert_eq!(rc.get(), 42);
    }

    #[test]
    fn test_into_arc() {
        // Test conversion to ArcReadonlySupplier
        let closure = || 42;
        let arc = closure.into_arc();
        assert_eq!(arc.get(), 42);
    }

    #[test]
    fn test_closure_get() {
        // Test the get method in impl<T, F> ReadonlySupplier<T>
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
// BoxReadonlySupplier Tests
// ======================================================================

#[cfg(test)]
mod test_box_readonly_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_new_basic() {
            // Test creating a new BoxReadonlySupplier
            let supplier = BoxReadonlySupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_new_with_closure() {
            // Test with a closure that captures variables
            let value = 100;
            let supplier = BoxReadonlySupplier::new(move || value);
            assert_eq!(supplier.get(), 100);
        }

        #[test]
        fn test_new_returns_same_value() {
            // Test that successive calls return same value
            let supplier = BoxReadonlySupplier::new(|| 42);
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
            let constant = BoxReadonlySupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_constant_string() {
            // Test constant with String type
            let constant = BoxReadonlySupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }

        #[test]
        fn test_constant_vec() {
            // Test constant with Vec type
            let constant = BoxReadonlySupplier::constant(vec![1, 2, 3]);
            assert_eq!(constant.get(), vec![1, 2, 3]);
            assert_eq!(constant.get(), vec![1, 2, 3]);
        }
    }

    mod test_map {
        use super::*;

        #[test]
        fn test_map_basic() {
            // Test map transformation
            let mapped = BoxReadonlySupplier::new(|| 10).map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_map_chain() {
            // Test chained map operations
            let pipeline = BoxReadonlySupplier::new(|| 10)
                .map(|x| x * 2)
                .map(|x| x + 5);
            assert_eq!(pipeline.get(), 25);
        }

        #[test]
        fn test_map_type_conversion() {
            // Test map with type conversion
            let mapped = BoxReadonlySupplier::new(|| 42).map(|x: i32| x.to_string());
            assert_eq!(mapped.get(), "42");
        }
    }

    mod test_filter {
        use super::*;

        #[test]
        fn test_filter_passes() {
            // Test filter that passes
            let filtered = BoxReadonlySupplier::new(|| 42).filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let filtered = BoxReadonlySupplier::new(|| 43).filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }

        #[test]
        fn test_filter_with_map() {
            // Test combining filter and map
            let pipeline = BoxReadonlySupplier::new(|| 10)
                .map(|x| x * 2)
                .filter(|x| *x > 15);
            assert_eq!(pipeline.get(), Some(20));
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = BoxReadonlySupplier::new(|| 42);
            let second = BoxReadonlySupplier::new(|| "hello");
            let zipped = first.zip(second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_different_types() {
            // Test zipping suppliers of different types
            let first = BoxReadonlySupplier::new(|| 100);
            let second = BoxReadonlySupplier::new(|| vec![1, 2, 3]);
            let zipped = first.zip(second);
            assert_eq!(zipped.get(), (100, vec![1, 2, 3]));
        }
    }

    mod test_trait_methods {
        use super::*;

        #[test]
        fn test_get() {
            // Test ReadonlySupplier::get method
            let supplier = BoxReadonlySupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_into_box() {
            // Test into_box (should return self)
            let supplier = BoxReadonlySupplier::new(|| 42);
            let boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_into_rc() {
            // Test conversion to RcReadonlySupplier
            let supplier = BoxReadonlySupplier::new(|| 42);
            let rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_into_fn() {
            // Test conversion to FnMut closure
            let supplier = BoxReadonlySupplier::new(|| 42);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);
        }

        #[test]
        fn test_into_fn_with_captured_value() {
            // Test into_fn with captured value
            let value = 100;
            let supplier = BoxReadonlySupplier::new(move || value * 2);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 200);
            assert_eq!(fn_mut(), 200);
        }

        #[test]
        fn test_into_fn_with_string() {
            // Test into_fn with String type
            let supplier = BoxReadonlySupplier::new(|| String::from("hello"));
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), "hello");
            assert_eq!(fn_mut(), "hello");
        }

        // Note: test_into_arc is not included here because
        // BoxReadonlySupplier cannot be converted to
        // ArcReadonlySupplier (inner function may not be Send +
        // Sync). This is enforced at compile time by trait bounds.
    }
}

// ======================================================================
// ArcReadonlySupplier Tests
// ======================================================================

#[cfg(test)]
mod test_arc_readonly_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_new_basic() {
            // Test creating a new ArcReadonlySupplier
            let supplier = ArcReadonlySupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_new_with_closure() {
            // Test with a closure that captures variables
            let value = 100;
            let supplier = ArcReadonlySupplier::new(move || value);
            assert_eq!(supplier.get(), 100);
        }

        #[test]
        fn test_new_returns_same_value() {
            // Test that successive calls return same value
            let supplier = ArcReadonlySupplier::new(|| 42);
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
            let constant = ArcReadonlySupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_constant_string() {
            // Test constant with String type
            let constant = ArcReadonlySupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }
    }

    mod test_map {
        use super::*;

        #[test]
        fn test_map_basic() {
            // Test map transformation
            let source = ArcReadonlySupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_map_chain() {
            // Test chained map operations
            let source = ArcReadonlySupplier::new(|| 10);
            let pipeline = source.map(|x| x * 2).map(|x| x + 5);
            assert_eq!(pipeline.get(), 25);
        }

        #[test]
        fn test_map_preserves_original() {
            // Test that mapping doesn't consume original
            let source = ArcReadonlySupplier::new(|| 10);
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
            let source = ArcReadonlySupplier::new(|| 42);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let source = ArcReadonlySupplier::new(|| 43);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = ArcReadonlySupplier::new(|| 42);
            let second = ArcReadonlySupplier::new(|| "hello");
            let zipped = first.zip(&second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_preserves_originals() {
            // Test that zip doesn't consume originals
            let first = ArcReadonlySupplier::new(|| 42);
            let second = ArcReadonlySupplier::new(|| "hello");
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
            let original = ArcReadonlySupplier::new(|| 42);
            let cloned = original.clone();
            assert_eq!(original.get(), 42);
            assert_eq!(cloned.get(), 42);
        }

        #[test]
        fn test_clone_shares_function() {
            // Test that clone shares the underlying function
            let original = ArcReadonlySupplier::new(|| String::from("hello"));
            let cloned = original.clone();
            assert_eq!(original.get(), cloned.get());
        }
    }

    mod test_thread_safety {
        use super::*;

        #[test]
        fn test_send_between_threads() {
            // Test that supplier can be sent between threads
            let supplier = ArcReadonlySupplier::new(|| 42);
            let handle = thread::spawn(move || supplier.get());
            assert_eq!(handle.join().unwrap(), 42);
        }

        #[test]
        fn test_concurrent_access() {
            // Test lock-free concurrent access
            let factory = ArcReadonlySupplier::new(|| String::from("Hello, World!"));

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
            let supplier = Arc::new(ArcReadonlySupplier::new(|| 100));

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
            // Test ReadonlySupplier::get method
            let supplier = ArcReadonlySupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_into_box() {
            // Test conversion to BoxReadonlySupplier
            let supplier = ArcReadonlySupplier::new(|| 42);
            let boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_into_rc() {
            // Test conversion to RcReadonlySupplier
            let supplier = ArcReadonlySupplier::new(|| 42);
            let rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_into_arc() {
            // Test into_arc (should return self)
            let supplier = ArcReadonlySupplier::new(|| 42);
            let arc = supplier.into_arc();
            assert_eq!(arc.get(), 42);
        }

        #[test]
        fn test_into_fn() {
            // Test conversion to FnMut closure
            let supplier = ArcReadonlySupplier::new(|| 42);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);
        }

        #[test]
        fn test_into_fn_with_captured_value() {
            // Test into_fn with captured value
            let value = 100;
            let supplier = ArcReadonlySupplier::new(move || value * 2);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 200);
            assert_eq!(fn_mut(), 200);
        }

        #[test]
        fn test_into_fn_with_string() {
            // Test into_fn with String type
            let supplier = ArcReadonlySupplier::new(|| String::from("hello"));
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), "hello");
            assert_eq!(fn_mut(), "hello");
        }

        #[test]
        fn test_into_fn_thread_safe() {
            // Test that into_fn result can be sent to another thread
            let supplier = ArcReadonlySupplier::new(|| 42);
            let mut fn_mut = supplier.into_fn();
            let handle = thread::spawn(move || fn_mut());
            assert_eq!(handle.join().unwrap(), 42);
        }
    }
}

// ======================================================================
// RcReadonlySupplier Tests
// ======================================================================

#[cfg(test)]
mod test_rc_readonly_supplier {
    use super::*;

    mod test_new {
        use super::*;

        #[test]
        fn test_new_basic() {
            // Test creating a new RcReadonlySupplier
            let supplier = RcReadonlySupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_new_with_closure() {
            // Test with a closure that captures variables
            let value = 100;
            let supplier = RcReadonlySupplier::new(move || value);
            assert_eq!(supplier.get(), 100);
        }

        #[test]
        fn test_new_returns_same_value() {
            // Test that successive calls return same value
            let supplier = RcReadonlySupplier::new(|| 42);
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
            let constant = RcReadonlySupplier::constant(42);
            assert_eq!(constant.get(), 42);
            assert_eq!(constant.get(), 42);
        }

        #[test]
        fn test_constant_string() {
            // Test constant with String type
            let constant = RcReadonlySupplier::constant(String::from("hello"));
            assert_eq!(constant.get(), "hello");
            assert_eq!(constant.get(), "hello");
        }
    }

    mod test_map {
        use super::*;

        #[test]
        fn test_map_basic() {
            // Test map transformation
            let source = RcReadonlySupplier::new(|| 10);
            let mapped = source.map(|x| x * 2);
            assert_eq!(mapped.get(), 20);
        }

        #[test]
        fn test_map_chain() {
            // Test chained map operations
            let source = RcReadonlySupplier::new(|| 10);
            let pipeline = source.map(|x| x * 2).map(|x| x + 5);
            assert_eq!(pipeline.get(), 25);
        }

        #[test]
        fn test_map_preserves_original() {
            // Test that mapping doesn't consume original
            let source = RcReadonlySupplier::new(|| 10);
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
            let source = RcReadonlySupplier::new(|| 42);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), Some(42));
        }

        #[test]
        fn test_filter_fails() {
            // Test filter that fails
            let source = RcReadonlySupplier::new(|| 43);
            let filtered = source.filter(|x| x % 2 == 0);
            assert_eq!(filtered.get(), None);
        }
    }

    mod test_zip {
        use super::*;

        #[test]
        fn test_zip_basic() {
            // Test zipping two suppliers
            let first = RcReadonlySupplier::new(|| 42);
            let second = RcReadonlySupplier::new(|| "hello");
            let zipped = first.zip(&second);
            assert_eq!(zipped.get(), (42, "hello"));
        }

        #[test]
        fn test_zip_preserves_originals() {
            // Test that zip doesn't consume originals
            let first = RcReadonlySupplier::new(|| 42);
            let second = RcReadonlySupplier::new(|| "hello");
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
            let original = RcReadonlySupplier::new(|| 42);
            let cloned = original.clone();
            assert_eq!(original.get(), 42);
            assert_eq!(cloned.get(), 42);
        }

        #[test]
        fn test_clone_shares_function() {
            // Test that clone shares the underlying function
            let original = RcReadonlySupplier::new(|| String::from("hello"));
            let cloned = original.clone();
            assert_eq!(original.get(), cloned.get());
        }
    }

    mod test_trait_methods {
        use super::*;

        #[test]
        fn test_get() {
            // Test ReadonlySupplier::get method
            let supplier = RcReadonlySupplier::new(|| 42);
            assert_eq!(supplier.get(), 42);
        }

        #[test]
        fn test_into_box() {
            // Test conversion to BoxReadonlySupplier
            let supplier = RcReadonlySupplier::new(|| 42);
            let boxed = supplier.into_box();
            assert_eq!(boxed.get(), 42);
        }

        #[test]
        fn test_into_rc() {
            // Test into_rc (should return self)
            let supplier = RcReadonlySupplier::new(|| 42);
            let rc = supplier.into_rc();
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_into_fn() {
            // Test conversion to FnMut closure
            let supplier = RcReadonlySupplier::new(|| 42);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);
        }

        #[test]
        fn test_into_fn_with_captured_value() {
            // Test into_fn with captured value
            let value = 100;
            let supplier = RcReadonlySupplier::new(move || value * 2);
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), 200);
            assert_eq!(fn_mut(), 200);
        }

        #[test]
        fn test_into_fn_with_string() {
            // Test into_fn with String type
            let supplier = RcReadonlySupplier::new(|| String::from("hello"));
            let mut fn_mut = supplier.into_fn();
            assert_eq!(fn_mut(), "hello");
            assert_eq!(fn_mut(), "hello");
        }

        // Note: test_into_arc is not included here because
        // RcReadonlySupplier cannot be converted to
        // ArcReadonlySupplier (Rc is not Send + Sync). This is
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
            error_supplier: ArcReadonlySupplier<String>,
        }

        impl Executor {
            fn execute(&self) -> Result<(), String> {
                // Can call supplier in &self method!
                Err(self.error_supplier.get())
            }
        }

        let executor = Executor {
            error_supplier: ArcReadonlySupplier::new(|| String::from("Error occurred")),
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

        let factory = BoxReadonlySupplier::new(|| Config { timeout: 30 });

        let config1 = factory.get();
        let config2 = factory.get();

        assert_eq!(config1, Config { timeout: 30 });
        assert_eq!(config2, Config { timeout: 30 });
    }

    #[test]
    fn test_concurrent_factory() {
        // Test using as factory in concurrent context
        let factory = Arc::new(ArcReadonlySupplier::new(|| vec![1, 2, 3, 4, 5]));

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
        let pipeline = BoxReadonlySupplier::new(|| 10)
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
// Map with Transformer Tests - BoxReadonlySupplier
// ======================================================================

#[cfg(test)]
mod test_box_readonly_supplier_map_with_transformer {
    use super::*;

    // 辅助函数指针
    fn double(x: i32) -> i32 {
        x * 2
    }

    fn to_string(x: i32) -> String {
        x.to_string()
    }

    #[test]
    fn test_map_with_closure() {
        // 测试 map 接受闭包
        let supplier = BoxReadonlySupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_function_pointer() {
        // 测试 map 接受函数指针
        let supplier = BoxReadonlySupplier::new(|| 10);
        let mapped = supplier.map(double);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_box_transformer() {
        // 测试 map 接受 BoxTransformer 对象
        let supplier = BoxReadonlySupplier::new(|| 10);
        let transformer = BoxTransformer::new(|x| x * 3);
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_chain_with_different_types() {
        // 测试链式调用，每个 map 使用不同类型的 transformer
        let supplier = BoxReadonlySupplier::new(|| 10);
        let step1 = supplier.map(|x| x * 2); // 闭包
        let step2 = step1.map(double); // 函数指针
        let step3 = step2.map(BoxTransformer::new(|x| x + 5)); // BoxTransformer
        assert_eq!(step3.get(), 45); // (10 * 2) * 2 + 5 = 45
    }

    #[test]
    fn test_map_with_closure_capturing_variables() {
        // 测试 map 使用捕获变量的闭包
        let multiplier = 3;
        let supplier = BoxReadonlySupplier::new(|| 10);
        let mapped = supplier.map(move |x| x * multiplier);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_with_type_conversion() {
        // 测试 map 进行类型转换
        let supplier = BoxReadonlySupplier::new(|| 42);

        // 使用闭包转换类型
        let mapped1 = supplier.map(|x: i32| x.to_string());
        assert_eq!(mapped1.get(), "42");

        // 使用 BoxTransformer 转换类型
        let supplier2 = BoxReadonlySupplier::new(|| 42);
        let transformer = BoxTransformer::new(to_string);
        let mapped2 = supplier2.map(transformer);
        assert_eq!(mapped2.get(), "42");
    }

    #[test]
    fn test_map_with_complex_transformer() {
        // 测试 map 使用复杂的 Transformer
        #[derive(Debug, PartialEq)]
        struct Data {
            value: i32,
        }

        let supplier = BoxReadonlySupplier::new(|| 10);
        let transformer = BoxTransformer::new(|x| Data { value: x * 2 });
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), Data { value: 20 });
    }
}

// ======================================================================
// Map with Transformer Tests - ArcReadonlySupplier
// ======================================================================

#[cfg(test)]
mod test_arc_readonly_supplier_map_with_transformer {
    use super::*;

    // 辅助函数指针
    fn double(x: i32) -> i32 {
        x * 2
    }

    fn to_string(x: i32) -> String {
        x.to_string()
    }

    #[test]
    fn test_map_with_closure() {
        // 测试 map 接受闭包
        let supplier = ArcReadonlySupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_function_pointer() {
        // 测试 map 接受函数指针
        let supplier = ArcReadonlySupplier::new(|| 10);
        let mapped = supplier.map(double);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_arc_transformer() {
        // 测试 map 接受 ArcTransformer 对象
        let supplier = ArcReadonlySupplier::new(|| 10);
        let transformer = ArcTransformer::new(|x| x * 3);
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_chain_with_different_types() {
        // 测试链式调用，每个 map 使用不同类型的 transformer
        let supplier = ArcReadonlySupplier::new(|| 10);
        let step1 = supplier.map(|x| x * 2); // 闭包
        let step2 = step1.map(double); // 函数指针
        let step3 = step2.map(ArcTransformer::new(|x| x + 5)); // ArcTransformer
        assert_eq!(step3.get(), 45); // (10 * 2) * 2 + 5 = 45
    }

    #[test]
    fn test_map_with_closure_capturing_variables() {
        // 测试 map 使用捕获变量的闭包
        let multiplier = 3;
        let supplier = ArcReadonlySupplier::new(|| 10);
        let mapped = supplier.map(move |x| x * multiplier);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_preserves_original_with_transformer() {
        // 测试使用 transformer 后原 supplier 仍可用
        let supplier = ArcReadonlySupplier::new(|| 10);
        let transformer = ArcTransformer::new(|x| x * 2);
        let mapped = supplier.map(transformer);

        // 原 supplier 仍然可用
        assert_eq!(supplier.get(), 10);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_thread_safety_with_transformer() {
        // 测试带 transformer 的 map 在多线程环境下的表现
        let supplier = ArcReadonlySupplier::new(|| 10);
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
        // 测试 map 进行类型转换
        let supplier = ArcReadonlySupplier::new(|| 42);

        // 使用闭包转换类型
        let mapped1 = supplier.map(|x: i32| x.to_string());
        assert_eq!(mapped1.get(), "42");

        // 使用 ArcTransformer 转换类型
        let transformer = ArcTransformer::new(to_string);
        let mapped2 = supplier.map(transformer);
        assert_eq!(mapped2.get(), "42");
    }

    #[test]
    fn test_map_with_shared_transformer() {
        // 测试多个 supplier 共享同一个 transformer
        let supplier1 = ArcReadonlySupplier::new(|| 10);
        let supplier2 = ArcReadonlySupplier::new(|| 20);

        let transformer = ArcTransformer::new(|x| x * 2);
        let mapped1 = supplier1.map(transformer.clone());
        let mapped2 = supplier2.map(transformer);

        assert_eq!(mapped1.get(), 20);
        assert_eq!(mapped2.get(), 40);
    }
}

// ======================================================================
// Map with Transformer Tests - RcReadonlySupplier
// ======================================================================

#[cfg(test)]
mod test_rc_readonly_supplier_map_with_transformer {
    use super::*;

    // 辅助函数指针
    fn double(x: i32) -> i32 {
        x * 2
    }

    fn to_string(x: i32) -> String {
        x.to_string()
    }

    #[test]
    fn test_map_with_closure() {
        // 测试 map 接受闭包
        let supplier = RcReadonlySupplier::new(|| 10);
        let mapped = supplier.map(|x| x * 2);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_function_pointer() {
        // 测试 map 接受函数指针
        let supplier = RcReadonlySupplier::new(|| 10);
        let mapped = supplier.map(double);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_rc_transformer() {
        // 测试 map 接受 RcTransformer 对象
        let supplier = RcReadonlySupplier::new(|| 10);
        let transformer = RcTransformer::new(|x| x * 3);
        let mapped = supplier.map(transformer);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_chain_with_different_types() {
        // 测试链式调用，每个 map 使用不同类型的 transformer
        let supplier = RcReadonlySupplier::new(|| 10);
        let step1 = supplier.map(|x| x * 2); // 闭包
        let step2 = step1.map(double); // 函数指针
        let step3 = step2.map(RcTransformer::new(|x| x + 5)); // RcTransformer
        assert_eq!(step3.get(), 45); // (10 * 2) * 2 + 5 = 45
    }

    #[test]
    fn test_map_with_closure_capturing_variables() {
        // 测试 map 使用捕获变量的闭包
        let multiplier = 3;
        let supplier = RcReadonlySupplier::new(|| 10);
        let mapped = supplier.map(move |x| x * multiplier);
        assert_eq!(mapped.get(), 30);
    }

    #[test]
    fn test_map_preserves_original_with_transformer() {
        // 测试使用 transformer 后原 supplier 仍可用
        let supplier = RcReadonlySupplier::new(|| 10);
        let transformer = RcTransformer::new(|x| x * 2);
        let mapped = supplier.map(transformer);

        // 原 supplier 仍然可用
        assert_eq!(supplier.get(), 10);
        assert_eq!(mapped.get(), 20);
    }

    #[test]
    fn test_map_with_type_conversion() {
        // 测试 map 进行类型转换
        let supplier = RcReadonlySupplier::new(|| 42);

        // 使用闭包转换类型
        let mapped1 = supplier.map(|x: i32| x.to_string());
        assert_eq!(mapped1.get(), "42");

        // 使用 RcTransformer 转换类型
        let transformer = RcTransformer::new(to_string);
        let mapped2 = supplier.map(transformer);
        assert_eq!(mapped2.get(), "42");
    }

    #[test]
    fn test_map_with_shared_transformer() {
        // 测试多个 supplier 共享同一个 transformer
        let supplier1 = RcReadonlySupplier::new(|| 10);
        let supplier2 = RcReadonlySupplier::new(|| 20);

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
        // 测试在管道中混合使用不同类型的 transformer
        let supplier = BoxReadonlySupplier::new(|| 5);

        let pipeline = supplier
            .map(|x| x * 2) // 闭包
            .map(|x: i32| -> i32 { x + 3 }) // 显式类型标注的闭包
            .map(|x: i32| x.to_string()); // 类型转换闭包

        assert_eq!(pipeline.get(), "13");
    }

    #[test]
    fn test_transformer_with_complex_logic() {
        // 测试包含复杂逻辑的 transformer
        #[derive(Debug, PartialEq)]
        struct Result {
            doubled: i32,
            squared: i32,
        }

        let supplier = ArcReadonlySupplier::new(|| 5);
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
        // 测试函数指针与泛型 supplier 的配合
        fn process(x: i32) -> String {
            format!("Value: {}", x * 2)
        }

        let supplier = ArcReadonlySupplier::new(|| 21);
        let mapped = supplier.map(process);
        assert_eq!(mapped.get(), "Value: 42");
    }

    #[test]
    fn test_transformer_reusability() {
        // 测试 Transformer 的可重用性
        let transformer = ArcTransformer::new(|x: i32| x * 10);

        let supplier1 = ArcReadonlySupplier::new(|| 1);
        let supplier2 = ArcReadonlySupplier::new(|| 2);
        let supplier3 = ArcReadonlySupplier::new(|| 3);

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

    /// A simple custom type that implements ReadonlySupplier with
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

    impl ReadonlySupplier<i32> for CounterSupplier {
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
    // Tests for ArcReadonlySupplier to_* methods
    // ============================================================

    mod test_arc_readonly_supplier_to_methods {
        use super::*;

        #[test]
        fn test_arc_to_box() {
            // Test ArcReadonlySupplier::to_box
            let arc = ArcReadonlySupplier::new(|| 42);
            let boxed = arc.to_box();

            assert_eq!(boxed.get(), 42);
            assert_eq!(boxed.get(), 42);

            // Original arc is still usable
            assert_eq!(arc.get(), 42);
        }

        #[test]
        fn test_arc_to_rc() {
            // Test ArcReadonlySupplier::to_rc
            let arc = ArcReadonlySupplier::new(|| 100);
            let rc = arc.to_rc();

            assert_eq!(rc.get(), 100);
            assert_eq!(rc.get(), 100);

            // Original arc is still usable
            assert_eq!(arc.get(), 100);
        }

        #[test]
        fn test_arc_to_arc() {
            // Test ArcReadonlySupplier::to_arc (optimized clone)
            let arc1 = ArcReadonlySupplier::new(|| 200);
            let arc2 = arc1.to_arc();

            assert_eq!(arc1.get(), 200);
            assert_eq!(arc2.get(), 200);

            // Both are still usable
            assert_eq!(arc1.get(), 200);
            assert_eq!(arc2.get(), 200);
        }

        #[test]
        fn test_arc_to_fn() {
            // Test ArcReadonlySupplier::to_fn
            let arc = ArcReadonlySupplier::new(|| 42);
            let mut fn_mut = arc.to_fn();

            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);

            // Original arc is still usable
            assert_eq!(arc.get(), 42);
        }

        #[test]
        fn test_arc_to_methods_with_string() {
            // Test to_* methods with String type
            let arc = ArcReadonlySupplier::new(|| {
                String::from("Hello")
            });

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
            let arc1 = ArcReadonlySupplier::new(|| 999);
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
    // Tests for RcReadonlySupplier to_* methods
    // ============================================================

    mod test_rc_readonly_supplier_to_methods {
        use super::*;

        #[test]
        fn test_rc_to_box() {
            // Test RcReadonlySupplier::to_box
            let rc = RcReadonlySupplier::new(|| 42);
            let boxed = rc.to_box();

            assert_eq!(boxed.get(), 42);
            assert_eq!(boxed.get(), 42);

            // Original rc is still usable
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_rc_to_rc() {
            // Test RcReadonlySupplier::to_rc (optimized clone)
            let rc1 = RcReadonlySupplier::new(|| 100);
            let rc2 = rc1.to_rc();

            assert_eq!(rc1.get(), 100);
            assert_eq!(rc2.get(), 100);

            // Both are still usable
            assert_eq!(rc1.get(), 100);
            assert_eq!(rc2.get(), 100);
        }

        #[test]
        fn test_rc_to_fn() {
            // Test RcReadonlySupplier::to_fn
            let rc = RcReadonlySupplier::new(|| 42);
            let mut fn_mut = rc.to_fn();

            assert_eq!(fn_mut(), 42);
            assert_eq!(fn_mut(), 42);

            // Original rc is still usable
            assert_eq!(rc.get(), 42);
        }

        #[test]
        fn test_rc_to_methods_with_string() {
            // Test to_* methods with String type
            let rc = RcReadonlySupplier::new(|| String::from("Hello"));

            let boxed = rc.to_box();
            assert_eq!(boxed.get(), "Hello");

            let rc2 = rc.to_rc();
            assert_eq!(rc2.get(), "Hello");

            let mut fn_mut = rc.to_fn();
            assert_eq!(fn_mut(), "Hello");

            // Original rc is still usable
            assert_eq!(rc.get(), "Hello");
        }

        // Note: to_arc is not implemented for RcReadonlySupplier
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
    // Note: BoxReadonlySupplier does not implement to_* methods
    // ============================================================
    //
    // BoxReadonlySupplier cannot implement to_* methods because
    // it does not implement Clone. Box provides unique ownership
    // and cannot be cloned unless the inner type implements Clone,
    // which dyn Fn() -> T does not.
    //
    // If you try to call to_box, to_rc, to_arc, or to_fn on
    // BoxReadonlySupplier, the compiler will fail with an error
    // indicating that BoxReadonlySupplier<T> does not implement
    // Clone, which is required by the default implementations.
}
