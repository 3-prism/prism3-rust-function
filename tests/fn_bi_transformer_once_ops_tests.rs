/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BiTransformerOnce, FnBiTransformerOnceOps};

#[cfg(test)]
mod fn_bi_transformer_once_ops_tests {
    use super::*;

    #[test]
    fn test_closure_and_then() {
        // Test closure's and_then method
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        let composed = add.and_then(double);
        assert_eq!(composed.apply_once(3, 5), 16); // (3 + 5) * 2 = 16
    }

    #[test]
    fn test_closure_and_then_with_type_conversion() {
        // Test and_then with type conversion
        let add = |x: i32, y: i32| x + y;
        let to_string = |x: i32| x.to_string();

        let composed = add.and_then(to_string);
        assert_eq!(composed.apply_once(20, 22), "42");
    }

    #[test]
    fn test_closure_when_with_or_else() {
        // Test closure's when method
        let add = |x: i32, y: i32| x + y;
        let multiply = |x: i32, y: i32| x * y;

        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply);
        assert_eq!(conditional.apply_once(5, 3), 8); // Condition met, execute addition

        // Need to recreate because it's FnOnce
        let add2 = |x: i32, y: i32| x + y;
        let multiply2 = |x: i32, y: i32| x * y;
        let conditional2 = add2
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply2);
        assert_eq!(conditional2.apply_once(-5, 3), -15); // Condition not met, execute multiplication
    }

    #[test]
    fn test_closure_when_with_single_condition() {
        // Test when with single condition
        let add = |x: i32, y: i32| x + y;
        let subtract = |x: i32, y: i32| x - y;

        let conditional = add.when(|x: &i32, _y: &i32| *x > 0).or_else(subtract);
        assert_eq!(conditional.apply_once(10, 3), 13); // x > 0, execute addition

        let add2 = |x: i32, y: i32| x + y;
        let subtract2 = |x: i32, y: i32| x - y;
        let conditional2 = add2.when(|x: &i32, _y: &i32| *x > 0).or_else(subtract2);
        assert_eq!(conditional2.apply_once(-10, 3), -13); // x <= 0, execute subtraction
    }

    #[test]
    fn test_function_pointer_and_then() {
        // Test function pointer's and_then
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        fn double(x: i32) -> i32 {
            x * 2
        }

        let composed = add.and_then(double);
        assert_eq!(composed.apply_once(3, 5), 16);
    }

    #[test]
    fn test_function_pointer_when() {
        // Test function pointer's when
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        fn multiply(x: i32, y: i32) -> i32 {
            x * y
        }
        fn both_positive(x: &i32, y: &i32) -> bool {
            *x > 0 && *y > 0
        }

        let conditional = add.when(both_positive).or_else(multiply);
        assert_eq!(conditional.apply_once(5, 3), 8);

        let conditional2 = add.when(both_positive).or_else(multiply);
        assert_eq!(conditional2.apply_once(-5, 3), -15);
    }

    #[test]
    fn test_chained_and_then() {
        // Test chained and_then - Note: the first and_then returns BoxBiTransformerOnce,
        // which doesn't have an and_then method, so we need to do it step by step
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        let step1 = add.and_then(double);
        let result = step1.apply_once(3, 5);
        assert_eq!(result, 16); // (3 + 5) * 2 = 16
    }

    #[test]
    fn test_and_then_with_consuming_closure() {
        // Test and_then with consuming closure
        let owned_value = String::from("prefix-");
        let concat = move |x: String, y: String| format!("{}{}{}", owned_value, x, y);
        let uppercase = |s: String| s.to_uppercase();

        let composed = concat.and_then(uppercase);
        assert_eq!(
            composed.apply_once("hello".to_string(), "world".to_string()),
            "PREFIX-HELLOWORLD"
        );
    }

    #[test]
    fn test_when_with_complex_predicate() {
        // Test complex predicate
        let add = |x: i32, y: i32| x + y;
        let multiply = |x: i32, y: i32| x * y;

        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0 && (*x + *y) < 20)
            .or_else(multiply);
        assert_eq!(conditional.apply_once(5, 3), 8); // Condition met

        let add2 = |x: i32, y: i32| x + y;
        let multiply2 = |x: i32, y: i32| x * y;
        let conditional2 = add2
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0 && (*x + *y) < 20)
            .or_else(multiply2);
        assert_eq!(conditional2.apply_once(15, 10), 150); // Condition not met (sum >= 20)

        let add3 = |x: i32, y: i32| x + y;
        let multiply3 = |x: i32, y: i32| x * y;
        let conditional3 = add3
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0 && (*x + *y) < 20)
            .or_else(multiply3);
        assert_eq!(conditional3.apply_once(-5, 3), -15); // Condition not met (x <= 0)
    }
}
