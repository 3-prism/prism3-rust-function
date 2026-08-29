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
    fn test_box_tester_new_basic() {
        let tester = BoxTester::new(|| true);
        assert!(tester.test());
    }

    #[test]
    fn test_box_tester_stateful() {
        // State is managed externally
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let tester = BoxTester::new(move || count_clone.load(Ordering::Relaxed) <= 3);

        assert!(tester.test()); // 0
        count.fetch_add(1, Ordering::Relaxed);
        assert!(tester.test()); // 1
        count.fetch_add(1, Ordering::Relaxed);
        assert!(tester.test()); // 2
        count.fetch_add(1, Ordering::Relaxed);
        assert!(tester.test()); // 3
        count.fetch_add(1, Ordering::Relaxed);
        assert!(!tester.test()); // 4
    }

    #[test]
    fn test_box_tester_and_true_true() {
        let combined = BoxTester::new(|| true).and(|| true);
        assert!(combined.test());
    }

    #[test]
    fn test_box_tester_and_true_false() {
        let combined = BoxTester::new(|| true).and(|| false);
        assert!(!combined.test());
    }

    #[test]
    fn test_box_tester_and_false_true() {
        let combined = BoxTester::new(|| false).and(|| true);
        assert!(!combined.test());
    }

    #[test]
    fn test_box_tester_and_short_circuit() {
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = Arc::clone(&executed);

        let combined = BoxTester::new(|| false).and(move || {
            executed_clone.store(true, Ordering::Relaxed);
            true
        });
        assert!(!combined.test());

        // Verify that the second test was not executed (short-circuit
        // evaluation)
        assert!(!executed.load(Ordering::Relaxed));
    }

    #[test]
    fn test_box_tester_or_true_true() {
        let combined = BoxTester::new(|| true).or(|| true);
        assert!(combined.test());
    }

    #[test]
    fn test_box_tester_or_true_false() {
        let combined = BoxTester::new(|| true).or(|| false);
        assert!(combined.test());
    }

    #[test]
    fn test_box_tester_or_false_true() {
        let combined = BoxTester::new(|| false).or(|| true);
        assert!(combined.test());
    }

    #[test]
    fn test_box_tester_or_false_false() {
        let combined = BoxTester::new(|| false).or(|| false);
        assert!(!combined.test());
    }

    #[test]
    fn test_box_tester_not_true() {
        let negated = !BoxTester::new(|| true);
        assert!(!negated.test());
    }

    #[test]
    fn test_box_tester_not_false() {
        let negated = !BoxTester::new(|| false);
        assert!(negated.test());
    }

    #[test]
    fn test_box_tester_complex_composition() {
        let combined = !(BoxTester::new(|| true).and(|| true).or(|| false));
        assert!(!combined.test());
    }

    #[test]
    fn test_box_tester_chain_with_state() {
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));
        let count1_clone = Arc::clone(&count1);
        let count2_clone = Arc::clone(&count2);

        let combined = BoxTester::new(move || count1_clone.load(Ordering::Relaxed) <= 2)
            .and(move || count2_clone.load(Ordering::Relaxed) <= 1);

        assert!(combined.test()); // count1=0, count2=0
        count1.fetch_add(1, Ordering::Relaxed);
        assert!(combined.test()); // count1=1, count2=0
        count2.fetch_add(1, Ordering::Relaxed);
        assert!(combined.test()); // count1=1, count2=1
        count2.fetch_add(1, Ordering::Relaxed);
        assert!(!combined.test()); // count1=1, count2=2 (second condition fails)
    }

    // ========================================================================
    // ArcTester tests
    // ========================================================================

    #[test]
    fn test_arc_tester_new_basic() {
        let tester = ArcTester::new(|| true);
        assert!(tester.test());
    }

    #[test]
    fn test_arc_tester_clone() {
        let tester = ArcTester::new(|| true);
        let clone = tester.clone();
        assert!(clone.test());
    }

    #[test]
    fn test_arc_tester_stateful_shared() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let tester = ArcTester::new(move || counter_clone.load(Ordering::Relaxed) <= 3);

        let t1 = tester.clone();
        let t2 = tester.clone();

        assert!(t1.test()); // 0
        counter.fetch_add(1, Ordering::Relaxed);
        assert!(t2.test()); // 1
        counter.fetch_add(1, Ordering::Relaxed);
        assert!(t1.test()); // 2
        counter.fetch_add(1, Ordering::Relaxed);
        assert!(t2.test()); // 3
        counter.fetch_add(1, Ordering::Relaxed);
        assert!(!t1.test()); // 4
    }

    #[test]
    fn test_arc_tester_and() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| true);
        let combined = first.and(second.clone());
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(first.test());
    }

    #[test]
    fn test_arc_tester_or() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| true);
        let combined = first.or(second.clone());
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(second.test());
    }

    #[test]
    fn test_arc_tester_not() {
        let original = ArcTester::new(|| true);
        let negated = !&original;
        assert!(!negated.test());

        // Verify that the original tester is still available
        assert!(original.test());
    }

    #[test]
    fn test_arc_tester_multi_threaded() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let tester = ArcTester::new(move || counter_clone.load(Ordering::Relaxed) < 10);

        let clone = tester.clone();
        let handle = std::thread::spawn(move || clone.test());

        assert!(tester.test());
        assert!(handle.join().expect("thread should not panic"));

        // Both threads successfully tested
        counter.fetch_add(1, Ordering::Relaxed);
        assert!(tester.test());
    }

    // ========================================================================
    // RcTester tests
    // ========================================================================

    #[test]
    fn test_rc_tester_new_basic() {
        let tester = RcTester::new(|| true);
        assert!(tester.test());
    }

    #[test]
    fn test_rc_tester_clone() {
        let tester = RcTester::new(|| true);
        let clone = tester.clone();
        assert!(clone.test());
    }

    #[test]
    fn test_rc_tester_stateful_shared() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);

        let tester = RcTester::new(move || *counter_clone.borrow() <= 3);

        let t1 = tester.clone();
        let t2 = tester.clone();

        assert!(t1.test()); // 0
        *counter.borrow_mut() += 1;
        assert!(t2.test()); // 1
        *counter.borrow_mut() += 1;
        assert!(t1.test()); // 2
        *counter.borrow_mut() += 1;
        assert!(t2.test()); // 3
        *counter.borrow_mut() += 1;
        assert!(!t1.test()); // 4
    }

    #[test]
    fn test_rc_tester_and() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| true);
        let combined = first.and(second.clone());
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(first.test());
    }

    #[test]
    fn test_rc_tester_or() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| true);
        let combined = first.or(second.clone());
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(second.test());
    }

    #[test]
    fn test_rc_tester_not() {
        let original = RcTester::new(|| true);
        let negated = !&original;
        assert!(!negated.test());

        // Verify that the original tester is still available
        assert!(original.test());
    }

    // ========================================================================
    // Tester Trait tests (closures)
    // ========================================================================

    #[test]
    fn test_closure_as_tester() {
        let closure = || true;
        assert!(closure.test());
    }

    #[test]
    fn test_closure_with_state() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let closure = move || count_clone.load(Ordering::Relaxed) <= 2;

        assert!(closure.test());
        count.fetch_add(1, Ordering::Relaxed);
        assert!(closure.test());
        count.fetch_add(1, Ordering::Relaxed);
        assert!(closure.test());
        count.fetch_add(1, Ordering::Relaxed);
        assert!(!closure.test());
    }

    // ========================================================================
    // Concrete wrapper composition tests
    // ========================================================================

    // ========================================================================
    // Real-world application scenario tests
    // ========================================================================

    #[test]
    fn test_health_check_scenario() {
        let healthy = Arc::new(AtomicBool::new(true));
        let healthy_clone = Arc::clone(&healthy);
        let check_count = Arc::new(AtomicUsize::new(0));
        let check_count_clone = Arc::clone(&check_count);

        let health_check = BoxTester::new(move || {
            check_count_clone.fetch_add(1, Ordering::Relaxed);
            healthy_clone.load(Ordering::Acquire)
        });

        assert!(health_check.test());
        assert!(health_check.test());
        assert_eq!(check_count.load(Ordering::Relaxed), 2);

        healthy.store(false, Ordering::Release);
        assert!(!health_check.test());
    }

    #[test]
    fn test_rate_limiter_scenario() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = Arc::clone(&attempts);
        let max_attempts = 3;

        let rate_limiter = BoxTester::new(move || attempts_clone.load(Ordering::Relaxed) <= max_attempts);

        assert!(rate_limiter.test());
        attempts.fetch_add(1, Ordering::Relaxed);
        assert!(rate_limiter.test());
        attempts.fetch_add(1, Ordering::Relaxed);
        assert!(rate_limiter.test());
        attempts.fetch_add(1, Ordering::Relaxed);
        assert!(rate_limiter.test());
        attempts.fetch_add(1, Ordering::Relaxed);
        assert!(!rate_limiter.test());
    }

    #[test]
    fn test_condition_waiting_scenario() {
        let ready_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&ready_count);

        let readiness = BoxTester::new(move || count_clone.load(Ordering::Relaxed) >= 3);

        // Simulate waiting until condition is met
        assert!(!readiness.test());
        ready_count.fetch_add(1, Ordering::Relaxed);
        assert!(!readiness.test());
        ready_count.fetch_add(1, Ordering::Relaxed);
        assert!(!readiness.test());
        ready_count.fetch_add(1, Ordering::Relaxed);
        assert!(readiness.test());
    }
}
