// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for the predicate module.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::predicates::ArcPredicate;
use qubit_function::predicates::BoxPredicate;
use qubit_function::predicates::Predicate;
use qubit_function::predicates::RcPredicate;

struct PositivePredicate;

impl Predicate<i32> for PositivePredicate {
    fn test(&self, value: &i32) -> bool {
        *value > 0
    }
}

#[cfg(test)]
mod interior_mutability_tests {
    use super::Arc;
    use super::ArcPredicate;
    use super::BoxPredicate;
    use super::Mutex;
    use super::Predicate;
    use super::RcPredicate;
    use super::RefCell;

    #[test]
    fn test_box_predicate_with_refcell_counter() {
        let count = RefCell::new(0);
        let pred = BoxPredicate::new(move |x: &i32| {
            *count.borrow_mut() += 1;
            *x > 0
        });

        assert!(pred.test(&5));
        assert!(pred.test(&10));
        assert!(!pred.test(&-3));
    }

    #[test]
    fn test_arc_predicate_with_mutex_counter() {
        let count = Arc::new(Mutex::new(0));
        let count_clone = Arc::clone(&count);

        let pred = ArcPredicate::new(move |x: &i32| {
            let mut c = count_clone.lock().expect("mutex should not be poisoned");
            *c += 1;
            *x > 0
        });

        assert!(pred.test(&5));
        assert!(pred.test(&10));
        assert!(!pred.test(&-3));

        assert_eq!(*count.lock().expect("mutex should not be poisoned"), 3);
    }

    #[test]
    fn test_rc_predicate_with_refcell_cache() {
        use std::collections::HashMap;

        let cache = RefCell::new(HashMap::new());
        let pred = RcPredicate::new(move |x: &i32| {
            let mut c = cache.borrow_mut();
            *c.entry(*x).or_insert_with(|| *x > 0 && x % 2 == 0)
        });

        // First call computes and caches
        assert!(pred.test(&4));
        // Second call uses cache
        assert!(pred.test(&4));
        assert!(!pred.test(&3));
    }

    #[test]
    fn test_arc_predicate_thread_safe_counter() {
        let count = Arc::new(Mutex::new(0));
        let pred = ArcPredicate::new({
            let count = Arc::clone(&count);
            move |x: &i32| {
                let mut c = count.lock().expect("mutex should not be poisoned");
                *c += 1;
                *x > 0
            }
        });

        let pred_clone = pred.clone();
        let count_clone = Arc::clone(&count);

        let handle = std::thread::spawn(move || {
            assert!(pred_clone.test(&5));
            assert!(pred_clone.test(&10));
        });

        assert!(pred.test(&3));
        handle.join().expect("thread should not panic");

        assert_eq!(*count_clone.lock().expect("mutex should not be poisoned"), 3);
    }
}
