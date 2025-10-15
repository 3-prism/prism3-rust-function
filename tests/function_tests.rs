/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    ArcFnFunction, BoxFnFunction, BoxFunction, FnFunctionOps, Function, RcFnFunction,
};

// ============================================================================
// BoxFunction Tests - One-time use
// ============================================================================

#[cfg(test)]
mod box_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = BoxFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxFunction::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxFunction::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_compose() {
        let double = BoxFunction::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_and_then_chain() {
        let add_one = BoxFunction::new(|x: i32| x + 1);
        let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
        assert_eq!(pipeline.apply(5), 9); // ((5 + 1) * 2) - 3 = 9
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxFunction::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.apply(42), "42_suffix");
    }

    #[test]
    fn test_map_option() {
        let double = |x: i32| x * 2;
        let option_double = BoxFunction::map_option(double);
        assert_eq!(option_double.apply(Some(21)), Some(42));

        let double2 = |x: i32| x * 2;
        let option_double2 = BoxFunction::map_option(double2);
        assert_eq!(option_double2.apply(None), None);
    }

    #[test]
    fn test_map_result() {
        let double = |x: i32| x * 2;
        let result_double = BoxFunction::map_result(double);
        assert_eq!(result_double.apply(Ok::<i32, &str>(21)), Ok(42));

        let double2 = |x: i32| x * 2;
        let result_double2 = BoxFunction::map_result(double2);
        assert_eq!(
            result_double2.apply(Err::<i32, &str>("error")),
            Err("error")
        );
    }

    #[test]
    fn test_result_to_option() {
        let to_option = BoxFunction::<Result<i32, &str>, Option<i32>>::result_to_option();
        assert_eq!(to_option.apply(Ok(42)), Some(42));

        let to_option2 = BoxFunction::<Result<i32, &str>, Option<i32>>::result_to_option();
        assert_eq!(to_option2.apply(Err("error")), None);
    }

    #[test]
    fn test_option_to_result() {
        let to_result = BoxFunction::option_to_result("missing");
        assert_eq!(to_result.apply(Some(42)), Ok(42));

        let to_result2: BoxFunction<Option<i32>, Result<i32, &str>> =
            BoxFunction::option_to_result("missing");
        assert_eq!(to_result2.apply(None), Err("missing"));
    }

    #[test]
    fn test_unwrap_or_else() {
        let use_default = |_err: &str| 0;
        let unwrap = BoxFunction::unwrap_or_else(use_default);
        assert_eq!(unwrap.apply(Ok(42)), 42);

        let use_default2 = |_err: &str| 0;
        let unwrap2 = BoxFunction::unwrap_or_else(use_default2);
        assert_eq!(unwrap2.apply(Err("error")), 0);
    }

    #[test]
    fn test_match_result() {
        let handle =
            BoxFunction::match_result(|x: i32| x.to_string(), |e: &str| format!("Error: {}", e));
        assert_eq!(handle.apply(Ok(42)), "42");

        let handle2 =
            BoxFunction::match_result(|x: i32| x.to_string(), |e: &str| format!("Error: {}", e));
        assert_eq!(handle2.apply(Err("failed")), "Error: failed");
    }

    #[test]
    fn test_flatten_result() {
        let flatten =
            BoxFunction::<Result<Result<i32, &str>, &str>, Result<i32, &str>>::flatten_result();
        assert_eq!(flatten.apply(Ok(Ok(42))), Ok(42));

        let flatten2 =
            BoxFunction::<Result<Result<i32, &str>, &str>, Result<i32, &str>>::flatten_result();
        assert_eq!(flatten2.apply(Ok(Err("inner"))), Err("inner"));

        let flatten3 =
            BoxFunction::<Result<Result<i32, &str>, &str>, Result<i32, &str>>::flatten_result();
        assert_eq!(flatten3.apply(Err("outer")), Err("outer"));
    }

    #[test]
    fn test_flatten_option() {
        let flatten = BoxFunction::<Option<Option<i32>>, Option<i32>>::flatten_option();
        assert_eq!(flatten.apply(Some(Some(42))), Some(42));

        let flatten2 = BoxFunction::<Option<Option<i32>>, Option<i32>>::flatten_option();
        assert_eq!(flatten2.apply(Some(None)), None);

        let flatten3 = BoxFunction::<Option<Option<i32>>, Option<i32>>::flatten_option();
        assert_eq!(flatten3.apply(None), None);
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let multiply = BoxFunction::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.apply(7), 21);
    }

    #[test]
    fn test_complex_pipeline() {
        let add_ten = BoxFunction::new(|x: i32| x + 10);
        let pipeline = add_ten.and_then(|x| x * 2).and_then(|x: i32| x.to_string());
        assert_eq!(pipeline.apply(5), "30");
    }
}

