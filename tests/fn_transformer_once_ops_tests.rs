/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for FnTransformerOnceOps extension trait

use prism3_function::{
    FnTransformerOnceOps,
    TransformerOnce,
};

#[cfg(test)]
mod fn_transformer_once_ops_tests {
    use super::*;

    #[test]
    fn test_and_then_with_closures() {
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let double = |x: i32| x * 2;

        let composed = parse.and_then(double);
        assert_eq!(composed.apply_once("21".to_string()), 42);
    }

    #[test]
    fn test_and_then_chain() {
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;

        let composed = parse.and_then(add_one).and_then(double);
        assert_eq!(composed.apply_once("5".to_string()), 12); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_compose_with_closures() {
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();

        let composed = to_string.compose(double);
        assert_eq!(composed.apply_once(21), "42");
    }

    #[test]
    fn test_compose_chain() {
        let triple = |x: i32| x * 3;
        let add_two = |x: i32| x + 2;
        let to_string = |x: i32| x.to_string();

        let composed = to_string.compose(triple).compose(add_two);
        assert_eq!(composed.apply_once(5), "21"); // ((5 + 2) * 3).to_string() = "21"
    }

    #[test]
    fn test_when_with_closure_predicate() {
        let double = |x: i32| x * 2;
        let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(conditional.apply_once(5), 10);
    }

    #[test]
    fn test_when_with_negative_value() {
        let double = |x: i32| x * 2;
        let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(conditional.apply_once(-5), 5);
    }

    #[test]
    fn test_when_with_identity_else() {
        let double = |x: i32| x * 2;
        let conditional = double.when(|x: &i32| *x > 10).or_else(|x: i32| x);

        assert_eq!(conditional.apply_once(20), 40);
    }

    #[test]
    fn test_when_with_identity_else_false_condition() {
        let double = |x: i32| x * 2;
        let conditional = double.when(|x: &i32| *x > 10).or_else(|x: i32| x);

        assert_eq!(conditional.apply_once(5), 5);
    }

    #[test]
    fn test_complex_composition() {
        // Complex composition: parse string, then if > 5 multiply by 2, otherwise multiply by 3, finally convert to string
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let double = |x: i32| x * 2;
        let triple = |x: i32| x * 3;
        let to_string = |x: i32| x.to_string();

        let composed = parse
            .and_then(double.when(|x: &i32| *x > 5).or_else(triple))
            .and_then(to_string);

        assert_eq!(composed.apply_once("10".to_string()), "20"); // 10 > 5, so 10 * 2 = 20
    }

    #[test]
    fn test_complex_composition_else_branch() {
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let double = |x: i32| x * 2;
        let triple = |x: i32| x * 3;
        let to_string = |x: i32| x.to_string();

        let composed = parse
            .and_then(double.when(|x: &i32| *x > 5).or_else(triple))
            .and_then(to_string);

        assert_eq!(composed.apply_once("3".to_string()), "9"); // 3 <= 5, so 3 * 3 = 9
    }

    #[test]
    fn test_function_pointer() {
        fn parse(s: String) -> i32 {
            s.parse().unwrap_or(0)
        }
        fn double(x: i32) -> i32 {
            x * 2
        }

        let composed = parse.and_then(double);
        assert_eq!(composed.apply_once("21".to_string()), 42);
    }

    #[test]
    fn test_mixed_closure_and_function_pointer() {
        fn parse(s: String) -> i32 {
            s.parse().unwrap_or(0)
        }

        let double = |x: i32| x * 2;
        let composed = parse.and_then(double);
        assert_eq!(composed.apply_once("21".to_string()), 42);
    }

    #[test]
    fn test_type_transformation() {
        let to_string = |x: i32| x.to_string();
        let get_length = |s: String| s.len();

        let composed = to_string.and_then(get_length);
        assert_eq!(composed.apply_once(12345), 5);
    }

    #[test]
    fn test_when_with_multiple_conditions() {
        let abs = |x: i32| x.abs();
        let double = |x: i32| x * 2;

        // If negative, take absolute value; otherwise double
        let transformer = abs.when(|x: &i32| *x < 0).or_else(double);

        assert_eq!(transformer.apply_once(-5), 5);
    }

    #[test]
    fn test_when_with_multiple_conditions_else_branch() {
        let abs = |x: i32| x.abs();
        let double = |x: i32| x * 2;

        let transformer = abs.when(|x: &i32| *x < 0).or_else(double);

        assert_eq!(transformer.apply_once(5), 10);
    }

    #[test]
    fn test_closure_capturing_environment() {
        let multiplier = 3;
        let multiply = move |x: i32| x * multiplier;
        let add_ten = |x: i32| x + 10;

        let composed = multiply.and_then(add_ten);
        assert_eq!(composed.apply_once(5), 25); // 5 * 3 + 10
    }

    #[test]
    fn test_consuming_string() {
        let owned = String::from("hello");
        let append = move |s: String| format!("{} {}", s, owned);
        let uppercase = |s: String| s.to_uppercase();

        let composed = append.and_then(uppercase);
        assert_eq!(composed.apply_once("world".to_string()), "WORLD HELLO");
    }

    #[test]
    fn test_parse_and_validate() {
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let validate = |x: i32| {
            if x > 0 {
                x
            } else {
                1
            }
        };

        let composed = parse.and_then(validate);
        assert_eq!(composed.apply_once("42".to_string()), 42);
    }

    #[test]
    fn test_parse_and_validate_negative() {
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let validate = |x: i32| {
            if x > 0 {
                x
            } else {
                1
            }
        };

        let composed = parse.and_then(validate);
        assert_eq!(composed.apply_once("-5".to_string()), 1);
    }
}
