/**
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
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
        assert_eq!(add.apply(10, 10), 20);
        assert_eq!(add.apply(5, 3), 8);
    }

    #[test]
    fn test_multiply() {
        let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
        assert_eq!(multiply.apply(6, 7), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxBiTransformer::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
        assert_eq!(constant.apply(789, 101), "hello");
    }

    #[test]
    fn test_with_string() {
        let concat = BoxBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));
        assert_eq!(
            concat.apply("hello".to_string(), "world".to_string()),
            "helloworld"
        );
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let weighted_sum =
            BoxBiTransformer::new(move |x: i32, y: i32| x * multiplier + y * multiplier);
        assert_eq!(weighted_sum.apply(2, 3), 15); // (2 * 3) + (3 * 3) = 15
    }

    #[test]
    fn test_different_types() {
        let format = BoxBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.apply("Alice".to_string(), 30), "Alice is 30");
    }

    #[test]
    fn test_with_option() {
        let safe_divide =
            BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });
        assert_eq!(safe_divide.apply(42, 2), Some(21));
        assert_eq!(safe_divide.apply(42, 0), None);
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
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_clone() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        assert_eq!(add.apply(20, 22), 42);
        assert_eq!(cloned.apply(20, 22), 42);
    }

    #[test]
    fn test_thread_safe() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        let handle = thread::spawn(move || cloned.apply(20, 22));

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcBiTransformer::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let multiply = ArcBiTransformer::new(|x: i32, y: i32| x * y);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let m = multiply.clone();
                thread::spawn(move || m.apply(i, i + 1))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(results, vec![0, 2, 6, 12]); // 0*1, 1*2, 2*3, 3*4
    }

    #[test]
    fn test_with_different_types() {
        let format = ArcBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.apply("Alice".to_string(), 30), "Alice is 30");
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
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_clone() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let cloned = add.clone();

        assert_eq!(add.apply(20, 22), 42);
        assert_eq!(cloned.apply(20, 22), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcBiTransformer::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let concat = RcBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));

        let func1 = concat.clone();
        let func2 = concat.clone();

        assert_eq!(
            concat.apply("hello".to_string(), "world".to_string()),
            "helloworld"
        );
        assert_eq!(func1.apply("foo".to_string(), "bar".to_string()), "foobar");
        assert_eq!(
            func2.apply("rust".to_string(), "lang".to_string()),
            "rustlang"
        );
    }

    #[test]
    fn test_with_different_types() {
        let format = RcBiTransformer::new(|name: String, age: i32| format!("{} is {}", name, age));
        assert_eq!(format.apply("Alice".to_string(), 30), "Alice is 30");
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

        assert_eq!(result.apply(5, 3), 8); // both positive, add
        assert_eq!(result.apply(-5, 3), -15); // not both positive, multiply
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);

        assert_eq!(result.apply(5, 3), 8);
        assert_eq!(result.apply(-5, 3), -15);
        assert_eq!(result.apply(0, 5), 0);
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

        assert_eq!(result.apply(5, 3), 8);
        assert_eq!(result.apply(-5, 3), -15);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);

        assert_eq!(result.apply(5, 3), 8);
        assert_eq!(result.apply(-5, 3), -15);
        assert_eq!(result.apply(0, 5), 0);
    }

    #[test]
    fn test_conditional_clone() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let cloned = conditional.clone();

        let result1 = conditional.or_else(|x: i32, y: i32| x * y);
        let result2 = cloned.or_else(|x: i32, y: i32| x * y);

        assert_eq!(result1.apply(5, 3), 8);
        assert_eq!(result2.apply(5, 3), 8);
        assert_eq!(result1.apply(-5, 3), -15);
        assert_eq!(result2.apply(-5, 3), -15);
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

        assert_eq!(result.apply(5, 3), 8);
        assert_eq!(result.apply(-5, 3), -15);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);

        assert_eq!(result.apply(5, 3), 8);
        assert_eq!(result.apply(-5, 3), -15);
        assert_eq!(result.apply(0, 5), 0);
    }

    #[test]
    fn test_conditional_clone() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let cloned = conditional.clone();

        let result1 = conditional.or_else(|x: i32, y: i32| x * y);
        let result2 = cloned.or_else(|x: i32, y: i32| x * y);

        assert_eq!(result1.apply(5, 3), 8);
        assert_eq!(result2.apply(5, 3), 8);
        assert_eq!(result1.apply(-5, 3), -15);
        assert_eq!(result2.apply(-5, 3), -15);
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
        assert_eq!(boxed.apply(20, 22), 42);
    }

    #[test]
    fn test_to_box_to_rc_to_arc_and_to_fn_on_references() {
        // closure reference conversions
        let add = |x: i32, y: i32| x + y;
        let b = add.to_box();
        assert_eq!(b.apply(1, 2), 3);

        let r = add.to_rc();
        assert_eq!(r.apply(3, 4), 7);

        // arc requires Send+Sync; use ArcBiTransformer
        let a = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let f = a.to_fn();
        assert_eq!(f(5, 6), 11);
    }

    #[test]
    fn test_closure_to_arc() {
        let add = |x: i32, y: i32| x + y;
        let arc = add.into_arc();
        assert_eq!(arc.apply(20, 22), 42);
    }

    #[test]
    fn test_closure_to_rc() {
        let add = |x: i32, y: i32| x + y;
        let rc = add.into_rc();
        assert_eq!(rc.apply(20, 22), 42);
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
        assert_eq!(rc.apply(20, 22), 42);
    }

    #[test]
    fn test_arc_to_box() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20, 22), 42);
    }

    #[test]
    fn test_arc_to_rc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.apply(20, 22), 42);
    }

    #[test]
    fn test_rc_to_box() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20, 22), 42);
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
            f.apply(x, y)
        }

        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        assert_eq!(apply_bi_transformer(&add, 20, 22), 42);
    }

    #[test]
    fn test_closure_as_bi_transformer() {
        fn apply_bi_transformer<F: BiTransformer<i32, i32, i32>>(f: &F, x: i32, y: i32) -> i32 {
            f.apply(x, y)
        }

        let add = |x: i32, y: i32| x + y;
        assert_eq!(apply_bi_transformer(&add, 20, 22), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_bi_transformer<T, U, R, F: BiTransformer<T, U, R>>(f: &F, x: T, y: U) -> R {
            f.apply(x, y)
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
        assert_eq!(constant.apply(123, 456), "hello");
        assert_eq!(constant.apply(789, 101), "hello");
    }

    #[test]
    fn test_with_option() {
        let safe_divide =
            BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });
        assert_eq!(safe_divide.apply(42, 2), Some(21));
        assert_eq!(safe_divide.apply(42, 0), None);
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
        assert_eq!(safe_divide.apply(42, 2), Ok(21));
        assert!(safe_divide.apply(42, 0).is_err());
    }

    #[test]
    fn test_with_vec() {
        let combine = BoxBiTransformer::new(|v1: Vec<i32>, v2: Vec<i32>| {
            let mut result = v1;
            result.extend(v2);
            result
        });
        assert_eq!(
            combine.apply(vec![1, 2, 3], vec![4, 5, 6]),
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
        assert_eq!(sum_vecs.apply(data1, data2), 5050);
    }

    #[test]
    fn test_with_tuples() {
        let swap = BoxBiTransformer::new(|x: i32, y: i32| (y, x));
        assert_eq!(swap.apply(1, 2), (2, 1));
    }

    #[test]
    fn test_string_operations() {
        let join = BoxBiTransformer::new(|s1: String, s2: String| format!("{} {}", s1, s2));
        assert_eq!(
            join.apply("Hello".to_string(), "World".to_string()),
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
        assert_eq!(boxed.apply(10, 20), 30);
    }

    #[test]
    fn test_box_into_rc() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.apply(10, 20), 30);
    }

    #[test]
    fn test_arc_into_arc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let arc = add.into_arc();
        assert_eq!(arc.apply(10, 20), 30);
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
        assert_eq!(rc.apply(10, 20), 30);
    }

    #[test]
    fn test_arc_into_box() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(10, 20), 30);
    }

    #[test]
    fn test_arc_into_rc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.apply(10, 20), 30);
    }

    #[test]
    fn test_rc_into_box() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(10, 20), 30);
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
        assert_eq!(add.apply(10, 20), 30);
    }

    #[test]
    fn test_closure_transform_with_string() {
        let concat = |s1: String, s2: String| format!("{}{}", s1, s2);
        assert_eq!(
            concat.apply("Hello".to_string(), "World".to_string()),
            "HelloWorld"
        );
    }

    #[test]
    fn test_closure_into_box() {
        let add = |x: i32, y: i32| x + y;
        let boxed = add.into_box();
        assert_eq!(boxed.apply(10, 20), 30);
    }

    #[test]
    fn test_closure_into_rc() {
        let add = |x: i32, y: i32| x + y;
        let rc = add.into_rc();
        assert_eq!(rc.apply(10, 20), 30);
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
        assert_eq!(multiply.apply(6, 7), 42);
    }

    #[test]
    fn test_function_pointer_into_box() {
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        let boxed = add.into_box();
        assert_eq!(boxed.apply(10, 20), 30);
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
        assert_eq!(multiply_by.apply(5, 5), 30);
    }

    #[test]
    fn test_closure_into_arc() {
        let add = |x: i32, y: i32| x + y;
        let arc = add.into_arc();
        assert_eq!(arc.apply(10, 20), 30);
    }
}

// ============================================================================
// Custom BiTransformer Tests - Testing default into_xxx() implementations
// ============================================================================

#[cfg(test)]
mod custom_bi_transformer_tests {
    use super::*;

    /// Custom BiTransformer implementation for testing default into_xxx() methods
    struct CustomBiTransformer {
        multiplier: i32,
    }

    impl CustomBiTransformer {
        fn new(multiplier: i32) -> Self {
            Self { multiplier }
        }
    }

    impl BiTransformer<i32, i32, i32> for CustomBiTransformer {
        fn apply(&self, first: i32, second: i32) -> i32 {
            (first + second) * self.multiplier
        }
    }

    #[test]
    fn test_custom_bi_transformer_apply() {
        let transformer = CustomBiTransformer::new(3);
        assert_eq!(transformer.apply(5, 10), 45); // (5 + 10) * 3 = 45
    }

    #[test]
    fn test_custom_bi_transformer_into_box() {
        let transformer = CustomBiTransformer::new(2);
        let boxed = transformer.into_box();
        assert_eq!(boxed.apply(10, 20), 60); // (10 + 20) * 2 = 60
        assert_eq!(boxed.apply(5, 5), 20); // (5 + 5) * 2 = 20
    }

    #[test]
    fn test_custom_bi_transformer_into_rc() {
        let transformer = CustomBiTransformer::new(4);
        let rc = transformer.into_rc();
        assert_eq!(rc.apply(3, 7), 40); // (3 + 7) * 4 = 40

        // Test cloning
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(2, 3), 20); // (2 + 3) * 4 = 20
        assert_eq!(rc.apply(1, 1), 8); // (1 + 1) * 4 = 8
    }

    #[test]
    fn test_custom_bi_transformer_into_fn() {
        let transformer = CustomBiTransformer::new(5);
        let func = transformer.into_fn();
        assert_eq!(func(4, 6), 50); // (4 + 6) * 5 = 50
        assert_eq!(func(1, 1), 10); // (1 + 1) * 5 = 10
    }

    /// Custom Send + Sync BiTransformer implementation
    struct ThreadSafeBiTransformer {
        multiplier: i32,
    }

    impl ThreadSafeBiTransformer {
        fn new(multiplier: i32) -> Self {
            Self { multiplier }
        }
    }

    impl BiTransformer<i32, i32, i32> for ThreadSafeBiTransformer {
        fn apply(&self, first: i32, second: i32) -> i32 {
            (first + second) * self.multiplier
        }
    }

    // Manual implementation of Send and Sync
    unsafe impl Send for ThreadSafeBiTransformer {}
    unsafe impl Sync for ThreadSafeBiTransformer {}

    #[test]
    fn test_custom_bi_transformer_into_arc() {
        let transformer = ThreadSafeBiTransformer::new(3);
        let arc = transformer.into_arc();
        assert_eq!(arc.apply(10, 5), 45); // (10 + 5) * 3 = 45

        // Test cloning
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.apply(2, 8), 30); // (2 + 8) * 3 = 30

        // Test cross-thread usage
        let arc_thread = arc.clone();
        let handle = thread::spawn(move || arc_thread.apply(3, 7));
        assert_eq!(handle.join().unwrap(), 30); // (3 + 7) * 3 = 30

        // Original arc still usable
        assert_eq!(arc.apply(1, 1), 6); // (1 + 1) * 3 = 6
    }

    #[test]
    fn test_custom_bi_transformer_chaining() {
        let transformer = CustomBiTransformer::new(2);
        let boxed = transformer.into_box();

        // Test multiple calls
        assert_eq!(boxed.apply(5, 10), 30); // (5 + 10) * 2 = 30
        assert_eq!(boxed.apply(3, 7), 20); // (3 + 7) * 2 = 20
        assert_eq!(boxed.apply(1, 1), 4); // (1 + 1) * 2 = 4
    }

    /// Test custom BiTransformer with different types combination
    struct StringCombiner {
        separator: String,
    }

    impl StringCombiner {
        fn new(separator: &str) -> Self {
            Self {
                separator: separator.to_string(),
            }
        }
    }

    impl BiTransformer<String, String, String> for StringCombiner {
        fn apply(&self, first: String, second: String) -> String {
            format!("{}{}{}", first, self.separator, second)
        }
    }

    #[test]
    fn test_custom_string_bi_transformer_into_box() {
        let combiner = StringCombiner::new(" - ");
        let boxed = combiner.into_box();
        assert_eq!(
            boxed.apply("Hello".to_string(), "World".to_string()),
            "Hello - World"
        );
        assert_eq!(
            boxed.apply("Rust".to_string(), "Language".to_string()),
            "Rust - Language"
        );
    }

    #[test]
    fn test_custom_string_bi_transformer_into_rc() {
        let combiner = StringCombiner::new(" + ");
        let rc = combiner.into_rc();

        assert_eq!(rc.apply("A".to_string(), "B".to_string()), "A + B");

        // Clone and use
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply("X".to_string(), "Y".to_string()), "X + Y");
        assert_eq!(rc.apply("1".to_string(), "2".to_string()), "1 + 2");
    }

    #[test]
    fn test_custom_string_bi_transformer_into_fn() {
        let combiner = StringCombiner::new(" & ");
        let func = combiner.into_fn();

        assert_eq!(func("Cat".to_string(), "Dog".to_string()), "Cat & Dog");
        assert_eq!(func("One".to_string(), "Two".to_string()), "One & Two");
    }

    /// Test custom BiTransformer's default to_xxx() implementations
    /// These are default implementations provided by the BiTransformer trait, requiring the type to implement Clone
    #[derive(Clone)]
    struct CloneableCustomBiTransformer {
        multiplier: i32,
    }

    impl CloneableCustomBiTransformer {
        fn new(multiplier: i32) -> Self {
            Self { multiplier }
        }
    }

    impl BiTransformer<i32, i32, i32> for CloneableCustomBiTransformer {
        fn apply(&self, first: i32, second: i32) -> i32 {
            (first + second) * self.multiplier
        }
    }

    #[test]
    fn test_custom_bi_transformer_default_to_box() {
        // Test the default to_box() implementation provided by BiTransformer trait
        let transformer = CloneableCustomBiTransformer::new(3);
        let boxed = transformer.to_box();
        assert_eq!(boxed.apply(5, 10), 45); // (5 + 10) * 3 = 45

        // Original transformer still usable
        assert_eq!(transformer.apply(2, 3), 15); // (2 + 3) * 3 = 15
    }

    #[test]
    fn test_custom_bi_transformer_default_to_rc() {
        // Test the default to_rc() implementation provided by BiTransformer trait
        let transformer = CloneableCustomBiTransformer::new(4);
        let rc = transformer.to_rc();
        assert_eq!(rc.apply(3, 7), 40); // (3 + 7) * 4 = 40

        // Original transformer still usable
        assert_eq!(transformer.apply(1, 1), 8); // (1 + 1) * 4 = 8

        // Test rc cloning
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(2, 3), 20); // (2 + 3) * 4 = 20
    }

    #[test]
    fn test_custom_bi_transformer_default_to_fn() {
        // Test the default to_fn() implementation provided by BiTransformer trait
        let transformer = CloneableCustomBiTransformer::new(5);
        let func = transformer.to_fn();
        assert_eq!(func(4, 6), 50); // (4 + 6) * 5 = 50

        // Original transformer still usable
        assert_eq!(transformer.apply(1, 1), 10); // (1 + 1) * 5 = 10
    }

    /// Test custom Send + Sync BiTransformer's default to_arc() implementation
    #[derive(Clone)]
    struct ThreadSafeCloneableBiTransformer {
        multiplier: i32,
    }

    impl ThreadSafeCloneableBiTransformer {
        fn new(multiplier: i32) -> Self {
            Self { multiplier }
        }
    }

    impl BiTransformer<i32, i32, i32> for ThreadSafeCloneableBiTransformer {
        fn apply(&self, first: i32, second: i32) -> i32 {
            (first + second) * self.multiplier
        }
    }

    // Manual implementation of Send and Sync
    unsafe impl Send for ThreadSafeCloneableBiTransformer {}
    unsafe impl Sync for ThreadSafeCloneableBiTransformer {}

    #[test]
    fn test_custom_bi_transformer_default_to_arc() {
        // Test the default to_arc() implementation provided by BiTransformer trait
        let transformer = ThreadSafeCloneableBiTransformer::new(3);
        let arc = transformer.to_arc();
        assert_eq!(arc.apply(10, 5), 45); // (10 + 5) * 3 = 45

        // Original transformer still usable
        assert_eq!(transformer.apply(2, 2), 12); // (2 + 2) * 3 = 12

        // Test arc cloning
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.apply(3, 7), 30); // (3 + 7) * 3 = 30

        // Test cross-thread usage
        let arc_thread = arc.clone();
        let handle = thread::spawn(move || arc_thread.apply(4, 6));
        assert_eq!(handle.join().unwrap(), 30); // (4 + 6) * 3 = 30

        // Original arc still usable
        assert_eq!(arc.apply(1, 1), 6); // (1 + 1) * 3 = 6
    }

    #[test]
    fn test_custom_bi_transformer_all_default_to_methods() {
        // Test all default to_xxx() methods
        let transformer = CloneableCustomBiTransformer::new(2);

        let boxed = transformer.to_box();
        let rc = transformer.to_rc();
        let func = transformer.to_fn();

        assert_eq!(boxed.apply(5, 10), 30); // (5 + 10) * 2 = 30
        assert_eq!(rc.apply(3, 7), 20); // (3 + 7) * 2 = 20
        assert_eq!(func(2, 3), 10); // (2 + 3) * 2 = 10

        // Original transformer still usable
        assert_eq!(transformer.apply(1, 1), 4); // (1 + 1) * 2 = 4
    }

    /// Test custom string type's default to_xxx() methods
    #[derive(Clone)]
    struct CloneableStringCombiner {
        separator: String,
    }

    impl CloneableStringCombiner {
        fn new(separator: &str) -> Self {
            Self {
                separator: separator.to_string(),
            }
        }
    }

    impl BiTransformer<String, String, String> for CloneableStringCombiner {
        fn apply(&self, first: String, second: String) -> String {
            format!("{}{}{}", first, self.separator, second)
        }
    }

    #[test]
    fn test_custom_string_bi_transformer_default_to_box() {
        let combiner = CloneableStringCombiner::new(" - ");
        let boxed = combiner.to_box();
        assert_eq!(
            boxed.apply("Hello".to_string(), "World".to_string()),
            "Hello - World"
        );

        // Original still usable
        assert_eq!(
            combiner.apply("Rust".to_string(), "Lang".to_string()),
            "Rust - Lang"
        );
    }

    #[test]
    fn test_custom_string_bi_transformer_default_to_rc() {
        let combiner = CloneableStringCombiner::new(" + ");
        let rc = combiner.to_rc();
        assert_eq!(rc.apply("A".to_string(), "B".to_string()), "A + B");

        // Original still usable
        assert_eq!(combiner.apply("X".to_string(), "Y".to_string()), "X + Y");

        // Clone and use
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply("1".to_string(), "2".to_string()), "1 + 2");
    }

    #[test]
    fn test_custom_string_bi_transformer_default_to_fn() {
        let combiner = CloneableStringCombiner::new(" & ");
        let func = combiner.to_fn();
        assert_eq!(func("Cat".to_string(), "Dog".to_string()), "Cat & Dog");

        // Original still usable
        assert_eq!(
            combiner.apply("One".to_string(), "Two".to_string()),
            "One & Two"
        );
    }
}

// ============================================================================
// BiTransformer Default Methods - to_xxx() Non-consuming Conversions
// ============================================================================

// Note: BoxBiTransformer does not implement Clone, so to_xxx() methods are not available
// The to_xxx() methods require Clone trait bound, which BoxBiTransformer intentionally
// does not implement to maintain single ownership semantics.

#[cfg(test)]
mod box_bi_transformer_to_methods_tests {
    use super::*;

    #[test]
    fn test_box_into_box() {
        // BoxBiTransformer::into_box() returns itself directly (zero-cost operation)
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add.into_box();
        assert_eq!(result.apply(10, 20), 30);
    }

    #[test]
    fn test_box_into_rc() {
        // BoxBiTransformer can be converted to RcBiTransformer
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.apply(10, 20), 30);

        // Test cloning Rc result
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(5, 5), 10);
    }

    #[test]
    fn test_box_into_fn() {
        // BoxBiTransformer can be converted to a closure
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.into_fn();
        assert_eq!(func(10, 20), 30);
    }
}

#[cfg(test)]
mod arc_bi_transformer_to_methods_tests {
    use super::*;

    #[test]
    fn test_arc_to_arc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let arc2 = add.to_arc();
        assert_eq!(arc2.apply(10, 20), 30);
        // Original add still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_arc_to_box() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.to_box();
        assert_eq!(boxed.apply(10, 20), 30);
        // Original add still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_arc_to_rc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.to_rc();
        assert_eq!(rc.apply(10, 20), 30);
        // Original add still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_arc_to_fn() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.to_fn();
        assert_eq!(func(10, 20), 30);
        // Original add still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_arc_to_multiple_conversions_shared() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);

        let box1 = add.to_box();
        let rc1 = add.to_rc();
        let arc2 = add.to_arc();
        let fn_ref = add.to_fn();

        assert_eq!(box1.apply(1, 2), 3);
        assert_eq!(rc1.apply(2, 3), 5);
        assert_eq!(arc2.apply(3, 4), 7);
        assert_eq!(fn_ref(4, 5), 9);

        // Original still usable
        assert_eq!(add.apply(5, 5), 10);
    }
}

#[cfg(test)]
mod rc_bi_transformer_to_methods_tests {
    use super::*;

    #[test]
    fn test_rc_to_rc() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc2 = add.to_rc();
        assert_eq!(rc2.apply(10, 20), 30);
        // Original add still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_rc_to_box() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.to_box();
        assert_eq!(boxed.apply(10, 20), 30);
        // Original add still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_rc_to_fn() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let func = add.to_fn();
        assert_eq!(func(10, 20), 30);
        // Original add still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_rc_to_multiple_conversions_shared() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);

        let box1 = add.to_box();
        let rc2 = add.to_rc();
        let fn_ref = add.to_fn();

        assert_eq!(box1.apply(1, 2), 3);
        assert_eq!(rc2.apply(2, 3), 5);
        assert_eq!(fn_ref(4, 5), 9);

        // Original still usable
        assert_eq!(add.apply(5, 5), 10);
    }
}

// ============================================================================
// Closure BiTransformer to_xxx() Methods Tests
// ============================================================================

#[cfg(test)]
mod closure_bi_transformer_to_methods_tests {
    use super::*;

    #[test]
    fn test_closure_to_box() {
        let add = |x: i32, y: i32| x + y;
        let boxed = add.to_box();
        assert_eq!(boxed.apply(10, 20), 30);
        // Original closure still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_closure_to_rc() {
        let add = |x: i32, y: i32| x + y;
        let rc = add.to_rc();
        assert_eq!(rc.apply(10, 20), 30);
        // Original closure still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_closure_to_arc() {
        let add = |x: i32, y: i32| x + y;
        let arc = add.to_arc();
        assert_eq!(arc.apply(10, 20), 30);
        // Original closure still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_closure_to_fn() {
        let add = |x: i32, y: i32| x + y;
        let func = add.to_fn();
        assert_eq!(func(10, 20), 30);
        // Original closure still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_function_pointer_to_box() {
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        let boxed = add.to_box();
        assert_eq!(boxed.apply(10, 20), 30);
        // Original function pointer still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_function_pointer_to_rc() {
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        let rc = add.to_rc();
        assert_eq!(rc.apply(10, 20), 30);
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_function_pointer_to_arc() {
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        let arc = add.to_arc();
        assert_eq!(arc.apply(10, 20), 30);
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_function_pointer_to_fn() {
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        let func = add.to_fn();
        assert_eq!(func(10, 20), 30);
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_closure_to_multiple_conversions() {
        let add = |x: i32, y: i32| x + y;

        let box1 = add.to_box();
        let rc1 = add.to_rc();
        let arc1 = add.to_arc();
        let fn_ref = add.to_fn();

        assert_eq!(box1.apply(1, 2), 3);
        assert_eq!(rc1.apply(2, 3), 5);
        assert_eq!(arc1.apply(3, 4), 7);
        assert_eq!(fn_ref(4, 5), 9);

        // Original still usable
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_closure_with_capture_to_conversions() {
        let multiplier = 2;
        let multiply = move |x: i32, y: i32| (x + y) * multiplier;

        let boxed = multiply.to_box();
        let rc = multiply.to_rc();

        assert_eq!(boxed.apply(5, 5), 20); // (5 + 5) * 2
        assert_eq!(rc.apply(3, 7), 20); // (3 + 7) * 2
    }
}

// ============================================================================
// Complete to_xxx() Test Coverage for All Types
// ============================================================================

#[cfg(test)]
mod complete_to_methods_coverage {
    use super::*;

    #[test]
    fn test_arc_all_conversions() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);

        let to_box = add.to_box();
        let to_rc = add.to_rc();
        let to_arc = add.to_arc();
        let to_fn = add.to_fn();

        assert_eq!(to_box.apply(2, 3), 5);
        assert_eq!(to_rc.apply(2, 3), 5);
        assert_eq!(to_arc.apply(2, 3), 5);
        assert_eq!(to_fn(2, 3), 5);

        // Original still usable
        assert_eq!(add.apply(2, 3), 5);
    }

    #[test]
    fn test_rc_all_conversions() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);

        let to_box = add.to_box();
        let to_rc = add.to_rc();
        let to_fn = add.to_fn();

        assert_eq!(to_box.apply(2, 3), 5);
        assert_eq!(to_rc.apply(2, 3), 5);
        assert_eq!(to_fn(2, 3), 5);

        // Original still usable
        assert_eq!(add.apply(2, 3), 5);
    }

    #[test]
    fn test_closure_all_conversions() {
        let add = |x: i32, y: i32| x + y;

        let to_box = add.to_box();
        let to_rc = add.to_rc();
        let to_arc = add.to_arc();
        let to_fn = add.to_fn();

        assert_eq!(to_box.apply(2, 3), 5);
        assert_eq!(to_rc.apply(2, 3), 5);
        assert_eq!(to_arc.apply(2, 3), 5);
        assert_eq!(to_fn(2, 3), 5);

        // Original still usable
        assert_eq!(add.apply(2, 3), 5);
    }
}

// ============================================================================
// Consuming into_xxx() and Non-consuming to_xxx() Comparison Tests
// ============================================================================

#[cfg(test)]
mod into_vs_to_comparison_tests {
    use super::*;

    #[test]
    fn test_box_into_vs_to_box() {
        // into_box: consumes ownership, gets same type
        let add1 = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed1 = add1.into_box();
        assert_eq!(boxed1.apply(10, 20), 30);
        // add1 no longer usable (moved)

        // to_box: non-consuming, original still usable (but BoxBiTransformer does not implement to_xxx)
        // BoxBiTransformer does not implement Clone, so does not support to_xxx() methods
    }

    #[test]
    fn test_arc_into_vs_to_arc() {
        // into_arc: consumes ownership, gets same type
        let add1 = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let arc1 = add1.into_arc();
        assert_eq!(arc1.apply(10, 20), 30);
        // add1 no longer usable (moved)

        // to_arc: non-consuming, original still usable
        let add2 = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let arc2 = add2.to_arc();
        assert_eq!(arc2.apply(10, 20), 30);
        // add2 still usable
        assert_eq!(add2.apply(5, 5), 10);
    }

    #[test]
    fn test_rc_into_vs_to_rc() {
        // into_rc: consumes ownership, gets same type
        let add1 = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc1 = add1.into_rc();
        assert_eq!(rc1.apply(10, 20), 30);
        // add1 no longer usable (moved)

        // to_rc: non-consuming, original still usable
        let add2 = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc2 = add2.to_rc();
        assert_eq!(rc2.apply(10, 20), 30);
        // add2 still usable
        assert_eq!(add2.apply(5, 5), 10);
    }

    #[test]
    fn test_closure_into_vs_to() {
        // into_xxx: consumes closure
        let add1 = |x: i32, y: i32| x + y;
        let boxed1 = add1.into_box();
        assert_eq!(boxed1.apply(10, 20), 30);
        // add1 no longer usable (moved)

        // to_xxx: non-consuming, original still usable
        let add2 = |x: i32, y: i32| x + y;
        let boxed2 = add2.to_box();
        assert_eq!(boxed2.apply(10, 20), 30);
        // add2 still usable
        assert_eq!(add2.apply(5, 5), 10);
    }
}

// ============================================================================
// BiTransformer Default Trait Methods - into_xxx() with Various Inputs
// ============================================================================

#[cfg(test)]
mod into_methods_comprehensive_tests {
    use super::*;

    #[test]
    fn test_box_into_box_same_type() {
        // into_box on BoxBiTransformer should directly return itself
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add.into_box();
        assert_eq!(result.apply(3, 4), 7);
    }

    #[test]
    fn test_box_into_rc() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.apply(3, 4), 7);

        // Test cloning Rc result
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(1, 2), 3);
    }

    #[test]
    fn test_arc_into_arc_same_type() {
        // into_arc on ArcBiTransformer should directly return itself
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add.into_arc();
        assert_eq!(result.apply(3, 4), 7);
    }

    #[test]
    fn test_arc_into_box() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(3, 4), 7);
    }

    #[test]
    fn test_arc_into_rc() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        assert_eq!(rc.apply(3, 4), 7);
    }

    #[test]
    fn test_rc_into_rc_same_type() {
        // into_rc on RcBiTransformer should directly return itself
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let result = add.into_rc();
        assert_eq!(result.apply(3, 4), 7);
    }

    #[test]
    fn test_rc_into_box() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(3, 4), 7);
    }

    #[test]
    fn test_custom_into_all_types() {
        struct CustomBiTransformer {
            value: i32,
        }

        impl BiTransformer<i32, i32, i32> for CustomBiTransformer {
            fn apply(&self, first: i32, second: i32) -> i32 {
                first + second + self.value
            }
        }

        // into_box
        let ct1 = CustomBiTransformer { value: 10 };
        let boxed = ct1.into_box();
        assert_eq!(boxed.apply(1, 2), 13); // 1 + 2 + 10

        // into_rc
        let ct2 = CustomBiTransformer { value: 20 };
        let rc = ct2.into_rc();
        assert_eq!(rc.apply(1, 2), 23); // 1 + 2 + 20

        // into_fn
        let ct3 = CustomBiTransformer { value: 30 };
        let func = ct3.into_fn();
        assert_eq!(func(1, 2), 33); // 1 + 2 + 30
    }
}

// ============================================================================
// Type Conversion Chain Tests
// ============================================================================

#[cfg(test)]
mod conversion_chain_tests {
    use super::*;

    #[test]
    fn test_box_to_rc_to_box_chain() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let rc = add.into_rc();
        let rc_clone = rc.clone();
        let boxed = rc_clone.into_box();
        assert_eq!(boxed.apply(5, 3), 8);
    }

    #[test]
    fn test_arc_to_box_to_rc_chain() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        let rc = boxed.into_rc();
        assert_eq!(rc.apply(5, 3), 8);
    }

    #[test]
    fn test_rc_to_arc_chain() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let _boxed = add.into_box();
        // Note: BoxBiTransformer is not Send+Sync, so cannot convert to ArcBiTransformer
        // Test with ArcBiTransformer instead
        let arc_add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let boxed_arc = arc_add.into_box();
        let rc_arc = boxed_arc.into_rc();
        assert_eq!(rc_arc.apply(5, 3), 8);
    }

    #[test]
    fn test_closure_multiple_conversions_chain() {
        let add = |x: i32, y: i32| x + y;

        // Chain conversion
        let boxed = add.into_box();
        let rc = boxed.into_rc();
        let rc_clone = rc.clone();
        let boxed2 = rc_clone.into_box();

        assert_eq!(boxed2.apply(5, 3), 8);
    }
}

// ============================================================================
// String and Complex Types Conversion Tests
// ============================================================================

#[cfg(test)]
mod complex_types_conversion_tests {
    use super::*;

    #[test]
    fn test_string_concat_to_box() {
        let concat = |s1: String, s2: String| format!("{}{}", s1, s2);
        let boxed = concat.to_box();

        assert_eq!(
            boxed.apply("Hello".to_string(), "World".to_string()),
            "HelloWorld"
        );
        // Original still usable
        assert_eq!(concat.apply("Foo".to_string(), "Bar".to_string()), "FooBar");
    }

    #[test]
    fn test_string_concat_to_rc() {
        let concat = |s1: String, s2: String| format!("{}{}", s1, s2);
        let rc = concat.to_rc();

        assert_eq!(
            rc.apply("Hello".to_string(), "World".to_string()),
            "HelloWorld"
        );

        let rc_clone = rc.clone();
        assert_eq!(
            rc_clone.apply("Foo".to_string(), "Bar".to_string()),
            "FooBar"
        );
    }

    #[test]
    fn test_vec_combine_to_box() {
        let combine = |v1: Vec<i32>, v2: Vec<i32>| {
            let mut result = v1;
            result.extend(v2);
            result
        };

        let boxed = combine.to_box();
        assert_eq!(boxed.apply(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);

        // Original still usable
        assert_eq!(combine.apply(vec![5], vec![6]), vec![5, 6]);
    }

    #[test]
    fn test_option_safe_divide_to_rc() {
        let safe_divide = |x: i32, y: i32| if y == 0 { None } else { Some(x / y) };
        let rc = safe_divide.to_rc();

        assert_eq!(rc.apply(10, 2), Some(5));
        assert_eq!(rc.apply(10, 0), None);

        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(20, 4), Some(5));
    }
}

// ============================================================================
// Send+Sync Verification Tests for Arc Conversions
// ============================================================================

#[cfg(test)]
mod arc_thread_safety_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_arc_to_arc_thread_safe() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let arc2 = add.to_arc();

        let handle = thread::spawn(move || arc2.apply(10, 20));

        assert_eq!(handle.join().unwrap(), 30);
        assert_eq!(add.apply(5, 5), 10);
    }

    #[test]
    fn test_closure_to_arc_thread_safe() {
        let add = |x: i32, y: i32| x + y;
        let arc = add.to_arc();

        let handle = thread::spawn(move || arc.apply(10, 20));

        assert_eq!(handle.join().unwrap(), 30);
    }

    #[test]
    fn test_arc_into_arc_thread_safe() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let arc = add.into_arc();

        let handle = thread::spawn(move || arc.apply(10, 20));

        assert_eq!(handle.join().unwrap(), 30);
    }
}
