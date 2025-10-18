/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcBiTransformer, BiTransformer, BoxBiTransformer, RcBiTransformer};
use std::thread;

// ============================================================================
// BoxBiTransformer Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod box_bi_transformer_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.transform(20, 22), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.transform(20, 22), 42);
        assert_eq!(add.transform(10, 10), 20);
        assert_eq!(add.transform(5, 3), 8);
    }

    #[test]
    fn test_multiply() {
        let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
        assert_eq!(multiply.transform(6, 7), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxBiTransformer::constant("hello");
        assert_eq!(constant.transform(123, 456), "hello");
        assert_eq!(constant.transform(789, 101), "hello");
    }

    #[test]
    fn test_with_string() {
        let concat = BoxBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));
        assert_eq!(
            concat.transform("hello".to_string(), "world".to_string()),
            "helloworld"
        );
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let weighted_sum =
            BoxBiTransformer::new(move |x: i32, y: i32| x * multiplier + y * multiplier);
        assert_eq!(weighted_sum.transform(2, 3), 15); // (2 * 3) + (3 * 3) = 15
    }

    #[test]
    fn test_different_types() {
        let format = BoxBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.transform("Alice".to_string(), 30), "Alice is 30");
    }

    #[test]
    fn test_with_option() {
        let safe_divide =
            BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });
        assert_eq!(safe_divide.transform(42, 2), Some(21));
        assert_eq!(safe_divide.transform(42, 0), None);
    }
}

// ============================================================================
// ArcBiTransformer Tests - Immutable, thread-safe
// ============================================================================

#[cfg(test)]
mod arc_bi_transformer_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.transform(20, 22), 42);
    }

    #[test]
    fn test_clone() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        assert_eq!(add.transform(20, 22), 42);
        assert_eq!(cloned.transform(20, 22), 42);
    }

    #[test]
    fn test_thread_safe() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        let handle = thread::spawn(move || cloned.transform(20, 22));

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(add.transform(20, 22), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcBiTransformer::constant("hello");
        assert_eq!(constant.transform(123, 456), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let m = multiply.clone();
                thread::spawn(move || m.transform(i, i + 1))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(results, vec![0, 2, 6, 12]); // 0*1, 1*2, 2*3, 3*4
    }

    #[test]
    fn test_with_different_types() {
        let format = ArcBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.transform("Alice".to_string(), 30), "Alice is 30");
    }
}

// ============================================================================
// RcBiTransformer Tests - Immutable, single-threaded
// ============================================================================

#[cfg(test)]
mod rc_bi_transformer_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.transform(20, 22), 42);
    }

    #[test]
    fn test_clone() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        assert_eq!(add.transform(20, 22), 42);
        assert_eq!(cloned.transform(20, 22), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcBiTransformer::constant("hello");
        assert_eq!(constant.transform(123, 456), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let concat = RcBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));

        let func1 = concat.clone();
        let func2 = concat.clone();

        assert_eq!(
            concat.transform("hello".to_string(), "world".to_string()),
            "helloworld"
        );
        assert_eq!(
            func1.transform("foo".to_string(), "bar".to_string()),
            "foobar"
        );
        assert_eq!(
            func2.transform("rust".to_string(), "lang".to_string()),
            "rustlang"
        );
    }

    #[test]
    fn test_with_different_types() {
        let format = RcBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.transform("Alice".to_string(), 30), "Alice is 30");
    }
}

// ============================================================================
// Conditional BiTransformer Tests
// ============================================================================

#[cfg(test)]
mod box_conditional_tests {
    use super::*;
    use prism3_function::BoxBiPredicate;

    #[test]
    fn test_when_or_else() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let both_positive = BoxBiPredicate::new(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
        let result = add.when(both_positive).or_else(multiply);

        assert_eq!(result.transform(5, 3), 8); // both positive, add
        assert_eq!(result.transform(-5, 3), -15); // not both positive, multiply
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);

        assert_eq!(result.transform(5, 3), 8);
        assert_eq!(result.transform(-5, 3), -15);
        assert_eq!(result.transform(0, 5), 0);
    }
}

#[cfg(test)]
mod arc_conditional_tests {
    use super::*;
    use prism3_function::ArcBiPredicate;

    #[test]
    fn test_when_or_else() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let both_positive = ArcBiPredicate::new(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);
        let result = add.when(both_positive).or_else(multiply);

