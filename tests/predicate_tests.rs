/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

#[cfg(test)]
mod tests {
    use prism3_function::predicate::{
        ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate,
    };
    use std::thread;

    // ========================================================================
    // Predicate Trait Tests - Test closure and function pointer implementations
    // ========================================================================

    mod predicate_trait_tests {
        use super::*;

        #[test]
        fn test_closure_implements_predicate() {
            let is_positive = |x: &i32| *x > 0;
            assert!(is_positive.test(&5));
            assert!(!is_positive.test(&-3));
            assert!(!is_positive.test(&0));
        }

        #[test]
        fn test_function_pointer_implements_predicate() {
            fn is_even(x: &i32) -> bool {
                x % 2 == 0
            }

            assert!(is_even.test(&4));
            assert!(!is_even.test(&5));
        }

        #[test]
        fn test_predicate_with_different_types() {
            // Test String
            let is_empty = |s: &String| s.is_empty();
            assert!(is_empty.test(&String::new()));
            assert!(!is_empty.test(&String::from("hello")));

            // Test &str
            let starts_with_a = |s: &&str| s.starts_with('a');
            assert!(starts_with_a.test(&"apple"));
            assert!(!starts_with_a.test(&"banana"));

            // Test f64
            let is_nan = |x: &f64| x.is_nan();
            assert!(is_nan.test(&f64::NAN));
            assert!(!is_nan.test(&1.0));

            // Test bool
            let is_true = |x: &bool| *x;
            assert!(is_true.test(&true));
            assert!(!is_true.test(&false));
        }
    }

    // ========================================================================
    // FnPredicateOps Tests - Test extension methods for closures
    // ========================================================================

    mod predicate_ext_tests {
        use super::*;

        #[test]
        fn test_closure_and() {
            let is_positive = |x: &i32| *x > 0;
            let is_even = |x: &i32| x % 2 == 0;

            let combined = is_positive.and(is_even);
            assert!(combined.test(&4));
            assert!(combined.test(&10));
            assert!(!combined.test(&3)); // Positive but odd
            assert!(!combined.test(&-4)); // Even but negative
            assert!(!combined.test(&0)); // Even but not positive
        }

        #[test]
        fn test_closure_or() {
            let is_negative = |x: &i32| *x < 0;
            let is_zero = |x: &i32| *x == 0;

            let combined = is_negative.or(is_zero);
            assert!(combined.test(&-5));
            assert!(combined.test(&0));
            assert!(!combined.test(&5));
        }

        #[test]
        fn test_closure_not() {
            let is_positive = |x: &i32| *x > 0;
            let is_not_positive = is_positive.not();

            assert!(!is_not_positive.test(&5));
            assert!(is_not_positive.test(&-3));
            assert!(is_not_positive.test(&0));
        }

        #[test]
        fn test_closure_xor() {
            let is_positive = |x: &i32| *x > 0;
            let is_even = |x: &i32| x % 2 == 0;

            let combined = is_positive.xor(is_even);
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
            assert!(!combined.test(&4)); // Positive and even
            assert!(!combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_closure_nand() {
            let is_positive = |x: &i32| *x > 0;
            let is_even = |x: &i32| x % 2 == 0;

            let combined = is_positive.nand(is_even);
            assert!(!combined.test(&4)); // Positive and even
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
            assert!(combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_closure_nor() {
            let is_positive = |x: &i32| *x > 0;
            let is_even = |x: &i32| x % 2 == 0;

            let combined = is_positive.nor(is_even);
            assert!(!combined.test(&4)); // Positive and even
            assert!(!combined.test(&3)); // Positive but odd
            assert!(!combined.test(&-4)); // Even but negative
            assert!(combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_closure_chain_combination() {
            let is_positive = |x: &i32| *x > 0;
            let is_even = |x: &i32| x % 2 == 0;
            let less_than_100 = |x: &i32| *x < 100;

            // (positive AND even) OR less_than_100
            let combined = is_positive.and(is_even).or(less_than_100);
            assert!(combined.test(&4)); // Positive even
            assert!(combined.test(&-5)); // Less than 100
            assert!(combined.test(&50)); // Both conditions satisfied
            assert!(!combined.test(&101)); // Neither condition satisfied
        }
    }

    // ========================================================================
    // BoxPredicate Tests
    // ========================================================================

    mod box_predicate_tests {
        use super::*;

        #[test]
        fn test_new() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0);
            assert!(pred.test(&5));
            assert!(!pred.test(&-3));
        }

        #[test]
        fn test_with_name() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            assert_eq!(pred.name(), Some("is_positive"));
        }

        #[test]
        fn test_name_none() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0);
            assert_eq!(pred.name(), None);
        }

        #[test]
        fn test_test_method() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0);
            assert!(pred.test(&1));
            assert!(pred.test(&100));
            assert!(!pred.test(&0));
            assert!(!pred.test(&-1));
        }

        #[test]
        fn test_and() {
            let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
            let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.and(pred2);
            assert!(combined.test(&4));
            assert!(!combined.test(&3));
            assert!(!combined.test(&-4));
        }

