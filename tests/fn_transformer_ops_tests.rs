/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for FnTransformerOps extension trait

use prism3_function::{FnTransformerOps, Transformer};

#[cfg(test)]
mod fn_transformer_ops_tests {
    use super::*;

    #[test]
    fn test_and_then_with_closures() {
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();

        let composed = double.and_then(to_string);
        assert_eq!(composed.apply(21), "42");
    }

    #[test]
    fn test_and_then_chain() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();

        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.apply(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_compose_with_closures() {
        let double = |x: i32| x * 2;
        let add_one = |x: i32| x + 1;

        let composed = double.compose(add_one);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose_chain() {
        let triple = |x: i32| x * 3;
        let add_two = |x: i32| x + 2;
        let subtract_one = |x: i32| x - 1;

        let composed = triple.compose(add_two).compose(subtract_one);
        assert_eq!(composed.apply(5), 18); // ((5 - 1) + 2) * 3 = 18
    }

    #[test]
    fn test_when_with_closure_predicate() {
        let double = |x: i32| x * 2;
        let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(conditional.apply(5), 10);
        assert_eq!(conditional.apply(-5), 5);
        assert_eq!(conditional.apply(0), 0);
    }

    #[test]
    fn test_when_with_identity_else() {
        let double = |x: i32| x * 2;
        let conditional = double.when(|x: &i32| *x > 10).or_else(|x: i32| x);

        assert_eq!(conditional.apply(20), 40);
        assert_eq!(conditional.apply(5), 5);
    }

    #[test]
    fn test_complex_composition() {
        // Complex composition: add 1, then if > 5 multiply by 2, otherwise multiply by 3, finally convert to string
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let triple = |x: i32| x * 3;
        let to_string = |x: i32| x.to_string();

        let composed = add_one
            .and_then(double.when(|x: &i32| *x > 5).or_else(triple))
            .and_then(to_string);

        assert_eq!(composed.apply(5), "12"); // (5 + 1) = 6 > 5, so 6 * 2 = 12
        assert_eq!(composed.apply(1), "6"); // (1 + 1) = 2 <= 5, so 2 * 3 = 6
        assert_eq!(composed.apply(10), "22"); // (10 + 1) = 11 > 5, so 11 * 2 = 22
    }

    #[test]
    fn test_function_pointer() {
        fn double(x: i32) -> i32 {
            x * 2
        }
        fn add_one(x: i32) -> i32 {
            x + 1
        }

        let composed = double.and_then(add_one);
        assert_eq!(composed.apply(5), 11); // 5 * 2 + 1
    }

    #[test]
    fn test_mixed_closure_and_function_pointer() {
        fn double(x: i32) -> i32 {
            x * 2
        }

        let add_one = |x: i32| x + 1;
        let composed = double.and_then(add_one);
        assert_eq!(composed.apply(5), 11); // 5 * 2 + 1
    }

    #[test]
    fn test_type_transformation() {
        let to_string = |x: i32| x.to_string();
        let get_length = |s: String| s.len();

        let composed = to_string.and_then(get_length);
        assert_eq!(composed.apply(12345), 5);
    }

    #[test]
    fn test_when_with_multiple_conditions() {
        let abs = |x: i32| x.abs();
        let double = |x: i32| x * 2;

        // If negative, take absolute value; otherwise double
        let transformer = abs.when(|x: &i32| *x < 0).or_else(double);

        assert_eq!(transformer.apply(-5), 5);
        assert_eq!(transformer.apply(5), 10);
        assert_eq!(transformer.apply(0), 0);
    }

    #[test]
    fn test_closure_capturing_environment() {
        let multiplier = 3;
        let multiply = move |x: i32| x * multiplier;
        let add_ten = |x: i32| x + 10;

        let composed = multiply.and_then(add_ten);
        assert_eq!(composed.apply(5), 25); // 5 * 3 + 10
    }
}
