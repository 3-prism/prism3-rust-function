/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for the predicate module.

use prism3_function::predicate::{
    ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate,
};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod closure_predicate_tests {
    use super::*;

    #[test]
    fn test_closure_implements_predicate() {
        let is_positive = |x: &i32| *x > 0;
        assert!(is_positive.test(&5));
        assert!(!is_positive.test(&-3));
        assert!(!is_positive.test(&0));
    }

    #[test]
    fn test_closure_and_composition() {
        let is_positive = |x: &i32| *x > 0;
        let is_even = |x: &i32| x % 2 == 0;

        let combined = is_positive.and(is_even);
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(!combined.test(&-2));
    }

    #[test]
    fn test_closure_or_composition() {
        let is_negative = |x: &i32| *x < 0;
        let is_even = |x: &i32| x % 2 == 0;

        let combined = is_negative.or(is_even);
        assert!(combined.test(&-5));
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_closure_not_composition() {
        let is_positive = |x: &i32| *x > 0;
        let is_not_positive = is_positive.not();

        assert!(!is_not_positive.test(&5));
        assert!(is_not_positive.test(&-3));
        assert!(is_not_positive.test(&0));
    }
}

#[cfg(test)]
mod box_predicate_tests {
    use super::*;

    #[test]
    fn test_new() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        assert!(pred.test(&5));
        assert!(!pred.test(&-3));
    }

    #[test]
    fn test_new_with_name() {
        let pred = BoxPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("is_positive"));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_set_name() {
        let mut pred = BoxPredicate::new(|x: &i32| *x > 0);
        assert_eq!(pred.name(), None);
        pred.set_name("is_positive");
        assert_eq!(pred.name(), Some("is_positive"));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_name_none() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        assert_eq!(pred.name(), None);
    }

    #[test]
    fn test_and_composition() {
        let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
        let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let combined = pred1.and(pred2);
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(!combined.test(&-2));
    }

    #[test]
    fn test_and_with_names() {
        let pred1 = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
        let pred2 = BoxPredicate::new_with_name("even", |x: &i32| x % 2 == 0);

        let combined = pred1.and(pred2);
        // Combined predicates do not inherit or generate names
        assert_eq!(combined.name(), None);
        assert!(combined.test(&4));
    }

    #[test]
    fn test_or_composition() {
        let pred1 = BoxPredicate::new(|x: &i32| *x < 0);
        let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let combined = pred1.or(pred2);
        assert!(combined.test(&-5));
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_or_with_names() {
        let pred1 = BoxPredicate::new_with_name("negative", |x: &i32| *x < 0);
        let pred2 = BoxPredicate::new_with_name("even", |x: &i32| x % 2 == 0);

        let combined = pred1.or(pred2);
        // Combined predicates do not inherit or generate names
        assert_eq!(combined.name(), None);
        assert!(combined.test(&-5));
    }

    #[test]
    fn test_not_composition() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let negated = pred.not();

        assert!(!negated.test(&5));
        assert!(negated.test(&-3));
        assert!(negated.test(&0));
    }

    #[test]
    fn test_not_with_name() {
        let pred = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
        let negated = pred.not();

        // Combined predicates do not inherit or generate names
        assert_eq!(negated.name(), None);
        assert!(!negated.test(&5));
    }

    #[test]
    fn test_complex_composition() {
        let positive = BoxPredicate::new(|x: &i32| *x > 0);
        let even = BoxPredicate::new(|x: &i32| x % 2 == 0);
        let less_than_ten = BoxPredicate::new(|x: &i32| *x < 10);

        let combined = positive.and(even).and(less_than_ten);
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(!combined.test(&12));
        assert!(!combined.test(&-2));
    }

    #[test]
    fn test_into_box() {
        let closure = |x: &i32| *x > 0;
        let pred: BoxPredicate<i32> = closure.into_box();
        assert!(pred.test(&5));
        assert!(!pred.test(&-3));
    }
}

#[cfg(test)]
mod rc_predicate_tests {
    use super::*;

    #[test]
    fn test_new() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        assert!(pred.test(&5));
        assert!(!pred.test(&-3));
    }

    #[test]
    fn test_new_with_name() {
        let pred = RcPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("is_positive"));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_set_name() {
        let mut pred = RcPredicate::new(|x: &i32| *x > 0);
        assert_eq!(pred.name(), None);
        pred.set_name("is_positive");
        assert_eq!(pred.name(), Some("is_positive"));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_clone() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let pred_clone = pred.clone();

        assert!(pred.test(&5));
        assert!(pred_clone.test(&5));
        assert!(!pred_clone.test(&-3));
    }

    #[test]
    fn test_and_composition() {
        let pred1 = RcPredicate::new(|x: &i32| *x > 0);
        let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

        let combined = pred1.and(pred2.clone());

        // Original predicates are still usable
        assert!(pred1.test(&5));
        assert!(pred2.test(&4));

        // Combined predicate works correctly
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_or_composition() {
        let pred1 = RcPredicate::new(|x: &i32| *x < 0);
        let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

        let combined = pred1.or(pred2.clone());

        // Original predicates are still usable
        assert!(pred1.test(&-5));
        assert!(pred2.test(&4));

        // Combined predicate works correctly
        assert!(combined.test(&-5));
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_not_composition() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let negated = pred.not();

        // Original predicate is still usable
        assert!(pred.test(&5));

        // Negated predicate works correctly
        assert!(!negated.test(&5));
        assert!(negated.test(&-3));
    }

    #[test]
    fn test_complex_reuse() {
        let positive = RcPredicate::new(|x: &i32| *x > 0);
        let even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let combined1 = positive.and(even.clone());
        let combined2 = positive.or(even.clone());

        // All predicates are still usable
        assert!(positive.test(&5));
        assert!(even.test(&4));
        assert!(combined1.test(&4));
        assert!(combined2.test(&5));
    }

    #[test]
    fn test_to_box() {
        let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
        let box_pred = rc_pred.to_box();

        assert!(rc_pred.test(&5));
        assert!(box_pred.test(&5));
        assert!(!box_pred.test(&-3));
    }

    #[test]
    fn test_into_rc() {
        let closure = |x: &i32| *x > 0;
        let pred: RcPredicate<i32> = closure.into_rc();
        assert!(pred.test(&5));
        assert!(!pred.test(&-3));
    }
}