        assert_eq!(result.transform(5, 3), 8);
        assert_eq!(result.transform(-5, 3), -15);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);

        assert_eq!(result.transform(5, 3), 8);
        assert_eq!(result.transform(-5, 3), -15);
        assert_eq!(result.transform(0, 5), 0);
    }

    #[test]
    fn test_conditional_clone() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply);

        let conditional_clone = conditional.clone();

        assert_eq!(conditional.transform(5, 3), 8);
        assert_eq!(conditional_clone.transform(-5, 3), -15);
    }
}

#[cfg(test)]
mod rc_conditional_tests {
    use super::*;
    use prism3_function::RcBiPredicate;

    #[test]
    fn test_when_or_else() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let both_positive = RcBiPredicate::new(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let multiply = RcBiTransformer::new(|x: i32, y: i32| x * y);
        let result = add.when(both_positive).or_else(multiply);

        assert_eq!(result.transform(5, 3), 8);
        assert_eq!(result.transform(-5, 3), -15);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);

        assert_eq!(result.transform(5, 3), 8);
        assert_eq!(result.transform(-5, 3), -15);
        assert_eq!(result.transform(0, 5), 0);
    }

    #[test]
    fn test_conditional_clone() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let multiply = RcBiTransformer::new(|x: i32, y: i32| x * y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply);

        let conditional_clone = conditional.clone();

        assert_eq!(conditional.transform(5, 3), 8);
        assert_eq!(conditional_clone.transform(-5, 3), -15);
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_closure_to_box() {
        let add = |x: i32, y: i32| x + y;
        let boxed = add.into_box();
        assert_eq!(boxed.transform(20, 22), 42);
    }

    #[test]
    fn test_closure_to_arc() {
        let add = |x: i32, y: i32| x + y;
        let arc = add.into_arc();
        assert_eq!(arc.transform(20, 22), 42);
    }

    #[test]
    fn test_closure_to_rc() {
        let add = |x: i32, y: i32| x + y;
        let rc = add.into_rc();
        assert_eq!(rc.transform(20, 22), 42);
    }

    #[test]
    fn test_box_to_fn() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.into_fn();
        assert_eq!(func(20, 22), 42);
    }

    #[test]
    fn test_arc_to_fn() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.into_fn();
        assert_eq!(func(20, 22), 42);
    }

    #[test]
    fn test_rc_to_fn() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.into_fn();
        assert_eq!(func(20, 22), 42);
    }

    #[test]
    fn test_box_to_rc() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.transform(20, 22), 42);
    }

    #[test]
    fn test_arc_to_box() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.transform(20, 22), 42);
    }

    #[test]
    fn test_arc_to_rc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.transform(20, 22), 42);
    }

    #[test]
    fn test_rc_to_box() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.transform(20, 22), 42);
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_bi_transformer_trait() {
        fn apply_bi_transformer<F: BiTransformer<i32, i32, i32>>(f: &F, x: i32, y: i32) -> i32 {
            f.transform(x, y)
        }

        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(apply_bi_transformer(&add, 20, 22), 42);
    }

    #[test]
    fn test_closure_as_bi_transformer() {
        fn apply_bi_transformer<F: BiTransformer<i32, i32, i32>>(f: &F, x: i32, y: i32) -> i32 {
            f.transform(x, y)
        }

        let add = |x: i32, y: i32| x + y;
        assert_eq!(apply_bi_transformer(&add, 20, 22), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_bi_transformer<T, U, R, F: BiTransformer<T, U, R>>(f: &F, x: T, y: U) -> R {
            f.transform(x, y)
        }

        let format = BoxBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(
            apply_bi_transformer(&format, "Alice".to_string(), 30),
            "Alice is 30"
        );
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxBiTransformer::constant("hello");
        assert_eq!(constant.transform(123, 456), "hello");
        assert_eq!(constant.transform(789, 101), "hello");
    }

    #[test]
    fn test_with_option() {
        let safe_divide =
            BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });
        assert_eq!(safe_divide.transform(42, 2), Some(21));
        assert_eq!(safe_divide.transform(42, 0), None);
    }

    #[test]
    fn test_with_result() {
        let safe_divide = BoxBiTransformer::new(|x: i32, y: i32| -> Result<i32, String> {
            if y == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(x / y)
            }
        });
        assert_eq!(safe_divide.transform(42, 2), Ok(21));
        assert!(safe_divide.transform(42, 0).is_err());
    }

    #[test]
    fn test_with_vec() {
        let combine = BoxBiTransformer::new(|v1: Vec<i32>, v2: Vec<i32>| {
            let mut result = v1;
            result.extend(v2);
            result
        });
        assert_eq!(
            combine.transform(vec![1, 2, 3], vec![4, 5, 6]),
            vec![1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn test_arc_with_large_data() {
        let sum_vecs = ArcBiTransformer::new(|v1: Vec<i32>, v2: Vec<i32>| {
            v1.iter().sum::<i32>() + v2.iter().sum::<i32>()
        });
        let data1 = (1..=50).collect::<Vec<_>>();
        let data2 = (51..=100).collect::<Vec<_>>();
        assert_eq!(sum_vecs.transform(data1, data2), 5050);
    }

    #[test]
    fn test_with_tuples() {
        let swap = BoxBiTransformer::new(|x: i32, y: i32| (y, x));
        assert_eq!(swap.transform(1, 2), (2, 1));
    }

    #[test]
    fn test_string_operations() {
        let join = BoxBiTransformer::new(|s1: String, s2: String| format!("{} {}", s1, s2));
        assert_eq!(
            join.transform("Hello".to_string(), "World".to_string()),
            "Hello World"
        );
    }
}

// ============================================================================
// Type Conversion Tests - Testing into_box, into_rc, into_arc methods
// ============================================================================

#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_box_into_box() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.transform(10, 20), 30);
    }

    #[test]
    fn test_box_into_rc() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.transform(10, 20), 30);
    }

    #[test]
    fn test_arc_into_arc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let arc = add.into_arc();
        assert_eq!(arc.transform(10, 20), 30);
    }

    #[test]
    fn test_arc_into_fn() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.into_fn();
        assert_eq!(func(10, 20), 30);
    }

    #[test]
    fn test_rc_into_fn() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.into_fn();
        assert_eq!(func(10, 20), 30);
    }

    #[test]
    fn test_box_into_fn() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.into_fn();
        assert_eq!(func(10, 20), 30);
    }

    #[test]
    fn test_rc_into_rc() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.transform(10, 20), 30);
    }

    #[test]
    fn test_arc_into_box() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.transform(10, 20), 30);
    }

    #[test]
    fn test_arc_into_rc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.transform(10, 20), 30);
    }

    #[test]
    fn test_rc_into_box() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.transform(10, 20), 30);
    }
}

