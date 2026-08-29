// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for Supplier types

use std::sync::Arc;
use std::thread;

use qubit_function::ArcSupplier;
use qubit_function::ArcTransformer;
use qubit_function::BoxSupplier;
use qubit_function::BoxTransformer;
use qubit_function::RcSupplier;
use qubit_function::RcTransformer;
use qubit_function::Supplier;

// ======================================================================
// Supplier Trait Tests (for closures)
// ======================================================================

#[cfg(test)]
mod test_integration {
    use super::Arc;
    use super::ArcSupplier;
    use super::BoxSupplier;
    use super::Supplier;
    use super::thread;

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
            assert_eq!(h.join().expect("thread should not panic"), vec![1, 2, 3, 4, 5]);
        }
    }

    #[test]
    fn test_mixed_transformations() {
        // Test combining multiple transformation methods
        let pipeline = BoxSupplier::new(|| 10)
            .map(|x| x * 2)
            .filter(|x: &i32| *x > 15)
            .map(|opt: Option<i32>| opt.map(|x| x.to_string()));

        assert_eq!(pipeline.get(), Some(String::from("20")));
    }
}

// ======================================================================
// Map with Transformer Tests - BoxSupplier
// ======================================================================
