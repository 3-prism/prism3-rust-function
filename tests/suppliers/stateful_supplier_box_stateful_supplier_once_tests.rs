// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for StatefulSupplier types

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use qubit_function::ArcStatefulSupplier;
use qubit_function::BoxStatefulSupplier;
use qubit_function::RcStatefulSupplier;
use qubit_function::StatefulSupplier;

// ==========================================================================
// StatefulSupplier Trait Tests (for closures)
// ==========================================================================

#[cfg(test)]
mod test_box_stateful_supplier_once {
    use super::BoxStatefulSupplier;
    use super::StatefulSupplier;

    mod test_get {
        use super::BoxStatefulSupplier;
        use super::StatefulSupplier;

        #[test]
        fn test_consumes_stateful_supplier() {
            let mut supplier = BoxStatefulSupplier::new(|| 42);
            let value = supplier.get();
            assert_eq!(value, 42);
            // StatefulSupplier is consumed, cannot be used again
        }

        #[test]
        fn test_with_string() {
            let mut supplier = BoxStatefulSupplier::new(|| String::from("hello"));
            let value = supplier.get();
            assert_eq!(value, "hello");
        }

        #[test]
        fn test_with_vec() {
            let mut supplier = BoxStatefulSupplier::new(|| vec![1, 2, 3]);
            let value = supplier.get();
            assert_eq!(value, vec![1, 2, 3]);
        }

        #[test]
        fn test_moves_captured_value() {
            let data = String::from("captured");
            let mut supplier = BoxStatefulSupplier::new(move || data.clone());
            let value = supplier.get();
            assert_eq!(value, "captured");
        }

        #[test]
        fn test_with_stateful_closure() {
            let mut counter = 0;
            let mut supplier = BoxStatefulSupplier::new(move || {
                counter += 1;
                counter
            });
            let value = supplier.get();
            assert_eq!(value, 1);
        }
    }
}

// ==========================================================================
// SupplierOnce Implementation Tests for ArcStatefulSupplier
// ==========================================================================