#[cfg(test)]
mod arc_predicate_tests {
    use super::*;

    #[test]
    fn test_new() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        assert!(pred.test(&5));
        assert!(!pred.test(&-3));
    }

    #[test]
    fn test_new_with_name() {
        let pred = ArcPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("is_positive"));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_set_name() {
        let mut pred = ArcPredicate::new(|x: &i32| *x > 0);
        assert_eq!(pred.name(), None);
        pred.set_name("is_positive");
        assert_eq!(pred.name(), Some("is_positive"));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_clone() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let pred_clone = pred.clone();

        assert!(pred.test(&5));
        assert!(pred_clone.test(&5));
        assert!(!pred_clone.test(&-3));
    }

    #[test]
    fn test_send_sync() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);

        std::thread::spawn(move || {
            assert!(pred.test(&5));
            assert!(!pred.test(&-3));
        })
        .join()
        .unwrap();
    }

    #[test]
    fn test_and_composition() {
        let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
        let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let combined = pred1.and(pred2.clone());

        // Original predicates are still usable
        assert!(pred1.test(&5));
        assert!(pred2.test(&4));

        // Combined predicate works correctly
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_or_composition() {
        let pred1 = ArcPredicate::new(|x: &i32| *x < 0);
        let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let combined = pred1.or(pred2.clone());

        // Original predicates are still usable
        assert!(pred1.test(&-5));
        assert!(pred2.test(&4));

        // Combined predicate works correctly
        assert!(combined.test(&-5));
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_not_composition() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let negated = pred.not();

        // Original predicate is still usable
        assert!(pred.test(&5));

        // Negated predicate works correctly
        assert!(!negated.test(&5));
        assert!(negated.test(&-3));
    }

    #[test]
    fn test_thread_safe_composition() {
        let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
        let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let combined = pred1.and(pred2.clone());
        let combined_clone = combined.clone();

        let handle = std::thread::spawn(move || {
            assert!(combined_clone.test(&4));
            assert!(!combined_clone.test(&3));
        });

        assert!(combined.test(&4));
        handle.join().unwrap();
    }

    #[test]
    fn test_to_box() {
        let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
        let box_pred = arc_pred.to_box();

        assert!(arc_pred.test(&5));
        assert!(box_pred.test(&5));
        assert!(!box_pred.test(&-3));
    }

    #[test]
    fn test_to_rc() {
        let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
        let rc_pred = arc_pred.to_rc();

        assert!(arc_pred.test(&5));
        assert!(rc_pred.test(&5));
        assert!(!rc_pred.test(&-3));
    }

    #[test]
    fn test_into_arc() {
        let closure = |x: &i32| *x > 0;
        let pred: ArcPredicate<i32> = closure.into_arc();
        assert!(pred.test(&5));
        assert!(!pred.test(&-3));
    }
}

#[cfg(test)]
mod interior_mutability_tests {
    use super::*;

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
            let mut c = count_clone.lock().unwrap();
            *c += 1;
            *x > 0
        });

        assert!(pred.test(&5));
        assert!(pred.test(&10));
        assert!(!pred.test(&-3));

        assert_eq!(*count.lock().unwrap(), 3);
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
                let mut c = count.lock().unwrap();
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
        handle.join().unwrap();

        assert_eq!(*count_clone.lock().unwrap(), 3);
    }
}

#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_closure_to_box() {
        let closure = |x: &i32| *x > 0;
        let pred: BoxPredicate<i32> = closure.into_box();
        assert!(pred.test(&5));
    }

    #[test]
    fn test_closure_to_rc() {
        let closure = |x: &i32| *x > 0;
        let pred: RcPredicate<i32> = closure.into_rc();
        assert!(pred.test(&5));
    }

    #[test]
    fn test_closure_to_arc() {
        let closure = |x: &i32| *x > 0;
        let pred: ArcPredicate<i32> = closure.into_arc();
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_to_box() {
        let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
        let box_pred = rc_pred.to_box();
        assert!(box_pred.test(&5));
    }

    #[test]
    fn test_arc_to_box() {
        let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
        let box_pred = arc_pred.to_box();
        assert!(box_pred.test(&5));
    }

    #[test]
    fn test_arc_to_rc() {
        let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
        let rc_pred = arc_pred.to_rc();
        assert!(rc_pred.test(&5));
    }
}

#[cfg(test)]
mod different_types_tests {
    use super::*;

    #[test]
    fn test_string_predicate() {
        let pred = BoxPredicate::new(|s: &String| s.len() > 3);
        assert!(pred.test(&"hello".to_string()));
        assert!(!pred.test(&"hi".to_string()));
    }

    #[test]
    fn test_str_predicate() {
        let pred = BoxPredicate::new(|s: &&str| s.len() > 3);
        assert!(pred.test(&"hello"));
        assert!(!pred.test(&"hi"));
    }