        #[test]
        fn test_and_with_closure() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0);
            let combined = pred.and(|x: &i32| x % 2 == 0);

            assert!(combined.test(&4));
            assert!(!combined.test(&3));
        }

        #[test]
        fn test_or() {
            let pred1 = BoxPredicate::new(|x: &i32| *x < 0);
            let pred2 = BoxPredicate::new(|x: &i32| *x > 10);

            let combined = pred1.or(pred2);
            assert!(combined.test(&-5));
            assert!(combined.test(&15));
            assert!(!combined.test(&5));
        }

        #[test]
        fn test_not() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0);
            let negated = pred.not();

            assert!(!negated.test(&5));
            assert!(negated.test(&-3));
            assert!(negated.test(&0));
        }

        #[test]
        fn test_xor() {
            let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
            let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.xor(pred2);
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
            assert!(!combined.test(&4)); // Positive and even
            assert!(!combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_nand() {
            let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
            let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.nand(pred2);
            assert!(!combined.test(&4)); // Positive and even
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
        }

        #[test]
        fn test_nor() {
            let pred1 = BoxPredicate::new(|x: &i32| *x > 0);
            let pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.nor(pred2);
            assert!(!combined.test(&4)); // Positive or even
            assert!(!combined.test(&3)); // Positive
            assert!(!combined.test(&-4)); // Even
            assert!(combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_chain_combination() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0)
                .and(|x: &i32| x % 2 == 0)
                .or(|x: &i32| *x < -10);

            assert!(pred.test(&4)); // Positive even
            assert!(pred.test(&-15)); // Less than -10
            assert!(!pred.test(&3)); // Positive odd
            assert!(!pred.test(&-5)); // Between -10 and 0
        }

        #[test]
        fn test_display() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            assert_eq!(format!("{}", pred), "BoxPredicate(is_positive)");

            let unnamed = BoxPredicate::new(|x: &i32| *x > 0);
            assert_eq!(format!("{}", unnamed), "BoxPredicate(unnamed)");
        }

        #[test]
        fn test_debug() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let debug_str = format!("{:?}", pred);
            assert!(debug_str.contains("BoxPredicate"));
            assert!(debug_str.contains("is_positive"));

            let unnamed = BoxPredicate::new(|x: &i32| *x > 0);
            let debug_str = format!("{:?}", unnamed);
            assert!(debug_str.contains("BoxPredicate"));
            assert!(debug_str.contains("None"));
        }

        #[test]
        fn test_with_different_types() {
            // String
            let string_pred = BoxPredicate::new(|s: &String| s.len() > 5);
            assert!(string_pred.test(&String::from("hello world")));
            assert!(!string_pred.test(&String::from("hi")));

            // f64
            let float_pred = BoxPredicate::new(|x: &f64| *x > 0.0);
            assert!(float_pred.test(&1.5));
            assert!(!float_pred.test(&-1.5));

            // bool
            let bool_pred = BoxPredicate::new(|x: &bool| *x);
            assert!(bool_pred.test(&true));
            assert!(!bool_pred.test(&false));

            // Vec
            let vec_pred = BoxPredicate::new(|v: &Vec<i32>| !v.is_empty());
            assert!(vec_pred.test(&vec![1, 2, 3]));
            assert!(!vec_pred.test(&vec![]));
        }
    }

    // ========================================================================
    // ArcPredicate Tests
    // ========================================================================

    mod arc_predicate_tests {
        use super::*;

        #[test]
        fn test_new() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0);
            assert!(pred.test(&5));
            assert!(!pred.test(&-3));
        }

        #[test]
        fn test_with_name() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            assert_eq!(pred.name(), Some("is_positive"));
        }

        #[test]
        fn test_name_none() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0);
            assert_eq!(pred.name(), None);
        }

        #[test]
        fn test_test_method() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0);
            assert!(pred.test(&1));
            assert!(pred.test(&100));
            assert!(!pred.test(&0));
            assert!(!pred.test(&-1));
        }

        #[test]
        fn test_clone() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0);
            let cloned = pred.clone();

            assert!(pred.test(&5));
            assert!(cloned.test(&5));
            assert!(!pred.test(&-3));
            assert!(!cloned.test(&-3));
        }

        #[test]
        fn test_clone_preserves_name() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let cloned = pred.clone();

            assert_eq!(pred.name(), Some("is_positive"));
            assert_eq!(cloned.name(), Some("is_positive"));
        }

        #[test]
        fn test_and() {
            let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
            let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.and(&pred2);
            assert!(combined.test(&4));
            assert!(!combined.test(&3));
            assert!(!combined.test(&-4));

            // Original predicates are still available
            assert!(pred1.test(&5));
            assert!(pred2.test(&6));
        }

        #[test]
        fn test_or() {
            let pred1 = ArcPredicate::new(|x: &i32| *x < 0);
            let pred2 = ArcPredicate::new(|x: &i32| *x > 10);

            let combined = pred1.or(&pred2);
            assert!(combined.test(&-5));
            assert!(combined.test(&15));
            assert!(!combined.test(&5));

            // Original predicates are still available
            assert!(pred1.test(&-1));
            assert!(pred2.test(&11));
        }

        #[test]
        fn test_not() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0);
            let negated = pred.not();

            assert!(!negated.test(&5));
            assert!(negated.test(&-3));
            assert!(negated.test(&0));

            // Original predicates are still available
            assert!(pred.test(&5));
        }

        #[test]
        fn test_xor() {
            let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
            let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.xor(&pred2);
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
            assert!(!combined.test(&4)); // Positive and even
            assert!(!combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_nand() {
            let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
            let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.nand(&pred2);
            assert!(!combined.test(&4)); // Positive and even
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
        }

        #[test]
        fn test_nor() {
            let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
            let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.nor(&pred2);
            assert!(!combined.test(&4)); // Positive or even
            assert!(!combined.test(&3)); // Positive
            assert!(!combined.test(&-4)); // Even
            assert!(combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_chain_combination() {
            let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
            let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);
            let pred3 = ArcPredicate::new(|x: &i32| *x < 100);

            let combined = pred1.and(&pred2).or(&pred3);
            assert!(combined.test(&4)); // Positive even
            assert!(combined.test(&-5)); // Less than 100
            assert!(combined.test(&50)); // Both conditions satisfied
            assert!(!combined.test(&101)); // Neither condition satisfied

            // Original predicates are still available
            assert!(pred1.test(&1));
            assert!(pred2.test(&2));
            assert!(pred3.test(&99));
        }

        #[test]
        fn test_thread_safety() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0);
            let pred_clone = pred.clone();

            let handle = thread::spawn(move || pred_clone.test(&5));

            assert!(handle.join().unwrap());
            // Original predicates are still available
            assert!(pred.test(&10));
        }

        #[test]
        fn test_multiple_threads() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0);

            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let pred_clone = pred.clone();
                    thread::spawn(move || pred_clone.test(&i))
                })
                .collect();

            let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

            assert_eq!(results[0], false); // 0
            assert_eq!(results[1], true); // 1
            assert_eq!(results[5], true); // 5
            assert_eq!(results[9], true); // 9
        }

        #[test]
        fn test_display() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            assert_eq!(format!("{}", pred), "ArcPredicate(is_positive)");

            let unnamed = ArcPredicate::new(|x: &i32| *x > 0);
            assert_eq!(format!("{}", unnamed), "ArcPredicate(unnamed)");
        }

        #[test]
        fn test_debug() {
            let pred = ArcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let debug_str = format!("{:?}", pred);
            assert!(debug_str.contains("ArcPredicate"));
            assert!(debug_str.contains("is_positive"));

            let unnamed = ArcPredicate::new(|x: &i32| *x > 0);
            let debug_str = format!("{:?}", unnamed);
            assert!(debug_str.contains("ArcPredicate"));
            assert!(debug_str.contains("None"));
        }

        #[test]
        fn test_with_different_types() {
            // String
            let string_pred = ArcPredicate::new(|s: &String| s.len() > 5);
            assert!(string_pred.test(&String::from("hello world")));
            assert!(!string_pred.test(&String::from("hi")));

            // f64
            let float_pred = ArcPredicate::new(|x: &f64| *x > 0.0);
            assert!(float_pred.test(&1.5));
            assert!(!float_pred.test(&-1.5));

            // bool
            let bool_pred = ArcPredicate::new(|x: &bool| *x);
            assert!(bool_pred.test(&true));
            assert!(!bool_pred.test(&false));

            // Vec
            let vec_pred = ArcPredicate::new(|v: &Vec<i32>| !v.is_empty());
            assert!(vec_pred.test(&vec![1, 2, 3]));
            assert!(!vec_pred.test(&vec![]));
        }
    }

    // ========================================================================
    // RcPredicate Tests
    // ========================================================================

    mod rc_predicate_tests {
        use super::*;

        #[test]
        fn test_new() {
            let pred = RcPredicate::new(|x: &i32| *x > 0);
            assert!(pred.test(&5));
            assert!(!pred.test(&-3));
        }

        #[test]
        fn test_with_name() {
            let pred = RcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            assert_eq!(pred.name(), Some("is_positive"));
        }

        #[test]
        fn test_name_none() {
            let pred = RcPredicate::new(|x: &i32| *x > 0);
            assert_eq!(pred.name(), None);
        }

        #[test]
        fn test_test_method() {
            let pred = RcPredicate::new(|x: &i32| *x > 0);
            assert!(pred.test(&1));
            assert!(pred.test(&100));
            assert!(!pred.test(&0));
            assert!(!pred.test(&-1));
        }

        #[test]
        fn test_clone() {
            let pred = RcPredicate::new(|x: &i32| *x > 0);
            let cloned = pred.clone();

            assert!(pred.test(&5));
            assert!(cloned.test(&5));
            assert!(!pred.test(&-3));
            assert!(!cloned.test(&-3));
        }

        #[test]
        fn test_clone_preserves_name() {
            let pred = RcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let cloned = pred.clone();

            assert_eq!(pred.name(), Some("is_positive"));
            assert_eq!(cloned.name(), Some("is_positive"));
        }

        #[test]
        fn test_and() {
            let pred1 = RcPredicate::new(|x: &i32| *x > 0);
            let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.and(&pred2);
            assert!(combined.test(&4));
            assert!(!combined.test(&3));
            assert!(!combined.test(&-4));

            // Original predicates are still available
            assert!(pred1.test(&5));
            assert!(pred2.test(&6));
        }

        #[test]
        fn test_or() {
            let pred1 = RcPredicate::new(|x: &i32| *x < 0);
            let pred2 = RcPredicate::new(|x: &i32| *x > 10);

            let combined = pred1.or(&pred2);
            assert!(combined.test(&-5));
            assert!(combined.test(&15));
            assert!(!combined.test(&5));

            // Original predicates are still available
            assert!(pred1.test(&-1));
            assert!(pred2.test(&11));
        }

        #[test]
        fn test_not() {
            let pred = RcPredicate::new(|x: &i32| *x > 0);
            let negated = pred.not();

            assert!(!negated.test(&5));
            assert!(negated.test(&-3));
            assert!(negated.test(&0));

            // Original predicates are still available
            assert!(pred.test(&5));
        }

        #[test]
        fn test_xor() {
            let pred1 = RcPredicate::new(|x: &i32| *x > 0);
            let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.xor(&pred2);
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
            assert!(!combined.test(&4)); // Positive and even
            assert!(!combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_nand() {
            let pred1 = RcPredicate::new(|x: &i32| *x > 0);
            let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.nand(&pred2);
            assert!(!combined.test(&4)); // Positive and even
            assert!(combined.test(&3)); // Positive but odd
            assert!(combined.test(&-4)); // Even but negative
        }

        #[test]
        fn test_nor() {
            let pred1 = RcPredicate::new(|x: &i32| *x > 0);
            let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

            let combined = pred1.nor(&pred2);
            assert!(!combined.test(&4)); // Positive or even
            assert!(!combined.test(&3)); // Positive
            assert!(!combined.test(&-4)); // Even
            assert!(combined.test(&-3)); // Negative and odd
        }

        #[test]
        fn test_chain_combination() {
            let pred1 = RcPredicate::new(|x: &i32| *x > 0);
            let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);
            let pred3 = RcPredicate::new(|x: &i32| *x < 100);

            let combined = pred1.and(&pred2).or(&pred3);
            assert!(combined.test(&4)); // Positive even
            assert!(combined.test(&-5)); // Less than 100
            assert!(combined.test(&50)); // Both conditions satisfied
            assert!(!combined.test(&101)); // Neither condition satisfied

            // Original predicates are still available
            assert!(pred1.test(&1));
            assert!(pred2.test(&2));
            assert!(pred3.test(&99));
        }

        #[test]
        fn test_display() {
            let pred = RcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            assert_eq!(format!("{}", pred), "RcPredicate(is_positive)");

            let unnamed = RcPredicate::new(|x: &i32| *x > 0);
            assert_eq!(format!("{}", unnamed), "RcPredicate(unnamed)");
        }

        #[test]
        fn test_debug() {
            let pred = RcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let debug_str = format!("{:?}", pred);
            assert!(debug_str.contains("RcPredicate"));
            assert!(debug_str.contains("is_positive"));

            let unnamed = RcPredicate::new(|x: &i32| *x > 0);
            let debug_str = format!("{:?}", unnamed);
            assert!(debug_str.contains("RcPredicate"));
            assert!(debug_str.contains("None"));
        }

        #[test]
        fn test_with_different_types() {
            // String
            let string_pred = RcPredicate::new(|s: &String| s.len() > 5);
            assert!(string_pred.test(&String::from("hello world")));
            assert!(!string_pred.test(&String::from("hi")));

            // f64
            let float_pred = RcPredicate::new(|x: &f64| *x > 0.0);
            assert!(float_pred.test(&1.5));
            assert!(!float_pred.test(&-1.5));

            // bool
            let bool_pred = RcPredicate::new(|x: &bool| *x);
            assert!(bool_pred.test(&true));
            assert!(!bool_pred.test(&false));

            // Vec
            let vec_pred = RcPredicate::new(|v: &Vec<i32>| !v.is_empty());
            assert!(vec_pred.test(&vec![1, 2, 3]));
            assert!(!vec_pred.test(&vec![]));
        }
    }

    // ========================================================================
    // Mixed Type Combination Tests
    // ========================================================================

    mod mixed_type_combination_tests {
        use super::*;

        #[test]
        fn test_box_with_closure() {
            let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
            let combined = box_pred.and(|x: &i32| x % 2 == 0);

            assert!(combined.test(&4));
            assert!(!combined.test(&3));
        }

        #[test]
        fn test_closure_to_box() {
            let closure = |x: &i32| *x > 0;
            let combined = closure.and(|x: &i32| x % 2 == 0);

            // combined is BoxPredicate
            assert!(combined.test(&4));
            assert!(!combined.test(&3));
        }

        #[test]
        fn test_arc_preserves_original() {
            let pred1 = ArcPredicate::new(|x: &i32| *x > 0);
            let pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

            let _combined = pred1.and(&pred2);

            // Original predicates should still be available
            assert!(pred1.test(&5));
            assert!(pred2.test(&6));
        }

        #[test]
        fn test_rc_preserves_original() {
            let pred1 = RcPredicate::new(|x: &i32| *x > 0);
            let pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

            let _combined = pred1.and(&pred2);

            // Original predicates should still be available
            assert!(pred1.test(&5));
            assert!(pred2.test(&6));
        }
    }

    // ========================================================================
    // Edge Case and Special Condition Tests
    // ========================================================================

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_always_true() {
            let always_true = BoxPredicate::new(|_: &i32| true);
            assert!(always_true.test(&0));
            assert!(always_true.test(&-1));
            assert!(always_true.test(&1));
        }

        #[test]
        fn test_always_false() {
            let always_false = BoxPredicate::new(|_: &i32| false);
            assert!(!always_false.test(&0));
            assert!(!always_false.test(&-1));
            assert!(!always_false.test(&1));
        }

        #[test]
        fn test_double_negation() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0);
            let double_negated = pred.not().not();

            assert!(double_negated.test(&5));
            assert!(!double_negated.test(&-3));
        }

        #[test]
        fn test_complex_chain() {
            let pred = BoxPredicate::new(|x: &i32| *x > 0)
                .and(|x: &i32| x % 2 == 0)
                .or(|x: &i32| *x < -10)
                .not();

            assert!(!pred.test(&4)); // Positive even
            assert!(!pred.test(&-15)); // Less than -10
            assert!(pred.test(&3)); // Positive odd
            assert!(pred.test(&-5)); // Between -10 and 0
        }

        #[test]
        fn test_with_zero() {
            let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
            let is_negative = BoxPredicate::new(|x: &i32| *x < 0);

            assert!(!is_positive.test(&0));
            assert!(!is_negative.test(&0));
        }

        #[test]
        fn test_with_large_numbers() {
            let pred = BoxPredicate::new(|x: &i64| *x > 1_000_000_000);
            assert!(pred.test(&2_000_000_000));
            assert!(!pred.test(&500_000_000));
        }

        #[test]
        fn test_with_floating_point() {
            let pred = BoxPredicate::new(|x: &f64| (*x - 1.0).abs() < 0.0001);
            assert!(pred.test(&1.00001));
            assert!(pred.test(&0.99999));
            assert!(!pred.test(&1.1));
        }

        #[test]
        fn test_with_empty_string() {
            let pred = BoxPredicate::new(|s: &String| !s.is_empty());
            assert!(pred.test(&String::from("hello")));
            assert!(!pred.test(&String::new()));
        }

        #[test]
        fn test_with_empty_vec() {
            let pred = BoxPredicate::new(|v: &Vec<i32>| v.len() > 2);
            assert!(pred.test(&vec![1, 2, 3]));
            assert!(!pred.test(&vec![1, 2]));
            assert!(!pred.test(&vec![]));
        }
    }

    // ========================================================================
    // Generic Constraint Tests - Test generic constraints accepting various Predicate types
    // ========================================================================

    mod generic_constraint_tests {
        use super::*;

        /// Generic filter function using Predicate trait as constraint
        fn filter_numbers<P>(numbers: &[i32], predicate: P) -> Vec<i32>
        where
            P: Predicate<i32>,
        {
            numbers
                .iter()
                .copied()
                .filter(|x| predicate.test(x))
                .collect()
        }

        /// Generic count function
        fn count_matching<T, P>(items: &[T], predicate: P) -> usize
        where
            P: Predicate<T>,
        {
            items.iter().filter(|item| predicate.test(item)).count()
        }

        /// Generic find function
        fn find_first<T, P>(items: &[T], predicate: P) -> Option<&T>
        where
            P: Predicate<T>,
        {
            items.iter().find(|item| predicate.test(item))
        }

        #[test]
        fn test_generic_function_accepts_closure() {
            let numbers = vec![-5, -2, 0, 3, 4, 7, 10];

            // Using closure
            let positive = filter_numbers(&numbers, |x: &i32| *x > 0);
            assert_eq!(positive, vec![3, 4, 7, 10]);

            let even = filter_numbers(&numbers, |x: &i32| x % 2 == 0);
            assert_eq!(even, vec![-2, 0, 4, 10]);
        }

        #[test]
        fn test_generic_function_accepts_function_pointer() {
            fn is_positive(x: &i32) -> bool {
                *x > 0
            }

            fn is_even(x: &i32) -> bool {
                x % 2 == 0
            }

            let numbers = vec![-5, -2, 0, 3, 4, 7, 10];

            let positive = filter_numbers(&numbers, is_positive);
            assert_eq!(positive, vec![3, 4, 7, 10]);

            let even = filter_numbers(&numbers, is_even);
            assert_eq!(even, vec![-2, 0, 4, 10]);
        }

        #[test]
        fn test_generic_function_accepts_box_predicate() {
            let numbers = vec![-5, -2, 0, 3, 4, 7, 10];

            let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
            let positive = filter_numbers(&numbers, is_positive);
            assert_eq!(positive, vec![3, 4, 7, 10]);

            let is_large = BoxPredicate::new(|x: &i32| *x > 5);
            let large = filter_numbers(&numbers, is_large);
            assert_eq!(large, vec![7, 10]);
        }

        #[test]
        fn test_generic_function_accepts_arc_predicate() {
            let numbers = vec![-5, -2, 0, 3, 4, 7, 10];

            let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
            let positive = filter_numbers(&numbers, is_positive.clone());
            assert_eq!(positive, vec![3, 4, 7, 10]);

            // ArcPredicate can be reused
            let positive_again = filter_numbers(&numbers, is_positive.clone());
            assert_eq!(positive_again, vec![3, 4, 7, 10]);
        }

        #[test]
        fn test_generic_function_accepts_rc_predicate() {
            let numbers = vec![-5, -2, 0, 3, 4, 7, 10];

            let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);
            let even = filter_numbers(&numbers, is_even.clone());
            assert_eq!(even, vec![-2, 0, 4, 10]);

            // RcPredicate can be reused
            let even_again = filter_numbers(&numbers, is_even.clone());
            assert_eq!(even_again, vec![-2, 0, 4, 10]);
        }

        #[test]
        fn test_generic_count_with_different_predicate_types() {
            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

            // Closure
            assert_eq!(count_matching(&numbers, |x: &i32| *x > 5), 5);

            // BoxPredicate
            let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
            assert_eq!(count_matching(&numbers, is_even), 5);

            // ArcPredicate
            let is_large = ArcPredicate::new(|x: &i32| *x > 7);
            assert_eq!(count_matching(&numbers, is_large.clone()), 3);

            // RcPredicate
            let is_small = RcPredicate::new(|x: &i32| *x < 4);
            assert_eq!(count_matching(&numbers, is_small.clone()), 3);
        }

        #[test]
        fn test_generic_find_with_different_predicate_types() {
            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

            // Closure
            assert_eq!(find_first(&numbers, |x: &i32| *x > 5), Some(&6));

            // BoxPredicate
            let is_even_large = BoxPredicate::new(|x: &i32| x % 2 == 0 && *x > 5);
            assert_eq!(find_first(&numbers, is_even_large), Some(&6));

            // ArcPredicate
            let is_odd = ArcPredicate::new(|x: &i32| x % 2 == 1);
            assert_eq!(find_first(&numbers, is_odd.clone()), Some(&1));

            // RcPredicate
            let is_nine = RcPredicate::new(|x: &i32| *x == 9);
            assert_eq!(find_first(&numbers, is_nine.clone()), Some(&9));
        }

        #[test]
        fn test_generic_with_custom_types() {
            #[derive(Debug, PartialEq)]
            struct Person {
                name: String,
                age: u32,
            }

            let people = vec![
                Person {
                    name: "Alice".to_string(),
                    age: 25,
                },
                Person {
                    name: "Bob".to_string(),
                    age: 17,
                },
                Person {
                    name: "Charlie".to_string(),
                    age: 30,
                },
            ];

            // Use closure
            assert_eq!(count_matching(&people, |p: &Person| p.age >= 18), 2);

            // Use BoxPredicate
            let is_adult = BoxPredicate::new(|p: &Person| p.age >= 18);
            assert_eq!(count_matching(&people, is_adult), 2);

            // Use ArcPredicate
            let is_young = ArcPredicate::new(|p: &Person| p.age < 30);
            assert_eq!(count_matching(&people, is_young.clone()), 2);

            // Use RcPredicate
            let name_starts_with_a = RcPredicate::new(|p: &Person| p.name.starts_with('A'));
            let alice = find_first(&people, name_starts_with_a.clone());
            assert_eq!(alice.map(|p| p.name.as_str()), Some("Alice"));
        }

        #[test]
        fn test_generic_with_combined_predicates() {
            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

            // Use BoxPredicate composition
            let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
            let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);
            let combined = is_positive.and(is_even);
            assert_eq!(count_matching(&numbers, combined), 5);

            // Use ArcPredicate composition
            let is_large = ArcPredicate::new(|x: &i32| *x > 5);
            let is_odd = ArcPredicate::new(|x: &i32| x % 2 == 1);
            let combined = is_large.and(&is_odd);
            assert_eq!(count_matching(&numbers, combined.clone()), 2); // 7, 9
        }

        #[test]
        fn test_mixed_predicate_types_in_sequence() {
            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

            // First filter with closure
            let positive = filter_numbers(&numbers, |x: &i32| *x > 0);

            // Then filter with BoxPredicate
            let even_box = BoxPredicate::new(|x: &i32| x % 2 == 0);
            let even = filter_numbers(&positive, even_box);

            // Finally filter with ArcPredicate
            let large_arc = ArcPredicate::new(|x: &i32| *x > 5);
            let large = filter_numbers(&even, large_arc.clone());

            assert_eq!(large, vec![6, 8, 10]);
        }

        #[test]
        fn test_thread_safety_with_arc_predicate() {
            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

            let pred_clone = is_even.clone();
            let numbers_clone = numbers.clone();
            let handle = thread::spawn(move || count_matching(&numbers_clone, pred_clone));

            let result = handle.join().unwrap();
            assert_eq!(result, 5);

            // Original predicate is still available
            assert_eq!(count_matching(&numbers, is_even.clone()), 5);
        }

        #[test]
        fn test_generic_function_with_string_predicates() {
            let words = vec![
                "hello".to_string(),
                "world".to_string(),
                "rust".to_string(),
                "programming".to_string(),
            ];

            // Closure
            assert_eq!(
                count_matching(&words, |s: &String| s.len() > 5),
                1 // Only "programming" has length > 5
            );

            // BoxPredicate
            let starts_with_r = BoxPredicate::new(|s: &String| s.starts_with('r'));
            assert_eq!(count_matching(&words, starts_with_r), 1);

            // ArcPredicate
            let is_long = ArcPredicate::new(|s: &String| s.len() > 7);
            assert_eq!(count_matching(&words, is_long.clone()), 1); // "programming"

            // RcPredicate
            let contains_o = RcPredicate::new(|s: &String| s.contains('o'));
            assert_eq!(count_matching(&words, contains_o.clone()), 3); // hello, world, programming
        }

        #[test]
        fn test_generic_with_option_predicates() {
            let options = vec![None, Some(5), None, Some(10), Some(3)];

            // Closure
            assert_eq!(
                count_matching(&options, |opt: &Option<i32>| opt.is_some()),
                3
            );

            // BoxPredicate
            let is_some_large =
                BoxPredicate::new(|opt: &Option<i32>| matches!(opt, Some(x) if *x > 5));
            assert_eq!(count_matching(&options, is_some_large), 1);

            // ArcPredicate
            let is_none = ArcPredicate::new(|opt: &Option<i32>| opt.is_none());
            assert_eq!(count_matching(&options, is_none.clone()), 2);
        }

        #[test]
        fn test_predicate_as_struct_field() {
            struct Filter<P> {
                predicate: P,
            }

            impl<P> Filter<P> {
                fn new(predicate: P) -> Self {
                    Self { predicate }
                }

                fn apply<'a, T>(&self, items: &'a [T]) -> Vec<&'a T>
                where
                    P: Predicate<T>,
                {
                    items
                        .iter()
                        .filter(|item| self.predicate.test(item))
                        .collect()
                }
            }

            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

            // Use BoxPredicate
            let filter = Filter::new(BoxPredicate::new(|x: &i32| *x > 5));
            let result: Vec<i32> = filter.apply(&numbers).into_iter().copied().collect();
            assert_eq!(result, vec![6, 7, 8, 9, 10]);

            // Use ArcPredicate
            let filter = Filter::new(ArcPredicate::new(|x: &i32| x % 2 == 0));
            let result: Vec<i32> = filter.apply(&numbers).into_iter().copied().collect();
            assert_eq!(result, vec![2, 4, 6, 8, 10]);
        }

        #[test]
        fn test_returning_predicate_from_function() {
            fn create_range_predicate(min: i32, max: i32) -> BoxPredicate<i32> {
                BoxPredicate::new(move |x: &i32| *x >= min && *x <= max)
            }

            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            let in_range = create_range_predicate(3, 7);
            let result = filter_numbers(&numbers, in_range);
            assert_eq!(result, vec![3, 4, 5, 6, 7]);
        }

        #[test]
        fn test_predicate_with_borrowed_data() {
            let threshold = 5;
            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

            // Closure captures external variable
            let greater_than_threshold = |x: &i32| *x > threshold;
            assert_eq!(count_matching(&numbers, greater_than_threshold), 5);

            // BoxPredicate captures external variable
            let multiplier = 2;
            let pred = BoxPredicate::new(move |x: &i32| *x > threshold * multiplier);
            assert_eq!(count_matching(&numbers, pred), 0); // None greater than 10
        }
    }

    // ========================================================================
    // Conversion Tests - Test into_box, into_rc, into_arc conversion methods
    // ========================================================================

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_closure_into_box() {
            let closure = |x: &i32| *x > 0;
            let box_pred: BoxPredicate<i32> = closure.into_box();
            assert!(box_pred.test(&5));
            assert!(!box_pred.test(&-3));
        }

        #[test]
        fn test_closure_into_rc() {
            let closure = |x: &i32| *x > 0;
            let rc_pred: RcPredicate<i32> = closure.into_rc();
            assert!(rc_pred.test(&5));
            assert!(!rc_pred.test(&-3));
        }

        #[test]
        fn test_closure_into_arc() {
            let closure = |x: &i32| *x > 0;
            let arc_pred: ArcPredicate<i32> = closure.into_arc();
            assert!(arc_pred.test(&5));
            assert!(!arc_pred.test(&-3));
        }

        #[test]
        fn test_box_to_box_zero_cost() {
            let box_pred1 = BoxPredicate::new(|x: &i32| *x > 0);
            let box_pred2 = box_pred1.into_box(); // Zero-cost
            assert!(box_pred2.test(&5));
        }

        #[test]
        fn test_box_to_rc() {
            let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
            let rc_pred = box_pred.into_rc();
            assert!(rc_pred.test(&5));
        }

        #[test]
        fn test_box_to_rc_preserves_name() {
            let box_pred = BoxPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let rc_pred = box_pred.into_rc();
            assert_eq!(rc_pred.name(), Some("is_positive"));
            assert!(rc_pred.test(&5));
        }

        // Note: BoxPredicate::into_arc() method exists but is almost unusable
        //
        // Since BoxPredicate internally uses `Box<dyn Fn(&T) -> bool>` (without Send + Sync bounds),
        // instances created through BoxPredicate::new() do not satisfy the `Self: Send + Sync` constraint required by into_arc().
        //
        // If you need to create a thread-safe predicate, you should:
        // 1. Call .into_arc() directly from a closure -> ArcPredicate
        // 2. Or use ArcPredicate::new() directly
        //
        // The tests below demonstrate the correct approach:

        #[test]
        fn test_closure_to_arc_instead_of_box_to_arc() {
            // Correct approach: Convert from closure to ArcPredicate directly
            let closure = |x: &i32| *x > 0;
            let arc_pred = closure.into_arc();

            assert!(arc_pred.test(&5));
            assert!(!arc_pred.test(&-3));

            // Can be used across threads
            let arc_clone = arc_pred.clone();
            let handle = thread::spawn(move || arc_clone.test(&10));
            assert!(handle.join().unwrap());
        }

        #[test]
        fn test_arc_predicate_new_instead_of_box() {
            // Another correct approach: Use ArcPredicate::new() directly
            let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);

            assert!(arc_pred.test(&5));

            // Can be used across threads
            let arc_clone = arc_pred.clone();
            let handle = thread::spawn(move || arc_clone.test(&10));
            assert!(handle.join().unwrap());
        }

        #[test]
        fn test_arc_to_arc_zero_cost() {
            let arc_pred1 = ArcPredicate::new(|x: &i32| *x > 0);
            let arc_pred2 = arc_pred1.into_arc(); // Zero-cost
            assert!(arc_pred2.test(&5));
        }

        #[test]
        fn test_arc_to_box() {
            let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
            let box_pred = arc_pred.into_box();
            assert!(box_pred.test(&5));
        }

        #[test]
        fn test_arc_to_box_preserves_name() {
            let arc_pred = ArcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let box_pred = arc_pred.into_box();
            assert_eq!(box_pred.name(), Some("is_positive"));
            assert!(box_pred.test(&5));
        }

        #[test]
        fn test_arc_to_rc() {
            let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
            let rc_pred = arc_pred.into_rc();
            assert!(rc_pred.test(&5));
        }

        #[test]
        fn test_arc_to_rc_preserves_name() {
            let arc_pred = ArcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let rc_pred = arc_pred.into_rc();
            assert_eq!(rc_pred.name(), Some("is_positive"));
            assert!(rc_pred.test(&5));
        }

        #[test]
        fn test_rc_to_rc_zero_cost() {
            let rc_pred1 = RcPredicate::new(|x: &i32| *x > 0);
            let rc_pred2 = rc_pred1.into_rc(); // Zero-cost
            assert!(rc_pred2.test(&5));
        }

        #[test]
        fn test_rc_to_box() {
            let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
            let box_pred = rc_pred.into_box();
            assert!(box_pred.test(&5));
        }

        #[test]
        fn test_rc_to_box_preserves_name() {
            let rc_pred = RcPredicate::new(|x: &i32| *x > 0).with_name("is_positive");
            let box_pred = rc_pred.into_box();
            assert_eq!(box_pred.name(), Some("is_positive"));
            assert!(box_pred.test(&5));
        }

        #[test]
        fn test_struct_storing_box_predicate() {
            struct MyFilter {
                predicate: BoxPredicate<i32>,
            }

            impl MyFilter {
                fn new<P>(predicate: P) -> Self
                where
                    P: Predicate<i32> + 'static,
                {
                    Self {
                        predicate: predicate.into_box(),
                    }
                }

                fn test(&self, value: &i32) -> bool {
                    self.predicate.test(value)
                }
            }

            // Create with closure
            let filter1 = MyFilter::new(|x: &i32| *x > 0);
            assert!(filter1.test(&5));
            assert!(!filter1.test(&-3));

            // Create with BoxPredicate (zero-cost)
            let filter2 = MyFilter::new(BoxPredicate::new(|x: &i32| x % 2 == 0));
            assert!(filter2.test(&4));
            assert!(!filter2.test(&3));

            // Create with ArcPredicate
            let filter3 = MyFilter::new(ArcPredicate::new(|x: &i32| *x < 10));
            assert!(filter3.test(&5));
            assert!(!filter3.test(&15));

            // Create with RcPredicate
            let filter4 = MyFilter::new(RcPredicate::new(|x: &i32| *x != 0));
            assert!(filter4.test(&5));
            assert!(!filter4.test(&0));
        }

        #[test]
        fn test_struct_storing_arc_predicate() {
            struct ThreadSafeFilter {
                predicate: ArcPredicate<i32>,
            }

            impl ThreadSafeFilter {
                fn new<P>(predicate: P) -> Self
                where
                    P: Predicate<i32> + Send + Sync + 'static,
                {
                    Self {
                        predicate: predicate.into_arc(),
                    }
                }

                fn test(&self, value: &i32) -> bool {
                    self.predicate.test(value)
                }
            }

            // Create with closure
            let filter1 = ThreadSafeFilter::new(|x: &i32| *x > 0);
            assert!(filter1.test(&5));

            // Create with ArcPredicate (zero-cost)
            let filter2 = ThreadSafeFilter::new(ArcPredicate::new(|x: &i32| x % 2 == 0));
            assert!(filter2.test(&4));

            // Note: Cannot create with RcPredicate because Rc is not Send + Sync
            // let filter3 = ThreadSafeFilter::new(RcPredicate::new(|x| *x > 0)); // Compilation error
        }

        #[test]
        fn test_conversion_chain() {
            // Test conversion chain
            let closure = |x: &i32| *x > 5;

            // Closure -> BoxPredicate -> test
            let box_pred = closure.into_box();
            assert!(box_pred.test(&10));

            // Closure -> RcPredicate -> BoxPredicate -> test
            let closure2 = |x: &i32| *x < 5;
            let rc_pred = closure2.into_rc();
            let box_pred2 = rc_pred.into_box();
            assert!(box_pred2.test(&3));

            // Closure -> ArcPredicate -> BoxPredicate -> test
            let closure3 = |x: &i32| x % 2 == 0;
            let arc_pred = closure3.into_arc();
            let box_pred3 = arc_pred.into_box();
            assert!(box_pred3.test(&4));
        }

        #[test]
        fn test_conversion_preserves_behavior() {
            let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

            // Original closure
            let closure = |x: &i32| *x > 5;
            let count1 = numbers.iter().filter(|x| closure.test(x)).count();

            // Convert to BoxPredicate
            let box_pred = (|x: &i32| *x > 5).into_box();
            let count2 = numbers.iter().filter(|x| box_pred.test(x)).count();

            // Convert to RcPredicate
            let rc_pred = (|x: &i32| *x > 5).into_rc();
            let count3 = numbers.iter().filter(|x| rc_pred.test(x)).count();

            // Convert to ArcPredicate
            let arc_pred = (|x: &i32| *x > 5).into_arc();
            let count4 = numbers.iter().filter(|x| arc_pred.test(x)).count();

            // All conversions should maintain the same behavior
            assert_eq!(count1, 5);
            assert_eq!(count2, 5);
            assert_eq!(count3, 5);
            assert_eq!(count4, 5);
        }
    }

    // ========================================================================
    // into_fn() Tests - Test converting predicates to closures for iterators
    // ========================================================================

    mod into_fn_tests {
        use super::*;

        #[test]
        fn test_box_predicate_into_fn_with_filter() {
            let predicate = BoxPredicate::new(|x: &i32| *x > 0);
            let values = vec![1, -2, 3, -4, 5];

            let result: Vec<i32> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(result, vec![1, 3, 5]);
        }

        #[test]
        fn test_arc_predicate_into_fn_with_filter() {
            let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
            let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
            let predicate = is_positive.and(&is_even);

            let values = vec![1, 2, 3, 4, 5, 6];
            let result: Vec<i32> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(result, vec![2, 4, 6]);
        }

        #[test]
        fn test_rc_predicate_into_fn_with_filter() {
            let predicate = RcPredicate::new(|x: &i32| *x > 0);
            let values = vec![1, -2, 3, -4, 5];

            let result: Vec<i32> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(result, vec![1, 3, 5]);
        }

        #[test]
        fn test_closure_into_fn_with_filter() {
            let predicate = |x: &i32| *x > 0;
            let values = vec![1, -2, 3, -4, 5];

            let result: Vec<i32> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(result, vec![1, 3, 5]);
        }

        #[test]
        fn test_into_fn_with_take_while() {
            let predicate = RcPredicate::new(|x: &i32| *x > 0);
            let values = vec![1, 2, -3, 4, -5, 6];

            let result: Vec<i32> = values
                .iter()
                .copied()
                .take_while(predicate.into_fn())
                .collect();

            assert_eq!(result, vec![1, 2]);
        }

        #[test]
        fn test_into_fn_with_skip_while() {
            let predicate = BoxPredicate::new(|x: &i32| *x < 0);
            let values = vec![-1, -2, 3, 4, -5, 6];

            let result: Vec<i32> = values
                .iter()
                .copied()
                .skip_while(predicate.into_fn())
                .collect();

            assert_eq!(result, vec![3, 4, -5, 6]);
        }

        #[test]
        fn test_into_fn_with_partition() {
            let predicate = ArcPredicate::new(|x: &i32| *x > 0);
            let values = vec![1, -2, 3, -4, 5];

            let (positives, negatives): (Vec<i32>, Vec<i32>) =
                values.into_iter().partition(predicate.into_fn());

            assert_eq!(positives, vec![1, 3, 5]);
            assert_eq!(negatives, vec![-2, -4]);
        }

        #[test]
        fn test_into_fn_with_complex_composition() {
            let predicate = BoxPredicate::new(|x: &i32| *x > 0)
                .and(|x: &i32| x % 2 == 0)
                .or(|x: &i32| *x > 100);

            let values = vec![1, 2, 3, 4, -5, 150];
            let result: Vec<i32> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(result, vec![2, 4, 150]);
        }

        #[test]
        fn test_into_fn_with_string() {
            let predicate = BoxPredicate::new(|s: &String| s.contains('a'));
            let values = vec![
                "apple".to_string(),
                "banana".to_string(),
                "cherry".to_string(),
                "date".to_string(),
            ];

            let result: Vec<String> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(
                result,
                vec![
                    "apple".to_string(),
                    "banana".to_string(),
                    "date".to_string()
                ]
            );
        }

        #[test]
        fn test_into_fn_preserves_closure_semantics() {
            // Test that the converted closure works correctly
            let threshold = 5;
            let predicate = BoxPredicate::new(move |x: &i32| *x > threshold);

            let values = vec![1, 5, 6, 10];
            let result: Vec<i32> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(result, vec![6, 10]);
        }

        #[test]
        fn test_into_fn_with_references() {
            // Test with owned values in iterator
            let predicate = RcPredicate::new(|x: &i32| *x > 0);
            let values = vec![1, -2, 3, -4, 5];

            // Use into_iter() to get owned values
            let result: Vec<i32> = values.into_iter().filter(predicate.into_fn()).collect();

            assert_eq!(result, vec![1, 3, 5]);
        }
    }
}
