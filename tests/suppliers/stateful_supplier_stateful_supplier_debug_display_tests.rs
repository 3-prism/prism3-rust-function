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
mod test_stateful_supplier_debug_display {
    use super::ArcStatefulSupplier;
    use super::BoxStatefulSupplier;
    use super::RcStatefulSupplier;

    // ============================================================
    // BoxStatefulSupplier Debug and Display Tests
    // ============================================================

    mod test_box_stateful_supplier_debug_display {
        use super::BoxStatefulSupplier;

        #[test]
        fn test_debug_without_name() {
            // Test Debug formatting for BoxStatefulSupplier without name
            let supplier = BoxStatefulSupplier::new(|| 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("BoxStatefulSupplier"));
            assert!(debug_str.contains("name: None"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_debug_with_name() {
            // Test Debug formatting for BoxStatefulSupplier with name
            let supplier = BoxStatefulSupplier::new_with_name("test_supplier", || 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("BoxStatefulSupplier"));
            assert!(debug_str.contains("name: Some(\"test_supplier\")"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_display_without_name() {
            // Test Display formatting for BoxStatefulSupplier without name
            let supplier = BoxStatefulSupplier::new(|| 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "BoxStatefulSupplier");
        }

        #[test]
        fn test_display_with_name() {
            // Test Display formatting for BoxStatefulSupplier with name
            let supplier = BoxStatefulSupplier::new_with_name("test_supplier", || 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "BoxStatefulSupplier(test_supplier)");
        }
    }

    // ============================================================
    // ArcStatefulSupplier Debug and Display Tests
    // ============================================================

    mod test_arc_stateful_supplier_debug_display {
        use super::ArcStatefulSupplier;

        #[test]
        fn test_debug_without_name() {
            // Test Debug formatting for ArcStatefulSupplier without name
            let supplier = ArcStatefulSupplier::new(|| 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("ArcStatefulSupplier"));
            assert!(debug_str.contains("name: None"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_debug_with_name() {
            // Test Debug formatting for ArcStatefulSupplier with name
            let supplier = ArcStatefulSupplier::new_with_name("test_supplier", || 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("ArcStatefulSupplier"));
            assert!(debug_str.contains("name: Some(\"test_supplier\")"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_display_without_name() {
            // Test Display formatting for ArcStatefulSupplier without name
            let supplier = ArcStatefulSupplier::new(|| 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "ArcStatefulSupplier");
        }

        #[test]
        fn test_display_with_name() {
            // Test Display formatting for ArcStatefulSupplier with name
            let supplier = ArcStatefulSupplier::new_with_name("test_supplier", || 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "ArcStatefulSupplier(test_supplier)");
        }
    }

    // ============================================================
    // RcStatefulSupplier Debug and Display Tests
    // ============================================================

    mod test_rc_stateful_supplier_debug_display {
        use super::RcStatefulSupplier;

        #[test]
        fn test_debug_without_name() {
            // Test Debug formatting for RcStatefulSupplier without name
            let supplier = RcStatefulSupplier::new(|| 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("RcStatefulSupplier"));
            assert!(debug_str.contains("name: None"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_debug_with_name() {
            // Test Debug formatting for RcStatefulSupplier with name
            let supplier = RcStatefulSupplier::new_with_name("test_supplier", || 42);
            let debug_str = format!("{:?}", supplier);
            assert!(debug_str.contains("RcStatefulSupplier"));
            assert!(debug_str.contains("name: Some(\"test_supplier\")"));
            assert!(debug_str.contains("function: \"<function>\""));
        }

        #[test]
        fn test_display_without_name() {
            // Test Display formatting for RcStatefulSupplier without name
            let supplier = RcStatefulSupplier::new(|| 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "RcStatefulSupplier");
        }

        #[test]
        fn test_display_with_name() {
            // Test Display formatting for RcStatefulSupplier with name
            let supplier = RcStatefulSupplier::new_with_name("test_supplier", || 42);
            let display_str = format!("{}", supplier);
            assert_eq!(display_str, "RcStatefulSupplier(test_supplier)");
        }
    }
}

// ============================================================================
// StatefulSupplier Trait Default Methods Tests - into_once, to_once
// ============================================================================
