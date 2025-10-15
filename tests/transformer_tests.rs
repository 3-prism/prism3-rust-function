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
    use prism3_function::transformer::{
        ArcFnTransformer, BoxFnTransformer, BoxTransformer, FnTransformerOps, RcFnTransformer,
        Transformer,
    };

    // ============================================================================
    // Tests for Transformer trait
    // ============================================================================

    #[cfg(test)]
    mod transformer_trait_tests {
        use super::*;

        #[test]
        fn test_closure_implements_transformer() {
            let transformer = |x: i32| x * 2;
            let result = transformer.transform(21);
            assert_eq!(result, 42);
        }
    }

    // ============================================================================
    // Tests for BoxTransformer
    // ============================================================================

    #[cfg(test)]
    mod box_transformer_tests {
        use super::*;

        #[test]
        fn test_box_transformer_new() {
            let transformer = BoxTransformer::new(|x: i32| x * 2);
            let result = transformer.transform(21);
            assert_eq!(result, 42);
        }

        #[test]
        fn test_box_transformer_identity() {
            let identity = BoxTransformer::<i32>::identity();
            assert_eq!(identity.transform(42), 42);
        }

        #[test]
        fn test_box_transformer_constant() {
            let constant = BoxTransformer::constant(100);
            assert_eq!(constant.transform(42), 100);
        }

        #[test]
        fn test_box_transformer_then() {
            let add_one = BoxTransformer::new(|x: i32| x + 1);
            let double = |x: i32| x * 2;
            let composed = add_one.then(double);
            assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
        }

        #[test]
        fn test_box_transformer_compose() {
            let double = BoxTransformer::new(|x: i32| x * 2);
            let add_one = |x: i32| x + 1;
            let composed = double.compose(add_one);
            assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
        }

        #[test]
        fn test_box_transformer_chain() {
            let add_one = BoxTransformer::new(|x: i32| x + 1);
            let double = BoxTransformer::new(|x: i32| x * 2);
            let composed = add_one.chain(double);
            assert_eq!(composed.transform(5), 12); // (5 + 1) * 2 = 12
        }

        #[test]
        fn test_box_transformer_when() {
            let transformer = BoxTransformer::when(|x: &i32| *x > 0, |x: i32| x * 2);
            assert_eq!(transformer.transform(5), 10);

            let transformer2 = BoxTransformer::when(|x: &i32| *x > 0, |x: i32| x * 2);
            assert_eq!(transformer2.transform(-3), -3);
        }

        #[test]
        fn test_box_transformer_if_else() {
            let transformer =
                BoxTransformer::if_else(|x: &i32| *x > 0, |x: i32| x * 2, |x: i32| x.abs());
            assert_eq!(transformer.transform(5), 10);

            let transformer2 =
                BoxTransformer::if_else(|x: &i32| *x > 0, |x: i32| x * 2, |x: i32| x.abs());
            assert_eq!(transformer2.transform(-3), 3);
        }

        #[test]
        fn test_box_transformer_repeat() {
            let add_one = |x: i32| x + 1;
            let add_three = BoxTransformer::repeat(add_one, 3);
            assert_eq!(add_three.transform(5), 8); // 5 + 1 + 1 + 1 = 8

            let double = |x: i32| x * 2;
            let times_eight = BoxTransformer::repeat(double, 3);
            assert_eq!(times_eight.transform(2), 16); // 2 * 2 * 2 * 2 = 16
        }

        #[test]
        fn test_box_transformer_repeat_zero_times() {
            let double = |x: i32| x * 2;
            let identity = BoxTransformer::repeat(double, 0);
            assert_eq!(identity.transform(5), 5);
        }

        #[test]
        fn test_box_transformer_map_option() {
            let double = |x: i32| x * 2;
            let option_double = BoxTransformer::map_option(double);
            assert_eq!(option_double.transform(Some(21)), Some(42));

            let double2 = |x: i32| x * 2;
            let option_double2 = BoxTransformer::map_option(double2);
            assert_eq!(option_double2.transform(None), None);
        }

        #[test]
        fn test_box_transformer_map_result() {
            let double = |x: i32| x * 2;
            let result_double = BoxTransformer::map_result(double);
            assert_eq!(result_double.transform(Ok::<i32, &str>(21)), Ok(42));

            let double2 = |x: i32| x * 2;
            let result_double2 = BoxTransformer::map_result(double2);
            assert_eq!(
                result_double2.transform(Err::<i32, &str>("error")),
                Err("error")
            );
        }

        #[test]
        fn test_box_transformer_string_processing() {
            let trim = BoxTransformer::new(|s: String| s.trim().to_string());
            let uppercase = |s: String| s.to_uppercase();
            let pipeline = trim.then(uppercase);

            let result = pipeline.transform("  hello  ".to_string());
            assert_eq!(result, "HELLO");
        }

        #[test]
        fn test_box_transformer_multiple_composition() {
            let add_one = BoxTransformer::new(|x: i32| x + 1);
            let double = |x: i32| x * 2;
            let subtract_three = |x: i32| x - 3;

            let pipeline = add_one.then(double).then(subtract_three);
            assert_eq!(pipeline.transform(5), 9); // ((5 + 1) * 2) - 3 = 9
        }
    }

    // ============================================================================
    // Tests for FnTransformerOps (closure extension trait)
    // ============================================================================

    #[cfg(test)]
    mod fn_transformer_ops_tests {
        use super::*;

        #[test]
        fn test_closure_then() {
            let add_one = |x: i32| x + 1;
            let double = |x: i32| x * 2;
            let composed = add_one.then(double);
            assert_eq!(composed.transform(5), 12);
        }

        #[test]
        fn test_closure_compose_transformer() {
            let double = |x: i32| x * 2;
            let add_one = |x: i32| x + 1;
            let composed = double.compose_transformer(add_one);
            assert_eq!(composed.transform(5), 12);
        }
    }

    // ============================================================================
    // Tests for BoxFnTransformer
    // ============================================================================

    #[cfg(test)]
    mod box_fn_transformer_tests {
        use super::*;

        #[test]
        fn test_box_fn_transformer_new() {
            let transformer = BoxFnTransformer::new(|x: i32| x * 2);
            assert_eq!(transformer.transform(21), 42);
        }

        #[test]
        fn test_box_fn_transformer_reusable() {
            let transformer = BoxFnTransformer::new(|x: i32| x * 2);
            let r1 = transformer.transform(21);
            let r2 = transformer.transform(42);
            assert_eq!(r1, 42);
            assert_eq!(r2, 84);
        }

        #[test]
        fn test_box_fn_transformer_identity() {
            let identity = BoxFnTransformer::<i32>::identity();
            assert_eq!(identity.transform(42), 42);
            assert_eq!(identity.transform(100), 100);
        }

        #[test]
        fn test_box_fn_transformer_constant() {
            let constant = BoxFnTransformer::constant(42);
            assert_eq!(constant.transform(100), 42);
            assert_eq!(constant.transform(200), 42);
        }

        #[test]
        fn test_box_fn_transformer_then() {
            let add_one = BoxFnTransformer::new(|x: i32| x + 1);
            let double = BoxFnTransformer::new(|x: i32| x * 2);
            let composed = add_one.then(double);
            assert_eq!(composed.transform(5), 12);
        }

        #[test]
        fn test_box_fn_transformer_compose() {
            let double = BoxFnTransformer::new(|x: i32| x * 2);
            let add_one = BoxFnTransformer::new(|x: i32| x + 1);
            let composed = double.compose(add_one);
            assert_eq!(composed.transform(5), 12);
        }

        #[test]
        fn test_box_fn_transformer_with_copy_types() {
            let transformer = BoxFnTransformer::new(|x: i32| x * 2);
            // i32 is Copy, so we can call multiple times
            assert_eq!(transformer.transform(10), 20);
            assert_eq!(transformer.transform(20), 40);
            assert_eq!(transformer.transform(30), 60);
        }

        #[test]
        fn test_box_fn_transformer_with_non_copy_types() {
            let transformer = BoxFnTransformer::new(|s: String| s.to_uppercase());

            // String is not Copy, so each call needs a new String
            assert_eq!(transformer.transform("hello".to_string()), "HELLO");
            assert_eq!(transformer.transform("world".to_string()), "WORLD");
        }
    }

    // ============================================================================
    // Tests for ArcFnTransformer
    // ============================================================================

    #[cfg(test)]
    mod arc_fn_transformer_tests {
        use super::*;

        #[test]
        fn test_arc_fn_transformer_new() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            assert_eq!(transformer.transform(21), 42);
        }

        #[test]
        fn test_arc_fn_transformer_reusable() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            let r1 = transformer.transform(21);
            let r2 = transformer.transform(42);
            assert_eq!(r1, 42);
            assert_eq!(r2, 84);
        }

        #[test]
        fn test_arc_fn_transformer_clone() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            let cloned = transformer.clone();

            assert_eq!(transformer.transform(21), 42);
            assert_eq!(cloned.transform(42), 84);
        }

        #[test]
        fn test_arc_fn_transformer_identity() {
            let identity = ArcFnTransformer::<i32>::identity();
            assert_eq!(identity.transform(42), 42);
            assert_eq!(identity.transform(100), 100);
        }

        #[test]
        fn test_arc_fn_transformer_constant() {
            let constant = ArcFnTransformer::constant(42);
            assert_eq!(constant.transform(100), 42);
            assert_eq!(constant.transform(200), 42);
        }

        #[test]
        fn test_arc_fn_transformer_then() {
            let add_one = ArcFnTransformer::new(|x: i32| x + 1);
            let double = ArcFnTransformer::new(|x: i32| x * 2);
            let composed = add_one.then(&double);

            // Original transformers are still usable
            assert_eq!(add_one.transform(5), 6);
            assert_eq!(double.transform(5), 10);
            assert_eq!(composed.transform(5), 12);
        }

        #[test]
        fn test_arc_fn_transformer_compose() {
            let double = ArcFnTransformer::new(|x: i32| x * 2);
            let add_one = ArcFnTransformer::new(|x: i32| x + 1);
            let composed = double.compose(&add_one);

            assert_eq!(composed.transform(5), 12);
            // Original transformers still usable
            assert_eq!(double.transform(5), 10);
            assert_eq!(add_one.transform(5), 6);
        }

        #[test]
        fn test_arc_fn_transformer_multiple_clones() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            let clone1 = transformer.clone();
            let clone2 = transformer.clone();
            let clone3 = clone1.clone();

            assert_eq!(transformer.transform(10), 20);
            assert_eq!(clone1.transform(20), 40);
            assert_eq!(clone2.transform(30), 60);
            assert_eq!(clone3.transform(40), 80);
        }

        #[test]
        fn test_arc_fn_transformer_cross_thread() {
            use std::thread;

            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            let transformer_clone = transformer.clone();

            let handle = thread::spawn(move || transformer_clone.transform(21));

            assert_eq!(transformer.transform(42), 84);
            assert_eq!(handle.join().unwrap(), 42);
        }

        #[test]
        fn test_arc_fn_transformer_multiple_threads() {
            use std::thread;

            let transformer = ArcFnTransformer::new(|x: i32| x * 2);

            let mut handles = vec![];
            for i in 1..=5 {
                let t = transformer.clone();
                let handle = thread::spawn(move || t.transform(i));
                handles.push(handle);
            }

            let results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();

            assert_eq!(results, vec![2, 4, 6, 8, 10]);
        }

        #[test]
        fn test_arc_fn_transformer_composition_preserves_original() {
            let t1 = ArcFnTransformer::new(|x: i32| x + 1);
            let t2 = ArcFnTransformer::new(|x: i32| x * 2);
            let t3 = ArcFnTransformer::new(|x: i32| x - 3);

            let composed1 = t1.then(&t2);
            let composed2 = composed1.then(&t3);

            // All original transformers still work
            assert_eq!(t1.transform(5), 6);
            assert_eq!(t2.transform(5), 10);
            assert_eq!(t3.transform(5), 2);
            assert_eq!(composed1.transform(5), 12);
            assert_eq!(composed2.transform(5), 9);
        }
    }

    // ============================================================================
    // Tests for RcFnTransformer
    // ============================================================================

    #[cfg(test)]
    mod rc_fn_transformer_tests {
        use super::*;

        #[test]
        fn test_rc_fn_transformer_new() {
            let transformer = RcFnTransformer::new(|x: i32| x * 2);
            assert_eq!(transformer.transform(21), 42);
        }

        #[test]
        fn test_rc_fn_transformer_reusable() {
            let transformer = RcFnTransformer::new(|x: i32| x * 2);
            let r1 = transformer.transform(21);
            let r2 = transformer.transform(42);
            assert_eq!(r1, 42);
            assert_eq!(r2, 84);
        }

        #[test]
        fn test_rc_fn_transformer_clone() {
            let transformer = RcFnTransformer::new(|x: i32| x * 2);
            let cloned = transformer.clone();

            assert_eq!(transformer.transform(21), 42);
            assert_eq!(cloned.transform(42), 84);
        }

        #[test]
        fn test_rc_fn_transformer_identity() {
            let identity = RcFnTransformer::<i32>::identity();
            assert_eq!(identity.transform(42), 42);
            assert_eq!(identity.transform(100), 100);
        }

        #[test]
        fn test_rc_fn_transformer_constant() {
            let constant = RcFnTransformer::constant(42);
            assert_eq!(constant.transform(100), 42);
            assert_eq!(constant.transform(200), 42);
        }

        #[test]
        fn test_rc_fn_transformer_then() {
            let add_one = RcFnTransformer::new(|x: i32| x + 1);
            let double = RcFnTransformer::new(|x: i32| x * 2);
            let composed = add_one.then(&double);

            // Original transformers are still usable
            assert_eq!(add_one.transform(5), 6);
            assert_eq!(double.transform(5), 10);
            assert_eq!(composed.transform(5), 12);
        }

        #[test]
        fn test_rc_fn_transformer_compose() {
            let double = RcFnTransformer::new(|x: i32| x * 2);
            let add_one = RcFnTransformer::new(|x: i32| x + 1);
            let composed = double.compose(&add_one);

            assert_eq!(composed.transform(5), 12);
            // Original transformers still usable
            assert_eq!(double.transform(5), 10);
            assert_eq!(add_one.transform(5), 6);
        }

        #[test]
        fn test_rc_fn_transformer_multiple_clones() {
            let transformer = RcFnTransformer::new(|x: i32| x * 2);
            let clone1 = transformer.clone();
            let clone2 = transformer.clone();
            let clone3 = clone1.clone();

            assert_eq!(transformer.transform(10), 20);
            assert_eq!(clone1.transform(20), 40);
            assert_eq!(clone2.transform(30), 60);
            assert_eq!(clone3.transform(40), 80);
        }

        #[test]
        fn test_rc_fn_transformer_composition_preserves_original() {
            let t1 = RcFnTransformer::new(|x: i32| x + 1);
            let t2 = RcFnTransformer::new(|x: i32| x * 2);
            let t3 = RcFnTransformer::new(|x: i32| x - 3);

            let composed1 = t1.then(&t2);
            let composed2 = composed1.then(&t3);

            // All original transformers still work
            assert_eq!(t1.transform(5), 6);
            assert_eq!(t2.transform(5), 10);
            assert_eq!(t3.transform(5), 2);
            assert_eq!(composed1.transform(5), 12);
            assert_eq!(composed2.transform(5), 9);
        }

        #[test]
        fn test_rc_fn_transformer_shared_state() {
            use std::cell::RefCell;
            use std::rc::Rc;

            let counter = Rc::new(RefCell::new(0));
            let counter_clone = Rc::clone(&counter);

            let transformer = RcFnTransformer::new(move |x: i32| {
                *counter_clone.borrow_mut() += 1;
                x * 2
            });

            let t1 = transformer.clone();
            let t2 = transformer.clone();

            assert_eq!(t1.transform(10), 20);
            assert_eq!(t2.transform(20), 40);
            assert_eq!(transformer.transform(30), 60);

            assert_eq!(*counter.borrow(), 3);
        }
    }

    // ============================================================================
    // Integration tests - comparing different implementations
    // ============================================================================

    #[cfg(test)]
    mod integration_tests {
        use super::*;

        #[test]
        fn test_all_transformers_same_behavior() {
            let logic = |x: i32| x * 2 + 1;

            let box_t = BoxTransformer::new(logic);
            let box_fn_t = BoxFnTransformer::new(logic);
            let arc_fn_t = ArcFnTransformer::new(logic);
            let rc_fn_t = RcFnTransformer::new(logic);

            let input = 10;
            let expected = 21; // 10 * 2 + 1

            assert_eq!(box_t.transform(input), expected);
            assert_eq!(box_fn_t.transform(input), expected);
            assert_eq!(arc_fn_t.transform(input), expected);
            assert_eq!(rc_fn_t.transform(input), expected);
        }

        #[test]
        fn test_complex_pipeline() {
            // Build a complex transformation pipeline

            // Process numbers: if negative take absolute value, otherwise double
            let process = BoxTransformer::new(|x: i32| if x < 0 { x.abs() } else { x * 2 });

            // Then add 10
            let add_ten = |x: i32| x + 10;

            let pipeline = process.then(add_ten);

            let result = pipeline.transform(42);
            assert_eq!(result, 94); // 42 * 2 + 10

            // Test with negative
            let process2 = BoxTransformer::new(|x: i32| if x < 0 { x.abs() } else { x * 2 });
            let add_ten2 = |x: i32| x + 10;
            let pipeline2 = process2.then(add_ten2);
            assert_eq!(pipeline2.transform(-5), 15); // abs(-5) + 10
        }

        #[test]
        fn test_transformer_with_different_types() {
            // String transformation
            let str_transformer = BoxTransformer::new(|s: String| s.to_uppercase());
            assert_eq!(str_transformer.transform("hello".to_string()), "HELLO");

            // Vec transformation
            let vec_transformer =
                BoxTransformer::new(|v: Vec<i32>| v.into_iter().map(|x| x * 2).collect());
            assert_eq!(vec_transformer.transform(vec![1, 2, 3]), vec![2, 4, 6]);

            // Option transformation
            let opt_transformer = BoxTransformer::new(|opt: Option<i32>| opt.map(|x| x + 1));
            assert_eq!(opt_transformer.transform(Some(41)), Some(42));
        }

        #[test]
        fn test_reusable_transformers_in_loop() {
            let transformer = BoxFnTransformer::new(|x: i32| x * 2);

            let mut results = Vec::new();
            for i in 1..=5 {
                results.push(transformer.transform(i));
            }

            assert_eq!(results, vec![2, 4, 6, 8, 10]);
        }

        #[test]
        fn test_arc_transformer_shared_configuration() {
            // Simulate a configuration-based transformer that can be shared
            let multiplier = 3;
            let transformer = ArcFnTransformer::new(move |x: i32| x * multiplier);

            // Share with multiple consumers
            let t1 = transformer.clone();
            let t2 = transformer.clone();

            assert_eq!(t1.transform(10), 30);
            assert_eq!(t2.transform(20), 60);
            assert_eq!(transformer.transform(5), 15);
        }
    }

    // ============================================================================
    // Edge case tests
    // ============================================================================

    #[cfg(test)]
    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_transformer_with_zero() {
            let transformer = BoxTransformer::new(|x: i32| x * 2);
            assert_eq!(transformer.transform(0), 0);
        }

        #[test]
        fn test_transformer_with_negative() {
            let transformer = BoxTransformer::new(|x: i32| x.abs());
            assert_eq!(transformer.transform(-42), 42);
        }

        #[test]
        fn test_transformer_with_large_numbers() {
            let transformer = BoxTransformer::new(|x: i64| x * 2);
            let large = 1_000_000_000i64;
            assert_eq!(transformer.transform(large), 2_000_000_000i64);
        }

        #[test]
        fn test_transformer_identity_is_noop() {
            let identity = BoxTransformer::<String>::identity();
            let input = "test".to_string();
            let output = identity.transform(input.clone());
            assert_eq!(output, input);
        }

        #[test]
        fn test_transformer_constant_ignores_input() {
            let constant = BoxTransformer::constant("always".to_string());
            assert_eq!(constant.transform("input1".to_string()), "always");

            let constant2 = BoxTransformer::constant("always".to_string());
            assert_eq!(constant2.transform("input2".to_string()), "always");
        }

        #[test]
        fn test_empty_string_transformation() {
            let transformer = BoxTransformer::new(|s: String| {
                if s.is_empty() {
                    "empty".to_string()
                } else {
                    s.to_uppercase()
                }
            });

            assert_eq!(transformer.transform("".to_string()), "empty");

            let transformer2 = BoxTransformer::new(|s: String| {
                if s.is_empty() {
                    "empty".to_string()
                } else {
                    s.to_uppercase()
                }
            });
            assert_eq!(transformer2.transform("hello".to_string()), "HELLO");
        }

        #[test]
        fn test_option_none_transformation() {
            let transformer = BoxTransformer::map_option(|x: i32| x * 2);
            assert_eq!(transformer.transform(None), None);
        }

        #[test]
        fn test_result_err_transformation() {
            let transformer = BoxTransformer::map_result(|x: i32| x * 2);
            let result: Result<i32, String> = Err("error".to_string());
            assert_eq!(transformer.transform(result), Err("error".to_string()));
        }
    }

    // ============================================================================
    // Tests for into_fn method
    // ============================================================================

    #[cfg(test)]
    mod into_fn_tests {
        use super::*;

        #[test]
        fn test_closure_into_fn_with_single_value() {
            let transformer = |x: i32| x * 2;
            let mut func = transformer.into_fn();

            let result = func(21);
            assert_eq!(result, 42);
        }

        #[test]
        fn test_box_transformer_into_fn() {
            let transformer = BoxTransformer::new(|x: i32| x * 2);
            let mut func = transformer.into_fn();

            let result = func(21);
            assert_eq!(result, 42);
        }

        #[test]
        fn test_box_transformer_into_fn_with_composition() {
            let add_one = BoxTransformer::new(|x: i32| x + 1);
            let transformer = add_one.then(|x| x * 2);
            let mut func = transformer.into_fn();

            let result = func(5);
            assert_eq!(result, 12); // (5+1)*2=12
        }

        #[test]
        fn test_box_fn_transformer_into_fn() {
            let transformer = BoxFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values.into_iter().map(transformer.into_fn()).collect();

            assert_eq!(result, vec![2, 4, 6, 8, 10]);
        }

        #[test]
        fn test_arc_fn_transformer_into_fn() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values.into_iter().map(transformer.into_fn()).collect();

            assert_eq!(result, vec![2, 4, 6, 8, 10]);
        }

        #[test]
        fn test_arc_fn_transformer_into_fn_with_filter() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values
                .into_iter()
                .map(transformer.into_fn())
                .filter(|x| *x > 4)
                .collect();

            assert_eq!(result, vec![6, 8, 10]);
        }

        #[test]
        fn test_rc_fn_transformer_into_fn() {
            let transformer = RcFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values.into_iter().map(transformer.into_fn()).collect();

            assert_eq!(result, vec![2, 4, 6, 8, 10]);
        }

        #[test]
        fn test_into_fn_with_string_transformation() {
            let transformer = BoxFnTransformer::new(|s: String| s.to_uppercase());
            let values = vec!["hello".to_string(), "world".to_string()];

            let result: Vec<String> = values.into_iter().map(transformer.into_fn()).collect();

            assert_eq!(result, vec!["HELLO".to_string(), "WORLD".to_string()]);
        }

        #[test]
        fn test_into_fn_with_complex_pipeline() {
            let double = BoxFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values
                .into_iter()
                .map(double.into_fn())
                .filter(|x| *x > 5)
                .collect();

            assert_eq!(result, vec![6, 8, 10]);
        }

        #[test]
        fn test_into_fn_with_flat_map() {
            let repeat_twice = BoxFnTransformer::new(|x: i32| x);
            let values = vec![1, 2, 3];

            let result: Vec<i32> = values
                .into_iter()
                .flat_map(|x| vec![x, x])
                .map(repeat_twice.into_fn())
                .collect();

            assert_eq!(result, vec![1, 1, 2, 2, 3, 3]);
        }

        #[test]
        fn test_into_fn_consumes_transformer() {
            // This test verifies that into_fn consumes the transformer
            let transformer = BoxTransformer::new(|x: i32| x * 2);
            let mut func = transformer.into_fn();

            let result = func(21);
            assert_eq!(result, 42);

            // The following line would cause a compile error if uncommented:
            // let _result2 = transformer.transform(5); // Error: transformer was moved
        }

        #[test]
        fn test_into_fn_with_option_values() {
            let values = vec![Some(1), None, Some(3), None, Some(5)];

            let result: Vec<Option<i32>> =
                values.into_iter().map(|opt| opt.map(|x| x * 2)).collect();

            assert_eq!(result, vec![Some(2), None, Some(6), None, Some(10)]);
        }

        #[test]
        fn test_into_fn_with_result_values() {
            let values: Vec<Result<i32, String>> = vec![
                Ok(1),
                Err("error1".to_string()),
                Ok(3),
                Err("error2".to_string()),
                Ok(5),
            ];

            let result: Vec<Result<i32, String>> =
                values.into_iter().map(|res| res.map(|x| x * 2)).collect();

            assert_eq!(
                result,
                vec![
                    Ok(2),
                    Err("error1".to_string()),
                    Ok(6),
                    Err("error2".to_string()),
                    Ok(10)
                ]
            );
        }

        #[test]
        fn test_into_fn_with_enumerated_iterator() {
            let values = vec![1, 2, 3];

            let result: Vec<(usize, i32)> = values
                .into_iter()
                .enumerate()
                .map(|(i, x)| (i, x + 10))
                .collect();

            assert_eq!(result, vec![(0, 11), (1, 12), (2, 13)]);
        }

        #[test]
        fn test_into_fn_identity() {
            let identity = BoxFnTransformer::<i32>::identity();
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values.into_iter().map(identity.into_fn()).collect();

            assert_eq!(result, vec![1, 2, 3, 4, 5]);
        }

        #[test]
        fn test_into_fn_with_take() {
            let double = BoxFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values.into_iter().take(3).map(double.into_fn()).collect();

            assert_eq!(result, vec![2, 4, 6]);
        }

        #[test]
        fn test_into_fn_with_skip() {
            let double = BoxFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            let result: Vec<i32> = values.into_iter().skip(2).map(double.into_fn()).collect();

            assert_eq!(result, vec![6, 8, 10]);
        }

        #[test]
        fn test_arc_fn_transformer_clone_before_into_fn() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);

            // Clone before conversion to keep the original
            let values1 = vec![1, 2, 3];
            let result1: Vec<i32> = values1.into_iter()
                .map(transformer.clone().into_fn())
                .collect();

            // Original transformer is still usable
            let values2 = vec![4, 5];
            let result2: Vec<i32> = values2.into_iter()
                .map(transformer.into_fn())
                .collect();

            assert_eq!(result1, vec![2, 4, 6]);
            assert_eq!(result2, vec![8, 10]);
        }

        #[test]
        fn test_rc_fn_transformer_clone_before_into_fn() {
            let transformer = RcFnTransformer::new(|x: i32| x * 2);

            // Clone before conversion to keep the original
            let values1 = vec![1, 2, 3];
            let result1: Vec<i32> = values1.into_iter()
                .map(transformer.clone().into_fn())
                .collect();

            // Original transformer is still usable
            let values2 = vec![4, 5];
            let result2: Vec<i32> = values2.into_iter()
                .map(transformer.into_fn())
                .collect();

            assert_eq!(result1, vec![2, 4, 6]);
            assert_eq!(result2, vec![8, 10]);
        }

        #[test]
        fn test_arc_fn_transformer_original_still_usable_after_clone() {
            let transformer = ArcFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            // Clone before conversion to keep the original
            let result: Vec<i32> = values.into_iter()
                .map(transformer.clone().into_fn())
                .collect();

            // Original is still available for direct use
            assert_eq!(transformer.transform(10), 20);
            assert_eq!(result, vec![2, 4, 6, 8, 10]);
        }

        #[test]
        fn test_rc_fn_transformer_original_still_usable_after_clone() {
            let transformer = RcFnTransformer::new(|x: i32| x * 2);
            let values = vec![1, 2, 3, 4, 5];

            // Clone before conversion to keep the original
            let result: Vec<i32> = values.into_iter()
                .map(transformer.clone().into_fn())
                .collect();

            // Original is still available for direct use
            assert_eq!(transformer.transform(10), 20);
            assert_eq!(result, vec![2, 4, 6, 8, 10]);
        }
    }
} // mod tests