// ============================================================================
// Closure BiTransformer Tests - Testing blanket implementation for closures
// ============================================================================

#[cfg(test)]
mod closure_bi_transformer_tests {
    use super::*;

    #[test]
    fn test_closure_transform() {
        let add = |x: i32, y: i32| x + y;
        assert_eq!(add.transform(10, 20), 30);
    }

    #[test]
    fn test_closure_transform_with_string() {
        let concat = |s1: String, s2: String| format!("{}{}", s1, s2);
        assert_eq!(
            concat.transform("Hello".to_string(), "World".to_string()),
            "HelloWorld"
        );
    }

    #[test]
    fn test_closure_into_box() {
        let add = |x: i32, y: i32| x + y;
        let boxed = add.into_box();
        assert_eq!(boxed.transform(10, 20), 30);
    }

    #[test]
    fn test_closure_into_rc() {
        let add = |x: i32, y: i32| x + y;
        let rc = add.into_rc();
        assert_eq!(rc.transform(10, 20), 30);
    }

    #[test]
    fn test_closure_into_fn() {
        let add = |x: i32, y: i32| x + y;
        let func = add.into_fn();
        assert_eq!(func(10, 20), 30);
    }

    #[test]
    fn test_function_pointer_transform() {
        fn multiply(x: i32, y: i32) -> i32 {
            x * y
        }
        assert_eq!(multiply.transform(6, 7), 42);
    }

    #[test]
    fn test_function_pointer_into_box() {
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        let boxed = add.into_box();
        assert_eq!(boxed.transform(10, 20), 30);
    }

    #[test]
    fn test_function_pointer_into_fn() {
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        let func = add.into_fn();
        assert_eq!(func(10, 20), 30);
    }

    #[test]
    fn test_closure_with_captured_variable() {
        let multiplier = 3;
        let multiply_by = move |x: i32, y: i32| (x + y) * multiplier;
        assert_eq!(multiply_by.transform(5, 5), 30);
    }

    #[test]
    fn test_closure_into_arc() {
        let add = |x: i32, y: i32| x + y;
        let arc = add.into_arc();
        assert_eq!(arc.transform(10, 20), 30);
    }
}