    #[test]
    fn test_vec_predicate() {
        let pred = BoxPredicate::new(|v: &Vec<i32>| v.len() > 2);
        assert!(pred.test(&vec![1, 2, 3]));
        assert!(!pred.test(&vec![1]));
    }

    #[test]
    fn test_option_predicate() {
        let pred = BoxPredicate::new(|opt: &Option<i32>| opt.is_some());
        assert!(pred.test(&Some(5)));
        assert!(!pred.test(&None));
    }

    #[test]
    fn test_tuple_predicate() {
        let pred = BoxPredicate::new(|(a, b): &(i32, i32)| a + b > 10);
        assert!(pred.test(&(6, 5)));
        assert!(!pred.test(&(2, 3)));
    }
}

#[cfg(test)]
mod generic_function_tests {
    use super::*;

    fn filter_by_predicate<T, P>(items: Vec<T>, pred: P) -> Vec<T>
    where
        P: Predicate<T>,
    {
        items.into_iter().filter(|item| pred.test(item)).collect()
    }

    #[test]
    fn test_with_box_predicate() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let result = filter_by_predicate(vec![-2, -1, 0, 1, 2], pred);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_with_rc_predicate() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let pred_clone = pred.clone();
        let result = filter_by_predicate(vec![-2, -1, 0, 1, 2], pred_clone);
        assert_eq!(result, vec![1, 2]);

        // pred is still usable
        assert!(pred.test(&5));
    }

    #[test]
    fn test_with_arc_predicate() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let pred_clone = pred.clone();
        let result = filter_by_predicate(vec![-2, -1, 0, 1, 2], pred_clone);
        assert_eq!(result, vec![1, 2]);

        // pred is still usable
        assert!(pred.test(&5));
    }

    #[test]
    fn test_with_closure() {
        let pred = |x: &i32| *x > 0;
        let result = filter_by_predicate(vec![-2, -1, 0, 1, 2], pred);
        assert_eq!(result, vec![1, 2]);
    }
}

#[cfg(test)]
mod logical_operations_tests {
    use super::*;

    // BoxPredicate NAND tests
    #[test]
    fn test_box_nand_basic() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let nand = is_positive.nand(is_even);

