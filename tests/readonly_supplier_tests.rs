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
    ArcReadonlySupplier, BoxReadonlySupplier, RcReadonlySupplier, ReadonlySupplier,
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
            let mapped = BoxReadonlySupplier::new(|| 42).map(|x| x.to_string());
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
            .map(|opt| opt.map(|x| x.to_string()));

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