// ============================================================================
// BoxFnFunction Tests - Reusable, single ownership
// ============================================================================

#[cfg(test)]
mod box_fn_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = BoxFnFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let double = BoxFnFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
        assert_eq!(double.apply(42), 84);
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_identity() {
        let identity = BoxFnFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
        assert_eq!(identity.apply(100), 100);
    }

    #[test]
    fn test_constant() {
        let constant = BoxFnFunction::constant("hello");
        assert_eq!(constant.apply(123), "hello");
        assert_eq!(constant.apply(456), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxFnFunction::new(|x: i32| x + 1);
        let double = BoxFnFunction::new(|x: i32| x * 2);
        let composed = add_one.and_then(double);

        assert_eq!(composed.apply(5), 12);
        assert_eq!(composed.apply(10), 22);
    }

    #[test]
    fn test_compose() {
        let double = BoxFnFunction::new(|x: i32| x * 2);
        let add_one = BoxFnFunction::new(|x: i32| x + 1);
        let composed = double.compose(add_one);

        assert_eq!(composed.apply(5), 12);
        assert_eq!(composed.apply(10), 22);
    }

    #[test]
    fn test_type_conversion_multiple_calls() {
        let to_string = BoxFnFunction::new(|x: i32| x.to_string());
        assert_eq!(to_string.apply(42), "42");
        assert_eq!(to_string.apply(100), "100");
    }
}

// ============================================================================
// ArcFnFunction Tests - Multi-threaded sharing, reusable
// ============================================================================