        // NAND: true unless both are true
        assert!(nand.test(&3)); // positive but odd: true && false = false, !false = true
        assert!(nand.test(&-2)); // negative but even: false && true = false, !false = true
        assert!(nand.test(&-1)); // negative and odd: false && false = false, !false = true
        assert!(!nand.test(&4)); // positive and even: true && true = true, !true = false
    }

    // BoxPredicate XOR tests
    #[test]
    fn test_box_xor_basic() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let xor = is_positive.xor(is_even);

        // XOR: true if exactly one is true
        assert!(xor.test(&3)); // positive but odd: true ^ false = true
        assert!(xor.test(&-2)); // negative but even: false ^ true = true
        assert!(!xor.test(&-1)); // negative and odd: false ^ false = false
        assert!(!xor.test(&4)); // positive and even: true ^ true = false
    }

    // BoxPredicate NOR tests
    #[test]
    fn test_box_nor_basic() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let nor = is_positive.nor(is_even);

        // NOR: true only when both are false
        assert!(nor.test(&-3)); // negative and odd: !(false || false) = true
        assert!(!nor.test(&3)); // positive but odd: !(true || false) = false
        assert!(!nor.test(&-2)); // negative but even: !(false || true) = false
        assert!(!nor.test(&4)); // positive and even: !(true || true) = false
    }

    // RcPredicate NAND tests
    #[test]
    fn test_rc_nand_basic() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let nand = is_positive.nand(is_even.clone());

        assert!(nand.test(&3)); // positive but odd
        assert!(nand.test(&-2)); // negative but even
        assert!(nand.test(&-1)); // negative and odd
        assert!(!nand.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // RcPredicate XOR tests
    #[test]
    fn test_rc_xor_basic() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let xor = is_positive.xor(is_even.clone());

        assert!(xor.test(&3)); // positive but odd
        assert!(xor.test(&-2)); // negative but even
        assert!(!xor.test(&-1)); // negative and odd
        assert!(!xor.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // RcPredicate NOR tests
    #[test]
    fn test_rc_nor_basic() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let nor = is_positive.nor(is_even.clone());

        // NOR: true only when both are false
        assert!(nor.test(&-3)); // negative and odd: !(false || false) = true
        assert!(!nor.test(&3)); // positive but odd: !(true || false) = false
        assert!(!nor.test(&-2)); // negative but even: !(false || true) = false
        assert!(!nor.test(&4)); // positive and even: !(true || true) = false

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // ArcPredicate NAND tests
    #[test]
    fn test_arc_nand_basic() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let nand = is_positive.nand(is_even.clone());

        assert!(nand.test(&3)); // positive but odd
        assert!(nand.test(&-2)); // negative but even
        assert!(nand.test(&-1)); // negative and odd
        assert!(!nand.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // ArcPredicate XOR tests
    #[test]
    fn test_arc_xor_basic() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let xor = is_positive.xor(is_even.clone());

        assert!(xor.test(&3)); // positive but odd
        assert!(xor.test(&-2)); // negative but even
        assert!(!xor.test(&-1)); // negative and odd
        assert!(!xor.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // ArcPredicate NOR tests
    #[test]
    fn test_arc_nor_basic() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let nor = is_positive.nor(is_even.clone());

        // NOR: true only when both are false
        assert!(nor.test(&-3)); // negative and odd: !(false || false) = true
        assert!(!nor.test(&3)); // positive but odd: !(true || false) = false
        assert!(!nor.test(&-2)); // negative but even: !(false || true) = false
        assert!(!nor.test(&4)); // positive and even: !(true || true) = false

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // Closure NAND tests (via FnPredicateOps)
    #[test]
    fn test_closure_nand_basic() {
        let is_positive = |x: &i32| *x > 0;
        let is_even = |x: &i32| x % 2 == 0;

        let nand = is_positive.nand(is_even);

        assert!(nand.test(&3)); // positive but odd
        assert!(nand.test(&-2)); // negative but even
        assert!(nand.test(&-1)); // negative and odd
        assert!(!nand.test(&4)); // positive and even
    }

    // Closure XOR tests (via FnPredicateOps)
    #[test]
    fn test_closure_xor_basic() {
        let is_positive = |x: &i32| *x > 0;
        let is_even = |x: &i32| x % 2 == 0;

        let xor = is_positive.xor(is_even);

        assert!(xor.test(&3)); // positive but odd
        assert!(xor.test(&-2)); // negative but even
        assert!(!xor.test(&-1)); // negative and odd
        assert!(!xor.test(&4)); // positive and even
    }

    // Closure NOR tests (via FnPredicateOps)
    #[test]
    fn test_closure_nor_basic() {
        let is_positive = |x: &i32| *x > 0;
        let is_even = |x: &i32| x % 2 == 0;

        let nor = is_positive.nor(is_even);

        // NOR: true only when both are false
        assert!(nor.test(&-3)); // negative and odd: !(false || false) = true
        assert!(!nor.test(&3)); // positive but odd: !(true || false) = false
        assert!(!nor.test(&-2)); // negative but even: !(false || true) = false
        assert!(!nor.test(&4)); // positive and even: !(true || true) = false
    }

    // Complex composition with NAND
    #[test]
    fn test_complex_nand_composition() {
        let is_positive = |x: &i32| *x > 0;
        let is_even = |x: &i32| x % 2 == 0;
        let is_small = |x: &i32| x.abs() < 10;

        // (positive NAND even) AND small
        let complex = is_positive.nand(is_even).and(BoxPredicate::new(is_small));

        assert!(complex.test(&3)); // !(true && false) && true = true && true = true
        assert!(complex.test(&-2)); // !(false && true) && true = true && true = true
        assert!(!complex.test(&4)); // !(true && true) && true = false && true = false
        assert!(!complex.test(&15)); // !(true && false) && false = true && false = false
    }

    // Complex composition with XOR
    #[test]
    fn test_complex_xor_composition() {
        let is_positive = |x: &i32| *x > 0;
        let is_even = |x: &i32| x % 2 == 0;
        let is_small = |x: &i32| x.abs() < 10;

        // (positive XOR even) AND small
        let complex = is_positive.xor(is_even).and(BoxPredicate::new(is_small));

        assert!(complex.test(&3)); // (true ^ false) && true = true && true = true
        assert!(complex.test(&-2)); // (false ^ true) && true = true && true = true
        assert!(!complex.test(&4)); // (true ^ true) && true = false && true = false
        assert!(!complex.test(&-1)); // (false ^ false) && true = false && true = false
    }

    // NAND with string predicates
    #[test]
    fn test_nand_with_strings() {
        let is_long = BoxPredicate::new(|s: &String| s.len() > 5);
        let has_uppercase = BoxPredicate::new(|s: &String| s.chars().any(|c| c.is_uppercase()));

        let nand = is_long.nand(has_uppercase);

        assert!(nand.test(&"hello".to_string())); // short, no uppercase: !(false && false) = true
        assert!(nand.test(&"Hello".to_string())); // short, has uppercase: !(false && true) = true
        assert!(nand.test(&"goodbye".to_string())); // long, no uppercase: !(true && false) = true
        assert!(!nand.test(&"HelloWorld".to_string())); // long, has uppercase: !(true && true) = false
    }

    // XOR with string predicates
    #[test]
    fn test_xor_with_strings() {
        let is_long = BoxPredicate::new(|s: &String| s.len() > 5);
        let has_uppercase = BoxPredicate::new(|s: &String| s.chars().any(|c| c.is_uppercase()));

        let xor = is_long.xor(has_uppercase);

        assert!(!xor.test(&"hello".to_string())); // short, no uppercase: false ^ false = false
        assert!(xor.test(&"Hello".to_string())); // short, has uppercase: false ^ true = true
        assert!(xor.test(&"goodbye".to_string())); // long, no uppercase: true ^ false = true
        assert!(!xor.test(&"HelloWorld".to_string())); // long, has uppercase: true ^ true = false
    }

    // NOR with string predicates
    #[test]
    fn test_nor_with_strings() {
        let is_long = BoxPredicate::new(|s: &String| s.len() > 5);
        let has_uppercase = BoxPredicate::new(|s: &String| s.chars().any(|c| c.is_uppercase()));

        let nor = is_long.nor(has_uppercase);

        assert!(nor.test(&"hello".to_string())); // short, no uppercase: !(false || false) = true
        assert!(!nor.test(&"Hello".to_string())); // short, has uppercase: !(false || true) = false
        assert!(!nor.test(&"goodbye".to_string())); // long, no uppercase: !(true || false) = false
        assert!(!nor.test(&"HelloWorld".to_string())); // long, has uppercase: !(true || true) = false
    }
}

#[cfg(test)]
mod parameter_types_tests {
    use super::*;

    // Helper functions
    fn is_even(x: &i32) -> bool {
        x % 2 == 0
    }

    fn is_large(x: &i32) -> bool {
        *x > 100
    }

    // ============================================================================
    // BoxPredicate::and parameter type tests
    // ============================================================================

    #[test]
    fn test_box_and_with_closure() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let combined = pred.and(|x: &i32| x % 2 == 0);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(!combined.test(&-2));
    }

    #[test]
    fn test_box_and_with_function() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let combined = pred.and(is_even);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(!combined.test(&-2));
    }

    #[test]
    fn test_box_and_with_box_predicate() {
        let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
        let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
        let combined = pred1.and(pred2);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(!combined.test(&-2));
    }

    #[test]
    fn test_box_and_with_rc_predicate() {
        let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
        let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);
        let combined = pred1.and(pred2);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    // ============================================================================
    // BoxPredicate::or parameter type tests
    // ============================================================================

    #[test]
    fn test_box_or_with_closure() {
        let pred = BoxPredicate::new(|x: &i32| *x < 0);
        let combined = pred.or(|x: &i32| *x > 100);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(!combined.test(&50));
    }

    #[test]
    fn test_box_or_with_function() {
        let pred = BoxPredicate::new(|x: &i32| *x < 0);
        let combined = pred.or(is_large);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(!combined.test(&50));
    }

    #[test]
    fn test_box_or_with_box_predicate() {
        let pred1 = BoxPredicate::new(|x: &i32| *x < 0);
        let pred2 = BoxPredicate::new(|x: &i32| *x > 100);
        let combined = pred1.or(pred2);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(!combined.test(&50));
    }

    // ============================================================================
    // BoxPredicate::nand parameter type tests
    // ============================================================================

    #[test]
    fn test_box_nand_with_closure() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let nand = pred.nand(|x: &i32| x % 2 == 0);

        assert!(nand.test(&3)); // !(true && false)
        assert!(!nand.test(&4)); // !(true && true)
    }

    #[test]
    fn test_box_nand_with_function() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let nand = pred.nand(is_even);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
    }

    #[test]
    fn test_box_nand_with_box_predicate() {
        let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
        let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
        let nand = pred1.nand(pred2);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
    }

    // ============================================================================
    // BoxPredicate::xor parameter type tests
    // ============================================================================

    #[test]
    fn test_box_xor_with_closure() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let xor = pred.xor(|x: &i32| x % 2 == 0);

        assert!(xor.test(&3)); // true ^ false
        assert!(!xor.test(&4)); // true ^ true
        assert!(!xor.test(&-1)); // false ^ false
    }

    #[test]
    fn test_box_xor_with_function() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let xor = pred.xor(is_even);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(!xor.test(&-1));
    }

    #[test]
    fn test_box_xor_with_box_predicate() {
        let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
        let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
        let xor = pred1.xor(pred2);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(!xor.test(&-1));
    }

    // ============================================================================
    // BoxPredicate::nor parameter type tests
    // ============================================================================

    #[test]
    fn test_box_nor_with_closure() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let nor = pred.nor(|x: &i32| x % 2 == 0);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(!nor.test(&3));
    }

    #[test]
    fn test_box_nor_with_function() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let nor = pred.nor(is_even);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
    }

    #[test]
    fn test_box_nor_with_box_predicate() {
        let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
        let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
        let nor = pred1.nor(pred2);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(!nor.test(&3));
    }

    // ============================================================================
    // RcPredicate::and parameter type tests
    // ============================================================================

    #[test]
    fn test_rc_and_with_closure() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let combined = pred.and(|x: &i32| x % 2 == 0);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));

        // Original predicate is still usable
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_and_with_function() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let combined = pred.and(is_even);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_and_with_rc_predicate() {
        let pred1 = RcPredicate::new(|x: &i32| *x > 0);
        let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);
        let combined = pred1.and(pred2.clone());

        assert!(combined.test(&4));
        assert!(!combined.test(&3));

        // Both original predicates are still usable
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    #[test]
    fn test_rc_and_with_box_predicate() {
        let pred1 = RcPredicate::new(|x: &i32| *x > 0);
        let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
        let combined = pred1.and(pred2);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(pred1.test(&5));
    }

    // ============================================================================
    // RcPredicate::or parameter type tests
    // ============================================================================

    #[test]
    fn test_rc_or_with_closure() {
        let pred = RcPredicate::new(|x: &i32| *x < 0);
        let combined = pred.or(|x: &i32| *x > 100);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(!combined.test(&50));
        assert!(pred.test(&-10));
    }

    #[test]
    fn test_rc_or_with_function() {
        let pred = RcPredicate::new(|x: &i32| *x < 0);
        let combined = pred.or(is_large);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(pred.test(&-10));
    }

    #[test]
    fn test_rc_or_with_rc_predicate() {
        let pred1 = RcPredicate::new(|x: &i32| *x < 0);
        let pred2 = RcPredicate::new(|x: &i32| *x > 100);
        let combined = pred1.or(pred2.clone());

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(pred1.test(&-10));
        assert!(pred2.test(&150));
    }

    // ============================================================================
    // RcPredicate::nand parameter type tests
    // ============================================================================

    #[test]
    fn test_rc_nand_with_closure() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let nand = pred.nand(|x: &i32| x % 2 == 0);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_nand_with_function() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let nand = pred.nand(is_even);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_nand_with_rc_predicate() {
        let pred1 = RcPredicate::new(|x: &i32| *x > 0);
        let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);
        let nand = pred1.nand(pred2.clone());

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    // ============================================================================
    // RcPredicate::xor parameter type tests
    // ============================================================================

    #[test]
    fn test_rc_xor_with_closure() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let xor = pred.xor(|x: &i32| x % 2 == 0);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(!xor.test(&-1));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_xor_with_function() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let xor = pred.xor(is_even);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_xor_with_rc_predicate() {
        let pred1 = RcPredicate::new(|x: &i32| *x > 0);
        let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);
        let xor = pred1.xor(pred2.clone());

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    // ============================================================================
    // RcPredicate::nor parameter type tests
    // ============================================================================

    #[test]
    fn test_rc_nor_with_closure() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let nor = pred.nor(|x: &i32| x % 2 == 0);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_nor_with_function() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let nor = pred.nor(is_even);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_rc_nor_with_rc_predicate() {
        let pred1 = RcPredicate::new(|x: &i32| *x > 0);
        let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);
        let nor = pred1.nor(pred2.clone());

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    // ============================================================================
    // ArcPredicate::and parameter type tests
    // ============================================================================

    #[test]
    fn test_arc_and_with_closure() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let combined = pred.and(|x: &i32| x % 2 == 0);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_and_with_function() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let combined = pred.and(is_even);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_and_with_arc_predicate() {
        let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
        let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);
        let combined = pred1.and(pred2.clone());

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    // ============================================================================
    // ArcPredicate::or parameter type tests
    // ============================================================================

    #[test]
    fn test_arc_or_with_closure() {
        let pred = ArcPredicate::new(|x: &i32| *x < 0);
        let combined = pred.or(|x: &i32| *x > 100);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(!combined.test(&50));
        assert!(pred.test(&-10));
    }

    #[test]
    fn test_arc_or_with_function() {
        let pred = ArcPredicate::new(|x: &i32| *x < 0);
        let combined = pred.or(is_large);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(pred.test(&-10));
    }

    #[test]
    fn test_arc_or_with_arc_predicate() {
        let pred1 = ArcPredicate::new(|x: &i32| *x < 0);
        let pred2 = ArcPredicate::new(|x: &i32| *x > 100);
        let combined = pred1.or(pred2.clone());

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(pred1.test(&-10));
        assert!(pred2.test(&150));
    }

    // ============================================================================
    // ArcPredicate::nand parameter type tests
    // ============================================================================

    #[test]
    fn test_arc_nand_with_closure() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let nand = pred.nand(|x: &i32| x % 2 == 0);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_nand_with_function() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let nand = pred.nand(is_even);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_nand_with_arc_predicate() {
        let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
        let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);
        let nand = pred1.nand(pred2.clone());

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    // ============================================================================
    // ArcPredicate::xor parameter type tests
    // ============================================================================

    #[test]
    fn test_arc_xor_with_closure() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let xor = pred.xor(|x: &i32| x % 2 == 0);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(!xor.test(&-1));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_xor_with_function() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let xor = pred.xor(is_even);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_xor_with_arc_predicate() {
        let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
        let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);
        let xor = pred1.xor(pred2.clone());

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    // ============================================================================
    // ArcPredicate::nor parameter type tests
    // ============================================================================

    #[test]
    fn test_arc_nor_with_closure() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let nor = pred.nor(|x: &i32| x % 2 == 0);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(!nor.test(&3));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_nor_with_function() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let nor = pred.nor(is_even);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(pred.test(&5));
    }

    #[test]
    fn test_arc_nor_with_arc_predicate() {
        let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
        let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);
        let nor = pred1.nor(pred2.clone());

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(pred1.test(&5));
        assert!(pred2.test(&6));
    }

    // ============================================================================
    // FnPredicateOps (closure) parameter type tests
    // ============================================================================

    #[test]
    fn test_closure_and_with_closure() {
        let is_pos = |x: &i32| *x > 0;
        let is_even_closure = |x: &i32| x % 2 == 0;

        let combined = is_pos.and(is_even_closure);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_closure_and_with_function() {
        let is_pos = |x: &i32| *x > 0;
        let combined = is_pos.and(is_even);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_closure_and_with_box_predicate() {
        let is_pos = |x: &i32| *x > 0;
        let pred = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let combined = is_pos.and(pred);

        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_closure_or_with_closure() {
        let is_neg = |x: &i32| *x < 0;
        let is_large_closure = |x: &i32| *x > 100;

        let combined = is_neg.or(is_large_closure);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
        assert!(!combined.test(&50));
    }

    #[test]
    fn test_closure_or_with_function() {
        let is_neg = |x: &i32| *x < 0;
        let combined = is_neg.or(is_large);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
    }

    #[test]
    fn test_closure_or_with_box_predicate() {
        let is_neg = |x: &i32| *x < 0;
        let pred = BoxPredicate::new(|x: &i32| *x > 100);

        let combined = is_neg.or(pred);

        assert!(combined.test(&-5));
        assert!(combined.test(&150));
    }

    #[test]
    fn test_closure_nand_with_closure() {
        let is_pos = |x: &i32| *x > 0;
        let is_even_closure = |x: &i32| x % 2 == 0;

        let nand = is_pos.nand(is_even_closure);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
    }

    #[test]
    fn test_closure_nand_with_function() {
        let is_pos = |x: &i32| *x > 0;
        let nand = is_pos.nand(is_even);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
    }

    #[test]
    fn test_closure_nand_with_box_predicate() {
        let is_pos = |x: &i32| *x > 0;
        let pred = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let nand = is_pos.nand(pred);

        assert!(nand.test(&3));
        assert!(!nand.test(&4));
    }

    #[test]
    fn test_closure_xor_with_closure() {
        let is_pos = |x: &i32| *x > 0;
        let is_even_closure = |x: &i32| x % 2 == 0;

        let xor = is_pos.xor(is_even_closure);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
        assert!(!xor.test(&-1));
    }

    #[test]
    fn test_closure_xor_with_function() {
        let is_pos = |x: &i32| *x > 0;
        let xor = is_pos.xor(is_even);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
    }

    #[test]
    fn test_closure_xor_with_box_predicate() {
        let is_pos = |x: &i32| *x > 0;
        let pred = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let xor = is_pos.xor(pred);

        assert!(xor.test(&3));
        assert!(!xor.test(&4));
    }

    #[test]
    fn test_closure_nor_with_closure() {
        let is_pos = |x: &i32| *x > 0;
        let is_even_closure = |x: &i32| x % 2 == 0;

        let nor = is_pos.nor(is_even_closure);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
        assert!(!nor.test(&3));
    }

    #[test]
    fn test_closure_nor_with_function() {
        let is_pos = |x: &i32| *x > 0;
        let nor = is_pos.nor(is_even);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
    }

    #[test]
    fn test_closure_nor_with_box_predicate() {
        let is_pos = |x: &i32| *x > 0;
        let pred = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let nor = is_pos.nor(pred);

        assert!(nor.test(&-3));
        assert!(!nor.test(&4));
    }
}

#[cfg(test)]
mod always_predicates_tests {
    use super::*;

    #[test]
    fn test_box_always_true() {
        let pred = BoxPredicate::<i32>::always_true();
        assert!(pred.test(&5));
        assert!(pred.test(&-5));
        assert!(pred.test(&0));
    }

    #[test]
    fn test_box_always_false() {
        let pred = BoxPredicate::<i32>::always_false();
        assert!(!pred.test(&5));
        assert!(!pred.test(&-5));
        assert!(!pred.test(&0));
    }

    #[test]
    fn test_rc_always_true() {
        let pred = RcPredicate::<i32>::always_true();
        assert!(pred.test(&5));
        assert!(pred.test(&-5));
        assert!(pred.test(&0));
    }

    #[test]
    fn test_rc_always_false() {
        let pred = RcPredicate::<i32>::always_false();
        assert!(!pred.test(&5));
        assert!(!pred.test(&-5));
        assert!(!pred.test(&0));
    }

    #[test]
    fn test_arc_always_true() {
        let pred = ArcPredicate::<i32>::always_true();
        assert!(pred.test(&5));
        assert!(pred.test(&-5));
        assert!(pred.test(&0));
    }

    #[test]
    fn test_arc_always_false() {
        let pred = ArcPredicate::<i32>::always_false();
        assert!(!pred.test(&5));
        assert!(!pred.test(&-5));
        assert!(!pred.test(&0));
    }

    #[test]
    fn test_always_true_with_composition() {
        let always = BoxPredicate::<i32>::always_true();
        let is_positive = |x: &i32| *x > 0;

        let and_result = always.and(is_positive);
        assert!(and_result.test(&5));
        assert!(!and_result.test(&-5));
    }

    #[test]
    fn test_always_false_with_composition() {
        let never = BoxPredicate::<i32>::always_false();
        let is_positive = |x: &i32| *x > 0;

        let or_result = never.or(is_positive);
        assert!(or_result.test(&5));
        assert!(!or_result.test(&-5));
    }

    #[test]
    fn test_new_with_name() {
        let mut pred = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("positive"));
        assert!(pred.test(&5));

        pred.set_name("updated");
        assert_eq!(pred.name(), Some("updated"));
    }

    #[test]
    fn test_rc_new_with_name() {
        let mut pred = RcPredicate::new_with_name("positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("positive"));
        assert!(pred.test(&5));

        pred.set_name("updated");
        assert_eq!(pred.name(), Some("updated"));
    }

    #[test]
    fn test_arc_new_with_name() {
        let mut pred = ArcPredicate::new_with_name("positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("positive"));
        assert!(pred.test(&5));

        pred.set_name("updated");
        assert_eq!(pred.name(), Some("updated"));
    }
}

#[cfg(test)]
mod to_fn_tests {
    use super::*;

    #[test]
    fn test_rc_to_fn() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let func = pred.to_fn();

        assert!(func(&5));
        assert!(!func(&-5));
        assert!(!func(&0));
    }

    #[test]
    fn test_rc_to_fn_multiple_calls() {
        let pred = RcPredicate::new(|x: &i32| *x % 2 == 0);
        let func = pred.to_fn();

        assert!(func(&2));
        assert!(func(&4));
        assert!(!func(&3));
        assert!(!func(&5));
    }

    #[test]
    fn test_arc_to_fn() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let func = pred.to_fn();

        assert!(func(&5));
        assert!(!func(&-5));
        assert!(!func(&0));
    }

    #[test]
    fn test_arc_to_fn_multiple_calls() {
        let pred = ArcPredicate::new(|x: &i32| *x % 2 == 0);
        let func = pred.to_fn();

        assert!(func(&2));
        assert!(func(&4));
        assert!(!func(&3));
        assert!(!func(&5));
    }

    #[test]
    fn test_rc_to_fn_with_composition() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let combined = is_positive.and(is_even);
        let func = combined.to_fn();

        assert!(func(&4));
        assert!(!func(&3));
        assert!(!func(&-2));
    }

    #[test]
    fn test_arc_to_fn_with_composition() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let combined = is_positive.and(is_even);
        let func = combined.to_fn();

        assert!(func(&4));
        assert!(!func(&3));
        assert!(!func(&-2));
    }
}

