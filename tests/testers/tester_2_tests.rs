// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unit tests for Tester

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use qubit_function::testers::tester::ArcTester;
    use qubit_function::testers::tester::BoxTester;
    use qubit_function::testers::tester::RcTester;
    use qubit_function::testers::tester::Tester;

    // ========================================================================
    // BoxTester tests
    // ========================================================================

    /// Custom tester type that only implements the core test() method
    /// to verify that default implementations of into_xxx() methods
    /// work correctly.
    struct AlwaysTrueTester;

    impl Tester for AlwaysTrueTester {
        fn test(&self) -> bool {
            true
        }
    }

    /// Custom tester type that only implements the core test() method
    /// for thread-safe scenarios to test into_arc() default impl.
    #[derive(Clone)]
    struct ThreadSafeTester {
        value: Arc<AtomicBool>,
    }

    impl ThreadSafeTester {
        fn new(value: bool) -> Self {
            Self {
                value: Arc::new(AtomicBool::new(value)),
            }
        }
    }

    impl Tester for ThreadSafeTester {
        fn test(&self) -> bool {
            self.value.load(Ordering::Relaxed)
        }
    }

    // ========================================================================
    // Custom Tester implementation tests for default into_xxx/to_xxx methods
    // ========================================================================

    // Define a custom Tester that implements Clone and is Send + Sync
    #[derive(Clone)]
    struct CustomTester {
        value: Arc<AtomicBool>,
    }

    impl Tester for CustomTester {
        fn test(&self) -> bool {
            self.value.load(Ordering::Relaxed)
        }
    }

    // Test with a non-Clone custom tester to ensure into_xxx works
    // but to_xxx would fail at compile time
    struct NonCloneTester {
        value: Arc<AtomicBool>,
    }

    impl Tester for NonCloneTester {
        fn test(&self) -> bool {
            self.value.load(Ordering::Relaxed)
        }
    }
    #[test]
    fn test_precondition_check_scenario() {
        struct Operation {
            precondition: BoxTester,
        }

        impl Operation {
            fn execute(&self) -> Result<String, String> {
                if !self.precondition.test() {
                    return Err("Precondition not met".to_string());
                }
                Ok("Operation completed".to_string())
            }
        }

        let can_execute = Arc::new(AtomicBool::new(true));
        let can_execute_clone = Arc::clone(&can_execute);

        let op = Operation {
            precondition: BoxTester::new(move || can_execute_clone.load(Ordering::Acquire)),
        };

        assert!(op.execute().is_ok());

        can_execute.store(false, Ordering::Release);
        assert!(op.execute().is_err());
    }

    #[test]
    fn test_complex_logical_conditions() {
        let db_ready = Arc::new(AtomicBool::new(true));
        let cache_ready = Arc::new(AtomicBool::new(true));
        let config_loaded = Arc::new(AtomicBool::new(false));

        let db_clone = Arc::clone(&db_ready);
        let cache_clone = Arc::clone(&cache_ready);
        let config_clone = Arc::clone(&config_loaded);

        let system_ready = BoxTester::new(move || db_clone.load(Ordering::Acquire))
            .and(move || cache_clone.load(Ordering::Acquire))
            .and(move || config_clone.load(Ordering::Acquire));

        assert!(!system_ready.test());

        config_loaded.store(true, Ordering::Release);
        assert!(system_ready.test());
    }

    #[test]
    fn test_fallback_logic() {
        let primary_available = Arc::new(AtomicBool::new(false));
        let fallback_available = Arc::new(AtomicBool::new(true));

        let primary_clone = Arc::clone(&primary_available);
        let fallback_clone = Arc::clone(&fallback_available);

        let availability = BoxTester::new(move || primary_clone.load(Ordering::Acquire))
            .or(move || fallback_clone.load(Ordering::Acquire));

        assert!(availability.test());

        fallback_available.store(false, Ordering::Release);
        assert!(!availability.test());

        primary_available.store(true, Ordering::Release);
        assert!(availability.test());
    }

    // ========================================================================
    // Boundary conditions and special case tests
    // ========================================================================

    #[test]
    fn test_always_true() {
        let tester = BoxTester::new(|| true);
        for _ in 0..100 {
            assert!(tester.test());
        }
    }

    #[test]
    fn test_always_false() {
        let tester = BoxTester::new(|| false);
        for _ in 0..100 {
            assert!(!tester.test());
        }
    }

    #[test]
    fn test_multiple_not() {
        let tester = !(!BoxTester::new(|| true));
        assert!(tester.test());
    }

    #[test]
    fn test_empty_and_chain() {
        let tester = BoxTester::new(|| true);
        assert!(tester.test());
    }

    #[test]
    fn test_deeply_nested_composition() {
        let tester = BoxTester::new(|| true)
            .and(|| true)
            .and(|| true)
            .and(|| true)
            .and(|| true);

        assert!(tester.test());
    }

    // ========================================================================
    // BoxTester nand/xor/nor tests
    // ========================================================================

    #[test]
    fn test_box_tester_nand_true_true() {
        let combined = BoxTester::new(|| true).nand(BoxTester::new(|| true));
        assert!(!combined.test()); // NAND: !(true && true) = false
    }

    #[test]
    fn test_box_tester_nand_true_false() {
        let combined = BoxTester::new(|| true).nand(BoxTester::new(|| false));
        assert!(combined.test()); // NAND: !(true && false) = true
    }

    #[test]
    fn test_box_tester_nand_false_true() {
        let combined = BoxTester::new(|| false).nand(BoxTester::new(|| true));
        assert!(combined.test()); // NAND: !(false && true) = true
    }

    #[test]
    fn test_box_tester_nand_false_false() {
        let combined = BoxTester::new(|| false).nand(BoxTester::new(|| false));
        assert!(combined.test()); // NAND: !(false && false) = true
    }

    #[test]
    fn test_box_tester_xor_true_true() {
        let combined = BoxTester::new(|| true).xor(BoxTester::new(|| true));
        assert!(!combined.test()); // XOR: true ^ true = false
    }

    #[test]
    fn test_box_tester_xor_true_false() {
        let combined = BoxTester::new(|| true).xor(BoxTester::new(|| false));
        assert!(combined.test()); // XOR: true ^ false = true
    }

    #[test]
    fn test_box_tester_xor_false_true() {
        let combined = BoxTester::new(|| false).xor(BoxTester::new(|| true));
        assert!(combined.test()); // XOR: false ^ true = true
    }

    #[test]
    fn test_box_tester_xor_false_false() {
        let combined = BoxTester::new(|| false).xor(BoxTester::new(|| false));
        assert!(!combined.test()); // XOR: false ^ false = false
    }

    #[test]
    fn test_box_tester_nor_true_true() {
        let combined = BoxTester::new(|| true).nor(BoxTester::new(|| true));
        assert!(!combined.test()); // NOR: !(true || true) = false
    }

    #[test]
    fn test_box_tester_nor_true_false() {
        let combined = BoxTester::new(|| true).nor(BoxTester::new(|| false));
        assert!(!combined.test()); // NOR: !(true || false) = false
    }

    #[test]
    fn test_box_tester_nor_false_true() {
        let combined = BoxTester::new(|| false).nor(BoxTester::new(|| true));
        assert!(!combined.test()); // NOR: !(false || true) = false
    }

    #[test]
    fn test_box_tester_nor_false_false() {
        let combined = BoxTester::new(|| false).nor(BoxTester::new(|| false));
        assert!(combined.test()); // NOR: !(false || false) = true
    }

    // ========================================================================
    // ArcTester nand/xor/nor tests
    // ========================================================================

    #[test]
    fn test_arc_tester_nand_true_true() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| true);
        let combined = first.nand(second.clone());
        assert!(!combined.test()); // NAND: !(true && true) = false
    }

    #[test]
    fn test_arc_tester_nand_true_false() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| false);
        let combined = first.nand(second.clone());
        assert!(combined.test()); // NAND: !(true && false) = true
    }

    #[test]
    fn test_arc_tester_nand_false_false() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| false);
        let combined = first.nand(second.clone());
        assert!(combined.test()); // NAND: !(false && false) = true
    }

    #[test]
    fn test_arc_tester_xor_true_true() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| true);
        let combined = first.xor(second.clone());
        assert!(!combined.test()); // XOR: true ^ true = false
    }

    #[test]
    fn test_arc_tester_xor_true_false() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| false);
        let combined = first.xor(second.clone());
        assert!(combined.test()); // XOR: true ^ false = true
    }

    #[test]
    fn test_arc_tester_xor_false_false() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| false);
        let combined = first.xor(second.clone());
        assert!(!combined.test()); // XOR: false ^ false = false
    }

    #[test]
    fn test_arc_tester_nor_true_true() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| true);
        let combined = first.nor(second.clone());
        assert!(!combined.test()); // NOR: !(true || true) = false
    }

    #[test]
    fn test_arc_tester_nor_false_true() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| true);
        let combined = first.nor(second.clone());
        assert!(!combined.test()); // NOR: !(false || true) = false
    }

    #[test]
    fn test_arc_tester_nor_false_false() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| false);
        let combined = first.nor(second.clone());
        assert!(combined.test()); // NOR: !(false || false) = true
    }

    // ========================================================================
    // RcTester nand/xor/nor tests
    // ========================================================================

    #[test]
    fn test_rc_tester_nand_true_true() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| true);
        let combined = first.nand(second.clone());
        assert!(!combined.test()); // NAND: !(true && true) = false
    }

    #[test]
    fn test_rc_tester_nand_true_false() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| false);
        let combined = first.nand(second.clone());
        assert!(combined.test()); // NAND: !(true && false) = true
    }

    #[test]
    fn test_rc_tester_nand_false_false() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| false);
        let combined = first.nand(second.clone());
        assert!(combined.test()); // NAND: !(false && false) = true
    }

    #[test]
    fn test_rc_tester_xor_true_true() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| true);
        let combined = first.xor(second.clone());
        assert!(!combined.test()); // XOR: true ^ true = false
    }

    #[test]
    fn test_rc_tester_xor_true_false() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| false);
        let combined = first.xor(second.clone());
        assert!(combined.test()); // XOR: true ^ false = true
    }

    #[test]
    fn test_rc_tester_xor_false_false() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| false);
        let combined = first.xor(second.clone());
        assert!(!combined.test()); // XOR: false ^ false = false
    }

    #[test]
    fn test_rc_tester_nor_true_true() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| true);
        let combined = first.nor(second.clone());
        assert!(!combined.test()); // NOR: !(true || true) = false
    }

    #[test]
    fn test_rc_tester_nor_false_true() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| true);
        let combined = first.nor(second.clone());
        assert!(!combined.test()); // NOR: !(false || true) = false
    }

    #[test]
    fn test_rc_tester_nor_false_false() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| false);
        let combined = first.nor(second.clone());
        assert!(combined.test()); // NOR: !(false || false) = true
    }

    // ========================================================================
    // Tester trait default implementation tests (for closures)
    // ========================================================================

    // ========================================================================
    // into_box tests for BoxTester
    // ========================================================================

    // ========================================================================
    // Panic tests for invalid conversions
    // ========================================================================
    // Note: BoxTester::into_arc() and RcTester::into_arc() cannot be tested
    // because they require Send + Sync bounds which BoxTester and RcTester
    // don't satisfy at compile time. The panic code is unreachable in practice.

    // ========================================================================
    // Custom Tester implementation tests
    // ========================================================================
}