#[cfg(test)]
mod arc_fn_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = ArcFnFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let double = ArcFnFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
        assert_eq!(double.apply(42), 84);
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_clone() {
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(21), 42);
        assert_eq!(cloned.apply(42), 84);
    }

    #[test]
    fn test_identity() {
        let identity = ArcFnFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
        assert_eq!(identity.apply(100), 100);
    }

    #[test]
    fn test_constant() {
        let constant = ArcFnFunction::constant("hello");
        assert_eq!(constant.apply(123), "hello");
        assert_eq!(constant.apply(456), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = ArcFnFunction::new(|x: i32| x + 1);
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let composed = add_one.and_then(&double);

        // Original functions are still available
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(double.apply(5), 10);
        assert_eq!(composed.apply(5), 12);
    }

    #[test]
    fn test_compose() {
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let add_one = ArcFnFunction::new(|x: i32| x + 1);
        let composed = double.compose(&add_one);

        // Original functions are still available
        assert_eq!(double.apply(5), 10);
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(composed.apply(5), 12);
    }

    #[test]
    fn test_and_then_preserves_original() {
        let func = ArcFnFunction::new(|x: i32| x * 2);
        let composed1 = func.and_then(&ArcFnFunction::new(|x| x + 1));
        let composed2 = func.and_then(&ArcFnFunction::new(|x| x - 1));

        // func is still available
        assert_eq!(func.apply(10), 20);
        assert_eq!(composed1.apply(10), 21);
        assert_eq!(composed2.apply(10), 19);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let double = ArcFnFunction::new(|x: i32| x * 2);
        let cloned = double.clone();

        let handle = thread::spawn(move || cloned.apply(21));

        assert_eq!(double.apply(42), 84);
        assert_eq!(handle.join().unwrap(), 42);
    }

    #[test]
    fn test_multiple_threads() {
        use std::thread;

        let double = ArcFnFunction::new(|x: i32| x * 2);

        let handles: Vec<_> = (0..5)
            .map(|i| {
                let func = double.clone();
                thread::spawn(move || func.apply(i * 10))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(results, vec![0, 20, 40, 60, 80]);
    }
}

// ============================================================================
// RcFnFunction Tests - Single-threaded sharing, reusable
// ============================================================================

#[cfg(test)]
mod rc_fn_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = RcFnFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let double = RcFnFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
        assert_eq!(double.apply(42), 84);
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_clone() {
        let double = RcFnFunction::new(|x: i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(21), 42);
        assert_eq!(cloned.apply(42), 84);
    }

    #[test]
    fn test_identity() {
        let identity = RcFnFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
        assert_eq!(identity.apply(100), 100);
    }

    #[test]
    fn test_constant() {
        let constant = RcFnFunction::constant("hello");
        assert_eq!(constant.apply(123), "hello");
        assert_eq!(constant.apply(456), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = RcFnFunction::new(|x: i32| x + 1);
        let double = RcFnFunction::new(|x: i32| x * 2);
        let composed = add_one.and_then(&double);

        // Original functions are still available
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(double.apply(5), 10);
        assert_eq!(composed.apply(5), 12);
    }

    #[test]
    fn test_compose() {
        let double = RcFnFunction::new(|x: i32| x * 2);
        let add_one = RcFnFunction::new(|x: i32| x + 1);
        let composed = double.compose(&add_one);

        // Original functions are still available
        assert_eq!(double.apply(5), 10);
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(composed.apply(5), 12);
    }

    #[test]
    fn test_and_then_preserves_original() {
        let func = RcFnFunction::new(|x: i32| x * 2);
        let composed1 = func.and_then(&RcFnFunction::new(|x| x + 1));
        let composed2 = func.and_then(&RcFnFunction::new(|x| x - 1));

        // func is still available
        assert_eq!(func.apply(10), 20);
        assert_eq!(composed1.apply(10), 21);
        assert_eq!(composed2.apply(10), 19);
    }

    #[test]
    fn test_multiple_clones() {
        let func = RcFnFunction::new(|x: i32| x * 2);
        let clone1 = func.clone();
        let clone2 = func.clone();
        let clone3 = clone1.clone();

        assert_eq!(func.apply(10), 20);
        assert_eq!(clone1.apply(20), 40);
        assert_eq!(clone2.apply(30), 60);
        assert_eq!(clone3.apply(40), 80);
    }
}

// ============================================================================
// FnFunctionOps Tests - Closure extension methods
// ============================================================================

#[cfg(test)]
mod fn_function_ops_tests {
    use super::*;

    #[test]
    fn test_closure_and_then() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);

        assert_eq!(composed.apply(5), 12);
    }

    #[test]
    fn test_closure_compose() {
        let double = |x: i32| x * 2;
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);

        assert_eq!(composed.apply(5), 12);
    }

    #[test]
    fn test_closure_chain() {
        let result = (|x: i32| x + 1).and_then(|x| x * 2).and_then(|x| x - 3);

        assert_eq!(result.apply(5), 9);
    }

    #[test]
    fn test_closure_type_conversion() {
        let to_string = |x: i32| x.to_string();
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));

        assert_eq!(add_suffix.apply(42), "42_suffix");
    }
}

// ============================================================================
// Function Trait Tests - Unified interface
// ============================================================================

#[cfg(test)]
mod function_trait_tests {
    use super::*;

    // Since BoxFunction's apply method consumes self,
    // we cannot pass it as a Function trait object
    // This is an inherent limitation of FnOnce