#[cfg(test)]
mod not_composition_tests {
    use super::*;

    #[test]
    fn test_box_not_and_composition() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.and(is_even);

        assert!(combined.test(&-2));
        assert!(!combined.test(&-3));
        assert!(!combined.test(&4));
    }

    #[test]
    fn test_box_not_or_composition() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.or(is_even);

        assert!(combined.test(&-3));
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_rc_not_and_composition() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.and(is_even);

        assert!(combined.test(&-2));
        assert!(!combined.test(&-3));
        assert!(!combined.test(&4));
    }

    #[test]
    fn test_rc_not_or_composition() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.or(is_even);

        assert!(combined.test(&-3));
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_arc_not_and_composition() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.and(is_even);

        assert!(combined.test(&-2));
        assert!(!combined.test(&-3));
        assert!(!combined.test(&4));
    }

    #[test]
    fn test_arc_not_or_composition() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.or(is_even);

        assert!(combined.test(&-3));
        assert!(combined.test(&4));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_double_not() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let not_positive = is_positive.not();
        let double_not = not_positive.not();

        assert!(double_not.test(&5));
        assert!(!double_not.test(&-5));
    }

    #[test]
    fn test_not_with_nand() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.nand(is_even);

        assert!(combined.test(&-3));
        assert!(combined.test(&3));
        assert!(!combined.test(&-2));
    }

    #[test]
    fn test_not_with_xor() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.xor(is_even);

        assert!(combined.test(&-3));
        assert!(combined.test(&4));
        assert!(!combined.test(&-2));
        assert!(!combined.test(&3));
    }

    #[test]
    fn test_not_with_nor() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let not_positive = is_positive.not();
        let combined = not_positive.nor(is_even);

        assert!(combined.test(&3));
        assert!(!combined.test(&-3));
        assert!(!combined.test(&4));
        assert!(!combined.test(&-2));
    }
}

