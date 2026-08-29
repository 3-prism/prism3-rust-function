// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[cfg(test)]
mod tests {
    use std::thread;

    use qubit_function::predicates::ArcBiPredicate;
    use qubit_function::predicates::BiPredicate;
    use qubit_function::predicates::BoxBiPredicate;
    use qubit_function::predicates::RcBiPredicate;

    // ========================================================================
    // BiPredicate Trait Tests - Test closure and function pointer
    // implementations
    // ========================================================================

    mod bi_predicate_ext_tests {
        use super::BiPredicate;
    }

    // ========================================================================
    // BoxBiPredicate Tests
    // ========================================================================

    mod generic_constraint_tests {
        use super::ArcBiPredicate;
        use super::BiPredicate;
        use super::BoxBiPredicate;
        use super::RcBiPredicate;
        use super::thread;

        fn filter_pairs<P>(pairs: Vec<(i32, i32)>, predicate: &P) -> Vec<(i32, i32)>
        where
            P: BiPredicate<i32, i32>,
        {
            pairs.into_iter().filter(|(x, y)| predicate.test(x, y)).collect()
        }

        #[test]
        fn test_generic_function_accepts_closure() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let result = filter_pairs(pairs, &|x: &i32, y: &i32| x + y > 0);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_box_bi_predicate() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let result = filter_pairs(pairs, &pred);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_arc_bi_predicate() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let result = filter_pairs(pairs, &pred);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_rc_bi_predicate() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let result = filter_pairs(pairs, &pred);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_function_pointer() {
            fn sum_positive(x: &i32, y: &i32) -> bool {
                x + y > 0
            }

            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let result = filter_pairs(pairs, &sum_positive);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_count_with_different_bi_predicate_types() {
            fn count_matching<P>(pairs: &[(i32, i32)], pred: &P) -> usize
            where
                P: BiPredicate<i32, i32>,
            {
                pairs.iter().filter(|(x, y)| pred.test(x, y)).count()
            }

            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];

            let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(count_matching(&pairs, &box_pred), 3);

            let arc_pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(count_matching(&pairs, &arc_pred), 3);

            let rc_pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(count_matching(&pairs, &rc_pred), 3);
        }

        #[test]
        fn test_generic_with_combined_bi_predicates() {
            let x_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let y_positive = ArcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);
            let combined = x_positive.and(y_positive);

            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let result = filter_pairs(pairs.to_vec(), &combined);
            assert_eq!(result, vec![(1, 2), (3, 4)]);
        }

        #[test]
        fn test_generic_with_string_bi_predicates() {
            fn filter_string_pairs<P>(pairs: Vec<(String, usize)>, predicate: &P) -> Vec<(String, usize)>
            where
                P: BiPredicate<String, usize>,
            {
                pairs.into_iter().filter(|(s, len)| predicate.test(s, len)).collect()
            }

            let pairs = vec![
                (String::from("hello"), 3),
                (String::from("hi"), 5),
                (String::from("world"), 4),
            ];

            let pred = BoxBiPredicate::new(|s: &String, len: &usize| s.len() > *len);
            let result = filter_string_pairs(pairs, &pred);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn test_bi_predicate_as_struct_field() {
            struct Validator<P> {
                predicate: P,
            }

            impl<P> Validator<P>
            where
                P: BiPredicate<i32, i32>,
            {
                fn validate(&self, x: i32, y: i32) -> bool {
                    self.predicate.test(&x, &y)
                }
            }

            let validator = Validator {
                predicate: BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0),
            };

            assert!(validator.validate(5, 3));
            assert!(!validator.validate(-5, -3));
        }

        #[test]
        fn test_returning_bi_predicate_from_function() {
            fn create_sum_checker(threshold: i32) -> impl BiPredicate<i32, i32> {
                move |x: &i32, y: &i32| x + y > threshold
            }

            let checker = create_sum_checker(10);
            assert!(checker.test(&6, &5));
            assert!(!checker.test(&3, &4));
        }

        #[test]
        fn test_thread_safety_with_arc_bi_predicate() {
            fn process_in_thread<P>(pred: P, x: i32, y: i32) -> bool
            where
                P: BiPredicate<i32, i32> + Send + 'static,
            {
                thread::spawn(move || pred.test(&x, &y))
                    .join()
                    .expect("thread should not panic")
            }

            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(process_in_thread(pred, 5, 3));
        }

        #[test]
        fn test_mixed_bi_predicate_types_in_sequence() {
            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];

            // Use different types in sequence
            let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let count1 = pairs.iter().filter(|(x, y)| box_pred.test(x, y)).count();

            let arc_pred = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let count2 = pairs.iter().filter(|(x, y)| arc_pred.test(x, y)).count();