    #[test]
    fn test_box_function_direct_call() {
        let double = BoxFunction::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_closure_direct_call() {
        let double = |x: i32| x * 2;
        let result: i32 = double.apply(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_fn_pointer_direct_call() {
        fn double(x: i32) -> i32 {
            x * 2
        }
        let result: i32 = double.apply(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_batch_with_arc_fn_function() {
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let values = vec![1, 2, 3, 4, 5];
        let results: Vec<i32> = values.into_iter().map(|v| double.apply(v)).collect();
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }
}

// ============================================================================
// Integration Tests - Complex scenarios
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_mixed_composition() {
        let box_func = BoxFunction::new(|x: i32| x + 1);
        let closure = |x: i32| x * 2;
        let composed = box_func.and_then(closure);

        assert_eq!(composed.apply(5), 12);
    }

    #[test]
    fn test_arc_fn_function_complex_pipeline() {
        let parse = ArcFnFunction::new(|s: String| s.parse::<i32>().unwrap_or(0));
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let to_string = ArcFnFunction::new(|x: i32| x.to_string());

        let pipeline = parse.and_then(&double).and_then(&to_string);

        assert_eq!(pipeline.apply("21".to_string()), "42");
        assert_eq!(parse.apply("10".to_string()), 10);
        assert_eq!(double.apply(5), 10);
    }

    #[test]
    fn test_rc_fn_function_reuse() {
        let double = RcFnFunction::new(|x: i32| x * 2);
        let add_ten = RcFnFunction::new(|x: i32| x + 10);

        let pipeline1 = double.and_then(&add_ten);
        let pipeline2 = add_ten.and_then(&double);

        assert_eq!(pipeline1.apply(5), 20); // (5 * 2) + 10 = 20
        assert_eq!(pipeline2.apply(5), 30); // (5 + 10) * 2 = 30

        // Original functions can still be used
        assert_eq!(double.apply(7), 14);
        assert_eq!(add_ten.apply(7), 17);
    }

    #[test]
    fn test_error_handling_pipeline() {
        let divide = BoxFunction::new(|x: i32| {
            if x != 0 {
                Ok(100 / x)
            } else {
                Err("division by zero")
            }
        });

        let handle = divide.and_then(|result| result.unwrap_or(0));

        assert_eq!(handle.apply(10), 10);

        let divide2 = BoxFunction::new(|x: i32| {
            if x != 0 {
                Ok(100 / x)
            } else {
                Err("division by zero")
            }
        });
        let handle2 = divide2.and_then(|result| result.unwrap_or(0));
        assert_eq!(handle2.apply(0), 0);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_with_max_i32() {
        let identity = BoxFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(i32::MAX), i32::MAX);
    }

    #[test]
    fn test_with_min_i32() {
        let identity = BoxFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(i32::MIN), i32::MIN);
    }

    #[test]
    fn test_with_empty_string() {
        let to_upper = BoxFunction::new(|s: String| s.to_uppercase());
        assert_eq!(to_upper.apply(String::new()), "");
    }

    #[test]
    fn test_with_long_string() {
        let get_length = BoxFunction::new(|s: String| s.len());
        let long_string = "a".repeat(10000);
        assert_eq!(get_length.apply(long_string), 10000);
    }

    #[test]
    fn test_with_tuple_input() {
        let sum = BoxFunction::new(|(a, b): (i32, i32)| a + b);
        assert_eq!(sum.apply((10, 20)), 30);
    }

    #[test]
    fn test_with_tuple_output() {
        let split = BoxFunction::new(|x: i32| (x / 2, x % 2));
        assert_eq!(split.apply(7), (3, 1));
    }

    #[test]
    fn test_arc_fn_function_with_copy_types() {
        let func = ArcFnFunction::new(|x: i32| x * 2);
        let results: Vec<_> = vec![1, 2, 3, 4, 5]
            .into_iter()
            .map(|x| func.apply(x))
            .collect();

        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }
}

// ============================================================================
// into_fn Tests - Testing the into_fn method for all function types
// ============================================================================

#[cfg(test)]
mod into_fn_tests {
    use super::*;

    // BoxFunction into_fn tests
    #[test]
    fn test_box_function_into_fn_basic() {
        let double = BoxFunction::new(|x: i32| x * 2);

        // into_fn converts to a closure
        let closure = double.into_fn();
        assert_eq!(closure(21), 42);
    }

    #[test]
    fn test_box_function_into_fn_with_once() {
        let parse_and_double = BoxFunction::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);

        let closure = parse_and_double.into_fn();
        assert_eq!(closure("21".to_string()), 42);
    }

    #[test]
    fn test_box_function_into_fn_with_string() {
        let to_upper = BoxFunction::new(|s: String| s.to_uppercase());

        let closure = to_upper.into_fn();
        assert_eq!(closure("hello".to_string()), "HELLO".to_string());
    }

    #[test]
    fn test_box_function_into_fn_with_option() {
        let parse = BoxFunction::new(|s: String| s.parse::<i32>().ok());

        let closure = parse.into_fn();
        assert_eq!(closure("42".to_string()), Some(42));
        // Note: closure is consumed after one use (FnOnce)
    }

    // BoxFnFunction into_fn tests
    #[test]
    fn test_box_fn_function_into_fn_with_map() {
        let double = BoxFnFunction::new(|x: i32| x * 2);
        let values = vec![1, 2, 3, 4, 5];

        let closure = double.into_fn();
        let result: Vec<i32> = values.iter().map(|&x| closure(x)).collect();

        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_box_fn_function_into_fn_reusable() {
        let double = BoxFnFunction::new(|x: i32| x * 2);
        let closure = double.into_fn();

        // Can use the closure multiple times
        assert_eq!(closure(5), 10);
        assert_eq!(closure(10), 20);
        assert_eq!(closure(15), 30);
    }

    // ArcFnFunction into_fn tests
    #[test]
    fn test_arc_fn_function_into_fn_with_map() {
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let values = vec![1, 2, 3, 4, 5];

        let closure = double.into_fn();
        let result: Vec<i32> = values.iter().map(|&x| closure(x)).collect();

        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_arc_fn_function_into_fn_cloneable() {
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let closure1 = double.into_fn();
        let closure2 = closure1.clone();

        // Both closures work
        assert_eq!(closure1(5), 10);
        assert_eq!(closure2(10), 20);
    }

    #[test]
    fn test_arc_fn_function_into_fn_thread_safe() {
        use std::thread;

        let double = ArcFnFunction::new(|x: i32| x * 2);
        let closure = double.into_fn();
        let closure_clone = closure.clone();

        let handle = thread::spawn(move || closure_clone(21));

        assert_eq!(closure(10), 20);
        assert_eq!(handle.join().unwrap(), 42);
    }

    #[test]
    fn test_arc_fn_function_into_fn_multiple_threads() {
        use std::thread;

        let processor = ArcFnFunction::new(|x: i32| x * x);
        let closure = processor.into_fn();

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let c = closure.clone();
                thread::spawn(move || c(i))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(results, vec![0, 1, 4, 9]);
    }

    // RcFnFunction into_fn tests
    #[test]
    fn test_rc_fn_function_into_fn_with_map() {
        let double = RcFnFunction::new(|x: i32| x * 2);
        let values = vec![1, 2, 3, 4, 5];

        let closure = double.into_fn();
        let result: Vec<i32> = values.iter().map(|&x| closure(x)).collect();

        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_rc_fn_function_into_fn_cloneable() {
        let double = RcFnFunction::new(|x: i32| x * 2);
        let closure1 = double.into_fn();
        let closure2 = closure1.clone();

        // Both closures work
        assert_eq!(closure1(5), 10);
        assert_eq!(closure2(10), 20);
    }

    #[test]
    fn test_rc_fn_function_into_fn_reusable() {
        let triple = RcFnFunction::new(|x: i32| x * 3);
        let closure = triple.into_fn();

        // Can use multiple times
        let results: Vec<_> = vec![1, 2, 3, 4, 5].iter().map(|&x| closure(x)).collect();

        assert_eq!(results, vec![3, 6, 9, 12, 15]);
    }

    // Closure into_fn tests (via Function trait)
    #[test]
    fn test_closure_into_fn_basic() {
        let double = |x: i32| x * 2;

        let closure = double.into_fn();
        assert_eq!(closure(21), 42);
    }

    #[test]
    fn test_closure_into_fn_complex() {
        let parse_and_square = |s: String| s.parse::<i32>().unwrap_or(0).pow(2);

        let closure = parse_and_square.into_fn();
        assert_eq!(closure("5".to_string()), 25);
    }

    // Integration tests with composed functions
    #[test]
    fn test_box_function_composition_into_fn() {
        let add_one = BoxFunction::new(|x: i32| x + 1);
        let composed = add_one.and_then(|x| x * 2);

        let closure = composed.into_fn();
        assert_eq!(closure(5), 12); // (5+1)*2=12
    }

    #[test]
    fn test_arc_fn_function_composition_into_fn() {
        let add_one = ArcFnFunction::new(|x: i32| x + 1);
        let double = ArcFnFunction::new(|x: i32| x * 2);
        let composed = add_one.and_then(&double);

        let closure = composed.into_fn();
        let values = vec![1, 2, 3];
        let result: Vec<i32> = values.iter().map(|&x| closure(x)).collect();

        assert_eq!(result, vec![4, 6, 8]);
    }

    // Edge case tests
    #[test]
    fn test_into_fn_with_option_type() {
        let extract = BoxFunction::new(|opt: Option<i32>| opt.unwrap_or(0));

        let closure = extract.into_fn();
        assert_eq!(closure(Some(42)), 42);
    }

    #[test]
    fn test_into_fn_with_result_type() {
        let handle = BoxFunction::new(|res: Result<i32, &str>| res.unwrap_or(0));

        let closure = handle.into_fn();
        assert_eq!(closure(Ok(42)), 42);
    }

    #[test]
    fn test_into_fn_with_tuple() {
        let sum = BoxFunction::new(|(a, b): (i32, i32)| a + b);

        let closure = sum.into_fn();
        assert_eq!(closure((10, 20)), 30);
    }
}
