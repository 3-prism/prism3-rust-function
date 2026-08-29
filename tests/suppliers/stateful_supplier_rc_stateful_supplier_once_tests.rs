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
mod test_rc_stateful_supplier_once {
    use super::Rc;
    use super::RcStatefulSupplier;
    use super::RefCell;
    use super::StatefulSupplier;

    mod test_get {
        use super::Rc;
        use super::RcStatefulSupplier;
        use super::RefCell;
        use super::StatefulSupplier;

        #[test]
        fn test_consumes_stateful_supplier() {
            let mut supplier = RcStatefulSupplier::new(|| 42);
            let value = supplier.get();
            assert_eq!(value, 42);
            // StatefulSupplier is consumed, cannot be used again
        }

        #[test]
        fn test_with_string() {
            let mut supplier = RcStatefulSupplier::new(|| String::from("hello"));
            let value = supplier.get();
            assert_eq!(value, "hello");
        }

        #[test]
        fn test_with_vec() {
            let mut supplier = RcStatefulSupplier::new(|| vec![1, 2, 3]);
            let value = supplier.get();
            assert_eq!(value, vec![1, 2, 3]);
        }

        #[test]
        fn test_with_shared_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);
            let mut supplier = RcStatefulSupplier::new(move || {
                let mut c = counter_clone.borrow_mut();
                *c += 1;
                *c
            });
            let value = supplier.get();
            assert_eq!(value, 1);
            assert_eq!(*counter.borrow(), 1);
        }

        #[test]
        fn test_cloned_stateful_suppliers_share_state() {
            let counter = Rc::new(RefCell::new(0));
            let counter_clone1 = Rc::clone(&counter);

            let stateful_supplier1 = RcStatefulSupplier::new(move || {
                let mut c = counter_clone1.borrow_mut();
                *c += 1;
                *c
            });

            let stateful_supplier2 = stateful_supplier1.clone();

            let mut stateful_supplier1 = stateful_supplier1;
            let mut stateful_supplier2 = stateful_supplier2;
            let value1 = stateful_supplier1.get();
            let value2 = stateful_supplier2.get();

            // Both should increment the same counter
            assert_eq!(value1 + value2, 3); // 1 + 2
            assert_eq!(*counter.borrow(), 2);
        }
    }
}
// ==========================================================================
// Concrete wrapper composition tests
// ==========================================================================