// ============================================================================
// Additional Type Conversion Tests
// ============================================================================

#[cfg(test)]
mod additional_type_conversion_tests {
    use super::*;

    #[test]
    fn test_box_into_box() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let boxed = pred.into_box();
        assert!(boxed.test(&5));
        assert!(!boxed.test(&-3));
    }

    #[test]
    fn test_box_into_rc() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let rc = pred.into_rc();
        assert!(rc.test(&5));
        assert!(!rc.test(&-3));
    }

    #[test]
    fn test_box_into_fn() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let func = pred.into_fn();
        assert!(func(&5));
        assert!(!func(&-3));
    }

    #[test]
    fn test_arc_into_arc() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let arc = pred.into_arc();
        assert!(arc.test(&5));
        assert!(!arc.test(&-3));
    }

    #[test]
    fn test_arc_into_box() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let boxed = pred.into_box();
        assert!(boxed.test(&5));
        assert!(!boxed.test(&-3));
    }

    #[test]
    fn test_arc_into_rc() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let rc = pred.into_rc();
        assert!(rc.test(&5));
        assert!(!rc.test(&-3));
    }

    #[test]
    fn test_arc_into_fn() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let func = pred.into_fn();
        assert!(func(&5));
        assert!(!func(&-3));
    }

    #[test]
    fn test_rc_into_rc() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let rc = pred.into_rc();
        assert!(rc.test(&5));
        assert!(!rc.test(&-3));
    }

    #[test]
    fn test_rc_into_box() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let boxed = pred.into_box();
        assert!(boxed.test(&5));
        assert!(!boxed.test(&-3));
    }

    #[test]
    fn test_rc_into_fn() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let func = pred.into_fn();
        assert!(func(&5));
        assert!(!func(&-3));
    }
}