            assert_eq!(count1, 3);
            assert_eq!(count2, 3);
        }

        #[test]
        fn test_generic_with_custom_types() {
            #[derive(Debug, Clone, PartialEq)]
            struct Point {
                x: i32,
                y: i32,
            }

            fn filter_points<P>(points: Vec<(Point, Point)>, pred: &P) -> Vec<(Point, Point)>
            where
                P: BiPredicate<Point, Point>,
            {
                points.into_iter().filter(|(p1, p2)| pred.test(p1, p2)).collect()
            }

            let points = vec![
                (Point { x: 1, y: 2 }, Point { x: 3, y: 4 }),
                (Point { x: -1, y: 2 }, Point { x: 1, y: -4 }),
            ];

            let pred = BoxBiPredicate::new(|p1: &Point, p2: &Point| p1.x + p2.x > 0);
            let result = filter_points(points, &pred);
            assert_eq!(result.len(), 1);
        }
    }

    // ========================================================================
    // Default Implementation Tests - Test that custom types can use
    // default implementations of into_xxx methods
    // ========================================================================

    mod default_implementation_tests {
        use super::BiPredicate;

        // Custom bi-predicate type that only implements the core
        // test method and relies on default implementations for
        // all conversion methods
        #[derive(Clone)]
        struct CustomBiPredicate<T, U>
        where
            T: 'static,
            U: 'static,
        {
            threshold: i32,
            _phantom: std::marker::PhantomData<(T, U)>,
        }

        impl CustomBiPredicate<i32, i32> {
            fn new(threshold: i32) -> Self {
                Self {
                    threshold,
                    _phantom: std::marker::PhantomData,
                }
            }
        }

        // Only implement the core test method - all into_xxx and to_xxx
        // methods will use default implementations
        impl BiPredicate<i32, i32> for CustomBiPredicate<i32, i32> {
            fn test(&self, first: &i32, second: &i32) -> bool {
                first + second > self.threshold
            }

            // All other methods (into_box, into_rc, into_arc, into_fn,
            // to_box, to_rc, to_arc, to_fn) use default implementations
            // automatically
        }

        #[test]
        fn test_custom_type_basic_test() {
            let pred = CustomBiPredicate::new(10);
            assert!(pred.test(&6, &5));
            assert!(pred.test(&10, &1));
            assert!(!pred.test(&5, &5));
            assert!(!pred.test(&3, &4));
        }

        #[test]
        fn test_custom_type_can_be_used_in_generic_context() {
            fn accepts_predicate<P>(pred: &P, x: i32, y: i32) -> bool
            where
                P: BiPredicate<i32, i32>,
            {
                pred.test(&x, &y)
            }

            let pred = CustomBiPredicate::new(10);
            assert!(accepts_predicate(&pred, 6, 5));
            assert!(!accepts_predicate(&pred, 3, 4));
        }

        // ========================================================================
        // Test default to_xxx implementations
        // ========================================================================
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    mod edge_case_tests {
        use super::BiPredicate;
        use super::BoxBiPredicate;

        #[test]
        fn test_with_zero() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(!sum_positive.test(&0, &0));
            assert!(sum_positive.test(&1, &0));
            assert!(sum_positive.test(&0, &1));
        }

        #[test]
        fn test_always_true() {
            let always_true = BoxBiPredicate::new(|_x: &i32, _y: &i32| true);
            assert!(always_true.test(&5, &3));
            assert!(always_true.test(&-5, &-3));
            assert!(always_true.test(&0, &0));
        }

        #[test]
        fn test_always_false() {
            let always_false = BoxBiPredicate::new(|_x: &i32, _y: &i32| false);
            assert!(!always_false.test(&5, &3));
            assert!(!always_false.test(&-5, &-3));
            assert!(!always_false.test(&0, &0));
        }

        #[test]
        fn test_double_negation() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let not_not = !(!sum_positive);
            assert!(not_not.test(&5, &3));
            assert!(!not_not.test(&-5, &-3));
        }

        #[test]
        fn test_with_empty_string() {
            let is_empty = BoxBiPredicate::new(|s1: &String, s2: &String| s1.is_empty() && s2.is_empty());
            assert!(is_empty.test(&String::new(), &String::new()));
            assert!(!is_empty.test(&String::from("a"), &String::new()));
        }

        #[test]
        fn test_with_large_numbers() {
            let sum_overflow_safe = BoxBiPredicate::new(|x: &i64, y: &i64| x.checked_add(*y).is_some());
            let max_minus_one = i64::MAX - 1;
            assert!(sum_overflow_safe.test(&max_minus_one, &1));
            assert!(!sum_overflow_safe.test(&i64::MAX, &1));
        }

        #[test]
        fn test_with_floating_point() {
            let close_enough = BoxBiPredicate::new(|x: &f64, y: &f64| (*x - *y).abs() < 0.01);
            assert!(close_enough.test(&1.0, &1.005));
            assert!(!close_enough.test(&1.0, &1.02));
        }

        #[test]
        fn test_complex_chain() {
            let p1 = BoxBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let p2 = BoxBiPredicate::new(|_x: &i32, y: &i32| *y > 0);
            let p3 = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 10);

            let complex = p1.and(p2).or(p3);
            assert!(complex.test(&5, &3)); // Both positive
            assert!(complex.test(&50, &-30)); // Sum > 10 (50 + (-30) = 20)
            assert!(!complex.test(&-5, &3)); // Not both positive, sum not > 10
        }
    }

    // ========================================================================
    // Mixed Type Combination Tests
    // ========================================================================

    mod mixed_type_combination_tests {
        use super::ArcBiPredicate;
        use super::BiPredicate;
        use super::BoxBiPredicate;
        use super::RcBiPredicate;

        #[test]
        fn test_box_with_closure() {
            let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let combined = box_pred.and(|x: &i32, _y: &i32| *x > 0);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &10));
        }

        #[test]
        fn test_arc_preserves_original() {
            let arc1 = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let arc2 = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let _combined = arc1.clone().and(arc2.clone());

            // Originals still usable
            assert!(arc1.test(&-5, &10));
            assert!(arc2.test(&5, &-10));
        }

        #[test]
        fn test_rc_preserves_original() {
            let rc1 = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let rc2 = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let _combined = rc1.clone().and(rc2.clone());

            // Originals still usable
            assert!(rc1.test(&-5, &10));
            assert!(rc2.test(&5, &-10));
        }
    }
}
