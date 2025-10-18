/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BoxTransformerOnce, TransformerOnce};

// ============================================================================
// BoxTransformerOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_transformer_once_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));

        assert_eq!(parse.transform("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        assert_eq!(identity.transform(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.transform(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
        assert_eq!(pipeline.transform(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxTransformerOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.transform("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.transform(42), "42_suffix");
    }
}

// ============================================================================
// Conditional Transformer Once Tests
// ============================================================================

#[cfg(test)]
mod conditional_tests {
    use super::*;
    use prism3_function::BoxPredicate;

    #[test]
    fn test_when_or_else() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformerOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.transform(5), 10);
    }

    #[test]
    fn test_when_or_else_negative() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformerOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.transform(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.transform(5), 10);
        let result2 = BoxTransformerOnce::new(|x: i32| x * 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: i32| -x);
        assert_eq!(result2.transform(-5), 5);
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
        let double = |x: i32| x * 2;
        let boxed = double.into_box();
        assert_eq!(boxed.transform(21), 42);
    }

    #[test]
    fn test_box_to_fn() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }
}

// ============================================================================
// Complex Composition Tests
// ============================================================================

#[cfg(test)]
mod complex_composition_tests {
    use super::*;

    #[test]
    fn test_multiple_and_then() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();
        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.transform(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_compose() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let square = BoxTransformerOnce::new(|x: i32| x * x);
        let composed = square.compose(double).compose(add_one);
        assert_eq!(composed.transform(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_mixed_composition() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
        let double = |x: i32| x * 2;
        let to_string = |x: i32| format!("Result: {}", x);
        let pipeline = parse.and_then(double).and_then(to_string);
        assert_eq!(pipeline.transform("21".to_string()), "Result: 42");
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_identity_composition() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        let composed = double.and_then(|x| identity.transform(x));
        assert_eq!(composed.transform(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.transform(123), "hello");

        let constant2 = BoxTransformerOnce::constant("world");
        assert_eq!(constant2.transform(456), "world");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.transform("42".to_string()), Some(42));

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse2.transform("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse.transform("42".to_string()).is_ok());

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse2.transform("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split = BoxTransformerOnce::new(|s: String| {
            s.split(',').map(|s| s.to_string()).collect::<Vec<_>>()
        });
        assert_eq!(
            split.transform("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_consuming_ownership() {
        let vec = vec![1, 2, 3, 4, 5];
        let sum = BoxTransformerOnce::new(|v: Vec<i32>| v.iter().sum::<i32>());
        assert_eq!(sum.transform(vec), 15);
        // vec is consumed and cannot be used again
    }

    #[test]
    fn test_with_box() {
        let boxed = Box::new(42);
        let unbox = BoxTransformerOnce::new(|b: Box<i32>| *b);
        assert_eq!(unbox.transform(boxed), 42);
    }

    #[test]
    fn test_with_closure_capture() {
        let multiplier = 3;
        let multiply = BoxTransformerOnce::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.transform(7), 21);
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_transformer_once_trait() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.transform(x)
        }

        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_closure_as_transformer_once() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.transform(x)
        }

        let double = |x: i32| x * 2;
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_transformer_once<T, R, F: TransformerOnce<T, R>>(f: F, x: T) -> R {
            f.transform(x)
        }

        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        assert_eq!(apply_transformer_once(to_string, 42), "42");
    }
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_box_into_box() {
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let boxed = add.into_box();
        assert_eq!(boxed.transform(20), 30);
    }

    #[test]
    fn test_box_into_fn() {
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let func = add.into_fn();
        assert_eq!(func(20), 30);
    }
}
