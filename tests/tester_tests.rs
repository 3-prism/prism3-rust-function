/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Unit tests for Tester

#[cfg(test)]
mod tests {
    use prism3_function::{ArcTester, BoxTester, FnTesterOps, RcTester, Tester};
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };

    // ========================================================================
    // BoxTester tests
    // ========================================================================

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

        // Verify that the second test was not executed (short-circuit evaluation)
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
        let negated = BoxTester::new(|| true).not();
        assert!(!negated.test());
    }

    #[test]
    fn test_box_tester_not_false() {
        let negated = BoxTester::new(|| false).not();
        assert!(negated.test());
    }

    #[test]
    fn test_box_tester_complex_composition() {
        let combined = BoxTester::new(|| true).and(|| true).or(|| false).not();
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

    #[test]
    fn test_box_tester_into_box() {
        let closure = || true;
        let boxed = closure.into_box();
        let tester: BoxTester = boxed;
        assert!(tester.test());
    }

    #[test]
    fn test_box_tester_into_rc() {
        let tester = BoxTester::new(|| true);
        let rc = tester.into_rc();
        let rc_copy = rc;
        assert!(rc_copy.test());
    }

    #[test]
    fn test_box_tester_into_fn() {
        let tester = BoxTester::new(|| true);
        let func = tester.into_fn();
        assert!(func());
    }

    #[test]
    fn test_box_tester_into_fn_with_state() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let tester = BoxTester::new(move || count_clone.load(Ordering::Relaxed) <= 2);
        let func = tester.into_fn();

        assert!(func()); // 0
        count.fetch_add(1, Ordering::Relaxed);
        assert!(func()); // 1
        count.fetch_add(1, Ordering::Relaxed);
        assert!(func()); // 2
        count.fetch_add(1, Ordering::Relaxed);
        assert!(!func()); // 3
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
        let combined = first.and(&second);
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(first.test());
    }

    #[test]
    fn test_arc_tester_or() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| true);
        let combined = first.or(&second);
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(second.test());
    }

    #[test]
    fn test_arc_tester_not() {
        let original = ArcTester::new(|| true);
        let negated = original.not();
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
        assert!(handle.join().unwrap());

        // Both threads successfully tested
        counter.fetch_add(1, Ordering::Relaxed);
        assert!(tester.test());
    }

    #[test]
    fn test_arc_tester_into_box() {
        let tester = ArcTester::new(|| true);
        let boxed = tester.into_box();
        assert!(boxed.test());
    }

    #[test]
    fn test_arc_tester_into_rc() {
        let tester = ArcTester::new(|| true);
        let rc = tester.into_rc();
        assert!(rc.test());
    }

    #[test]
    fn test_arc_tester_into_arc() {
        let tester = ArcTester::new(|| true);
        let arc = tester.into_arc();
        assert!(arc.test());
    }

    #[test]
    fn test_arc_tester_into_fn() {
        let tester = ArcTester::new(|| true);
        let func = tester.into_fn();
        assert!(func());
    }

    #[test]
    fn test_arc_tester_into_fn_with_state() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let tester = ArcTester::new(move || count_clone.load(Ordering::Relaxed) <= 2);
        let func = tester.into_fn();

        assert!(func()); // 0
        count.fetch_add(1, Ordering::Relaxed);
        assert!(func()); // 1
        count.fetch_add(1, Ordering::Relaxed);
        assert!(func()); // 2
        count.fetch_add(1, Ordering::Relaxed);
        assert!(!func()); // 3
    }

    #[test]
    fn test_arc_tester_to_box() {
        let tester = ArcTester::new(|| true);
        let boxed = tester.to_box();
        assert!(boxed.test());
        // Original tester is still available
        assert!(tester.test());
    }

    #[test]
    fn test_arc_tester_to_rc() {
        let tester = ArcTester::new(|| false);
        let rc = tester.to_rc();
        assert!(!rc.test());
        // Original tester is still available
        assert!(!tester.test());
    }

    #[test]
    fn test_arc_tester_to_arc() {
        let tester = ArcTester::new(|| true);
        let arc = tester.to_arc();
        assert!(arc.test());
        // Original tester is still available
        assert!(tester.test());
    }

    #[test]
    fn test_arc_tester_to_fn() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let tester = ArcTester::new(move || count_clone.load(Ordering::Relaxed) <= 1);
        let func = tester.to_fn();

        assert!(func());
        assert!(tester.test());

        count.fetch_add(1, Ordering::Relaxed);
        assert!(func());
        assert!(tester.test());

        count.fetch_add(1, Ordering::Relaxed);
        assert!(!func());
        assert!(!tester.test());
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
        let combined = first.and(&second);
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(first.test());
    }

    #[test]
    fn test_rc_tester_or() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| true);
        let combined = first.or(&second);
        assert!(combined.test());

        // Verify that the original tester is still available
        assert!(second.test());
    }

    #[test]
    fn test_rc_tester_not() {
        let original = RcTester::new(|| true);
        let negated = original.not();
        assert!(!negated.test());

        // Verify that the original tester is still available
        assert!(original.test());
    }

    #[test]
    fn test_rc_tester_into_box() {
        let tester = RcTester::new(|| true);
        let boxed = tester.into_box();
        assert!(boxed.test());
    }

    #[test]
    fn test_rc_tester_into_rc() {
        let tester = RcTester::new(|| true);
        let rc = tester.into_rc();
        assert!(rc.test());
    }

    #[test]
    fn test_rc_tester_into_fn() {
        let tester = RcTester::new(|| true);
        let func = tester.into_fn();
        assert!(func());
    }

    #[test]
    fn test_rc_tester_into_fn_with_state() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let count = Rc::new(RefCell::new(0));
        let count_clone = Rc::clone(&count);

        let tester = RcTester::new(move || *count_clone.borrow() <= 2);
        let func = tester.into_fn();

        assert!(func()); // 0
        *count.borrow_mut() += 1;
        assert!(func()); // 1
        *count.borrow_mut() += 1;
        assert!(func()); // 2
        *count.borrow_mut() += 1;
        assert!(!func()); // 3
    }

    #[test]
    fn test_rc_tester_to_box() {
        let tester = RcTester::new(|| true);
        let boxed = tester.to_box();
        assert!(boxed.test());
        // Original tester is still available
        assert!(tester.test());
    }

    #[test]
    fn test_rc_tester_to_rc() {
        let tester = RcTester::new(|| false);
        let rc = tester.to_rc();
        assert!(!rc.test());
        // Original tester is still available
        assert!(!tester.test());
    }

    #[test]
    fn test_rc_tester_to_fn() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let count = Rc::new(RefCell::new(0));
        let count_clone = Rc::clone(&count);

        let tester = RcTester::new(move || *count_clone.borrow() <= 1);
        let func = tester.to_fn();

        assert!(func());
        assert!(tester.test());

        *count.borrow_mut() += 1;
        assert!(func());
        assert!(tester.test());

        *count.borrow_mut() += 1;
        assert!(!func());
        assert!(!tester.test());
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

    #[test]
    fn test_closure_into_fn() {
        let closure = || true;
        let func = closure.into_fn();
        assert!(func());
    }

    #[test]
    fn test_closure_into_fn_with_state() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let closure = move || count_clone.load(Ordering::Relaxed) <= 2;
        let func = closure.into_fn();

        assert!(func()); // 0
        count.fetch_add(1, Ordering::Relaxed);
        assert!(func()); // 1
        count.fetch_add(1, Ordering::Relaxed);
        assert!(func()); // 2
        count.fetch_add(1, Ordering::Relaxed);
        assert!(!func()); // 3
    }

    // ========================================================================
    // FnTesterOps tests
    // ========================================================================

    #[test]
    fn test_fn_ops_into_box_tester() {
        let tester = (|| true).into_box();
        assert!(tester.test());
    }

    #[test]
    fn test_fn_ops_into_rc_tester() {
        let tester = (|| true).into_rc();
        assert!(tester.test());
    }

    #[test]
    fn test_fn_ops_into_arc_tester() {
        let tester = (|| true).into_arc();
        assert!(tester.test());
    }

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

        let rate_limiter =
            BoxTester::new(move || attempts_clone.load(Ordering::Relaxed) <= max_attempts);

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
        let tester = BoxTester::new(|| true).not().not();
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
        let combined = first.nand(&second);
        assert!(!combined.test()); // NAND: !(true && true) = false
    }

    #[test]
    fn test_arc_tester_nand_true_false() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| false);
        let combined = first.nand(&second);
        assert!(combined.test()); // NAND: !(true && false) = true
    }

    #[test]
    fn test_arc_tester_nand_false_false() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| false);
        let combined = first.nand(&second);
        assert!(combined.test()); // NAND: !(false && false) = true
    }

    #[test]
    fn test_arc_tester_xor_true_true() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| true);
        let combined = first.xor(&second);
        assert!(!combined.test()); // XOR: true ^ true = false
    }

    #[test]
    fn test_arc_tester_xor_true_false() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| false);
        let combined = first.xor(&second);
        assert!(combined.test()); // XOR: true ^ false = true
    }

    #[test]
    fn test_arc_tester_xor_false_false() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| false);
        let combined = first.xor(&second);
        assert!(!combined.test()); // XOR: false ^ false = false
    }

    #[test]
    fn test_arc_tester_nor_true_true() {
        let first = ArcTester::new(|| true);
        let second = ArcTester::new(|| true);
        let combined = first.nor(&second);
        assert!(!combined.test()); // NOR: !(true || true) = false
    }

    #[test]
    fn test_arc_tester_nor_false_true() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| true);
        let combined = first.nor(&second);
        assert!(!combined.test()); // NOR: !(false || true) = false
    }

    #[test]
    fn test_arc_tester_nor_false_false() {
        let first = ArcTester::new(|| false);
        let second = ArcTester::new(|| false);
        let combined = first.nor(&second);
        assert!(combined.test()); // NOR: !(false || false) = true
    }

    // ========================================================================
    // RcTester nand/xor/nor tests
    // ========================================================================

    #[test]
    fn test_rc_tester_nand_true_true() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| true);
        let combined = first.nand(&second);
        assert!(!combined.test()); // NAND: !(true && true) = false
    }

    #[test]
    fn test_rc_tester_nand_true_false() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| false);
        let combined = first.nand(&second);
        assert!(combined.test()); // NAND: !(true && false) = true
    }

    #[test]
    fn test_rc_tester_nand_false_false() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| false);
        let combined = first.nand(&second);
        assert!(combined.test()); // NAND: !(false && false) = true
    }

    #[test]
    fn test_rc_tester_xor_true_true() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| true);
        let combined = first.xor(&second);
        assert!(!combined.test()); // XOR: true ^ true = false
    }

    #[test]
    fn test_rc_tester_xor_true_false() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| false);
        let combined = first.xor(&second);
        assert!(combined.test()); // XOR: true ^ false = true
    }

    #[test]
    fn test_rc_tester_xor_false_false() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| false);
        let combined = first.xor(&second);
        assert!(!combined.test()); // XOR: false ^ false = false
    }

    #[test]
    fn test_rc_tester_nor_true_true() {
        let first = RcTester::new(|| true);
        let second = RcTester::new(|| true);
        let combined = first.nor(&second);
        assert!(!combined.test()); // NOR: !(true || true) = false
    }

    #[test]
    fn test_rc_tester_nor_false_true() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| true);
        let combined = first.nor(&second);
        assert!(!combined.test()); // NOR: !(false || true) = false
    }

    #[test]
    fn test_rc_tester_nor_false_false() {
        let first = RcTester::new(|| false);
        let second = RcTester::new(|| false);
        let combined = first.nor(&second);
        assert!(combined.test()); // NOR: !(false || false) = true
    }

    // ========================================================================
    // Tester trait default implementation tests (for closures)
    // ========================================================================

    #[test]
    fn test_closure_and_operation() {
        let first = || true;
        let second = || true;
        let combined = first.and(second);
        assert!(combined.test());
    }

    #[test]
    fn test_closure_or_operation() {
        let first = || false;
        let second = || true;
        let combined = first.or(second);
        assert!(combined.test());
    }

    #[test]
    fn test_closure_not_operation() {
        let closure = || true;
        let negated = closure.not();
        assert!(!negated.test());
    }

    #[test]
    fn test_closure_nand_operation() {
        let first = || true;
        let second = || true;
        let combined = first.nand(second);
        assert!(!combined.test()); // NAND: !(true && true) = false
    }

    #[test]
    fn test_closure_xor_operation() {
        let first = || true;
        let second = || false;
        let combined = first.xor(second);
        assert!(combined.test()); // XOR: true ^ false = true
    }

    #[test]
    fn test_closure_nor_operation() {
        let first = || false;
        let second = || false;
        let combined = first.nor(second);
        assert!(combined.test()); // NOR: !(false || false) = true
    }

    // ========================================================================
    // into_box tests for BoxTester
    // ========================================================================

    #[test]
    fn test_box_tester_into_box_identity() {
        let tester = BoxTester::new(|| true);
        let boxed = tester.into_box();
        assert!(boxed.test());
    }

    // ========================================================================
    // Panic tests for invalid conversions
    // ========================================================================
    // Note: BoxTester::into_arc() and RcTester::into_arc() cannot be tested
    // because they require Send + Sync bounds which BoxTester and RcTester
    // don't satisfy at compile time. The panic code is unreachable in practice.

    // ========================================================================
    // Custom Tester implementation tests
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

    #[test]
    fn test_custom_tester_into_box_uses_default_impl() {
        // Test that custom tester can be converted to BoxTester using
        // the default implementation provided by the Tester trait
        let custom = AlwaysTrueTester;
        let boxed = custom.into_box();
        assert!(boxed.test());
    }

    #[test]
    fn test_custom_tester_into_rc_uses_default_impl() {
        // Test that custom tester can be converted to RcTester using
        // the default implementation provided by the Tester trait
        let custom = AlwaysTrueTester;
        let rc = custom.into_rc();
        assert!(rc.test());

        // Verify that RcTester can be cloned
        let rc_clone = rc.clone();
        assert!(rc_clone.test());
    }

    #[test]
    fn test_custom_tester_into_arc_uses_default_impl() {
        // Test that custom tester can be converted to ArcTester using
        // the default implementation provided by the Tester trait
        let custom = ThreadSafeTester::new(true);
        let arc = custom.into_arc();
        assert!(arc.test());

        // Verify that ArcTester can be cloned and sent across threads
        let arc_clone = arc.clone();
        let handle = std::thread::spawn(move || arc_clone.test());
        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_custom_tester_chaining_conversions() {
        // Test that custom tester can be converted through different
        // wrapper types using the default implementations
        let custom1 = AlwaysTrueTester;
        let boxed = custom1.into_box();
        let rc = boxed.into_rc();
        assert!(rc.test());

        let custom2 = ThreadSafeTester::new(true);
        let arc1 = custom2.into_arc();
        let boxed2 = arc1.into_box();
        assert!(boxed2.test());
    }

    #[test]
    fn test_custom_tester_with_state() {
        // Test custom tester with internal state to verify that state
        // is properly captured in the default implementations
        struct CounterTester {
            counter: Arc<AtomicUsize>,
            threshold: usize,
        }

        impl Tester for CounterTester {
            fn test(&self) -> bool {
                self.counter.load(Ordering::Relaxed) < self.threshold
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let tester = CounterTester {
            counter: Arc::clone(&counter),
            threshold: 3,
        };

        // Convert to BoxTester using default implementation
        let boxed = tester.into_box();
        assert!(boxed.test());

        counter.fetch_add(1, Ordering::Relaxed);
        assert!(boxed.test());

        counter.fetch_add(1, Ordering::Relaxed);
        assert!(boxed.test());

        counter.fetch_add(1, Ordering::Relaxed);
        assert!(!boxed.test());
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

    #[test]
    fn test_custom_tester_into_box() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default into_box implementation
        let boxed = tester.into_box();
        assert!(boxed.test());

        value.store(false, Ordering::Relaxed);
        assert!(!boxed.test());
    }

    #[test]
    fn test_custom_tester_into_rc() {
        let value = Arc::new(AtomicBool::new(false));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default into_rc implementation
        let rc = tester.into_rc();
        assert!(!rc.test());

        value.store(true, Ordering::Relaxed);
        assert!(rc.test());
    }

    #[test]
    fn test_custom_tester_into_arc() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default into_arc implementation
        let arc = tester.into_arc();
        assert!(arc.test());

        // Can clone and share across threads
        let arc_clone = arc.clone();
        let handle = std::thread::spawn(move || arc_clone.test());

        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_custom_tester_into_fn() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default into_fn implementation
        let func = tester.into_fn();
        assert!(func());

        value.store(false, Ordering::Relaxed);
        assert!(!func());
    }

    #[test]
    fn test_custom_tester_to_box() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default to_box implementation (requires Clone)
        let boxed = tester.to_box();
        assert!(boxed.test());

        // Original tester is still available
        assert!(tester.test());

        value.store(false, Ordering::Relaxed);
        assert!(!boxed.test());
        assert!(!tester.test());
    }

    #[test]
    fn test_custom_tester_to_rc() {
        let value = Arc::new(AtomicBool::new(false));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default to_rc implementation (requires Clone)
        let rc = tester.to_rc();
        assert!(!rc.test());

        // Original tester is still available
        assert!(!tester.test());

        value.store(true, Ordering::Relaxed);
        assert!(rc.test());
        assert!(tester.test());
    }

    #[test]
    fn test_custom_tester_to_arc() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default to_arc implementation (requires Clone + Send + Sync)
        let arc = tester.to_arc();
        assert!(arc.test());

        // Original tester is still available
        assert!(tester.test());

        // Can clone and share across threads
        let arc_clone = arc.clone();
        let tester_clone = tester.clone();
        let handle = std::thread::spawn(move || arc_clone.test() && tester_clone.test());

        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_custom_tester_to_fn() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Use default to_fn implementation (requires Clone)
        let func = tester.to_fn();
        assert!(func());

        // Original tester is still available
        assert!(tester.test());

        value.store(false, Ordering::Relaxed);
        assert!(!func());
        assert!(!tester.test());
    }

    #[test]
    fn test_custom_tester_conversions_chain() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // Chain multiple conversions
        let arc = tester.clone().into_arc();
        let boxed = tester.clone().into_box();
        let rc = tester.clone().into_rc();
        let func = tester.clone().into_fn();

        assert!(arc.test());
        assert!(boxed.test());
        assert!(rc.test());
        assert!(func());
        assert!(tester.test());

        value.store(false, Ordering::Relaxed);

        assert!(!arc.test());
        assert!(!boxed.test());
        assert!(!rc.test());
        assert!(!func());
        assert!(!tester.test());
    }

    #[test]
    fn test_custom_tester_to_methods_preserve_original() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = CustomTester {
            value: Arc::clone(&value),
        };

        // All to_xxx methods should preserve the original
        let boxed = tester.to_box();
        let rc = tester.to_rc();
        let arc = tester.to_arc();
        let func = tester.to_fn();

        // All should be true initially
        assert!(tester.test());
        assert!(boxed.test());
        assert!(rc.test());
        assert!(arc.test());
        assert!(func());

        // Change state
        value.store(false, Ordering::Relaxed);

        // All should reflect the change
        assert!(!tester.test());
        assert!(!boxed.test());
        assert!(!rc.test());
        assert!(!arc.test());
        assert!(!func());
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
    fn test_non_clone_tester_into_methods() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = NonCloneTester {
            value: Arc::clone(&value),
        };

        // into_box should work without Clone
        let boxed = tester.into_box();
        assert!(boxed.test());

        value.store(false, Ordering::Relaxed);
        assert!(!boxed.test());
    }

    #[test]
    fn test_non_clone_tester_into_rc() {
        let value = Arc::new(AtomicBool::new(false));
        let tester = NonCloneTester {
            value: Arc::clone(&value),
        };

        // into_rc should work without Clone
        let rc = tester.into_rc();
        assert!(!rc.test());

        value.store(true, Ordering::Relaxed);
        assert!(rc.test());
    }

    #[test]
    fn test_non_clone_tester_into_fn() {
        let value = Arc::new(AtomicBool::new(true));
        let tester = NonCloneTester {
            value: Arc::clone(&value),
        };

        // into_fn should work without Clone
        let func = tester.into_fn();
        assert!(func());

        value.store(false, Ordering::Relaxed);
        assert!(!func());
    }

    // Note: The following tests would fail to compile if uncommented,
    // demonstrating that to_xxx methods correctly require Clone:
    //
    // #[test]
    // fn test_non_clone_tester_to_box_compile_error() {
    //     let value = Arc::new(AtomicBool::new(true));
    //     let tester = NonCloneTester { value };
    //     let _boxed = tester.to_box(); // Compile error: NonCloneTester
    //                                    // doesn't implement Clone
    // }
}