// ============================================================================
// Display and Debug Tests
// ============================================================================

#[cfg(test)]
mod display_debug_tests {
    use super::*;

    #[test]
    fn test_box_display_unnamed() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let display_str = format!("{}", pred);
        assert_eq!(display_str, "BoxPredicate(unnamed)");
    }

    #[test]
    fn test_box_display_named() {
        let pred = BoxPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
        let display_str = format!("{}", pred);
        assert_eq!(display_str, "BoxPredicate(is_positive)");
    }

    #[test]
    fn test_box_debug() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", pred);
        assert!(debug_str.contains("BoxPredicate"));
        assert!(debug_str.contains("name"));
    }

    #[test]
    fn test_arc_display_unnamed() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let display_str = format!("{}", pred);
        assert_eq!(display_str, "ArcPredicate(unnamed)");
    }

    #[test]
    fn test_arc_display_named() {
        let pred = ArcPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
        let display_str = format!("{}", pred);
        assert_eq!(display_str, "ArcPredicate(is_positive)");
    }

    #[test]
    fn test_arc_debug() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", pred);
        assert!(debug_str.contains("ArcPredicate"));
        assert!(debug_str.contains("name"));
    }

    #[test]
    fn test_rc_display_unnamed() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let display_str = format!("{}", pred);
        assert_eq!(display_str, "RcPredicate(unnamed)");
    }

    #[test]
    fn test_rc_display_named() {
        let pred = RcPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
        let display_str = format!("{}", pred);
        assert_eq!(display_str, "RcPredicate(is_positive)");
    }

    #[test]
    fn test_rc_debug() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", pred);
        assert!(debug_str.contains("RcPredicate"));
        assert!(debug_str.contains("name"));
    }
}
