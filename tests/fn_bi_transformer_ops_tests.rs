/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BiTransformer, FnBiTransformerOps};

#[cfg(test)]
mod fn_bi_transformer_ops_tests {
    use super::*;

    #[test]
    fn test_closure_and_then() {
        // Test closure's and_then method
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        let composed = add.and_then(double);
        assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2 = 16
    }

    #[test]
    fn test_closure_and_then_with_type_conversion() {
        // Test type conversion and_then
        let add = |x: i32, y: i32| x + y;
        let to_string = |x: i32| x.to_string();

        let composed = add.and_then(to_string);
        assert_eq!(composed.apply(20, 22), "42");
    }

    #[test]
    fn test_closure_when_with_or_else() {
        // Test closure's when method
        let add = |x: i32, y: i32| x + y;
        let multiply = |x: i32, y: i32| x * y;

        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply);

        assert_eq!(conditional.apply(5, 3), 8); // Condition met, execute addition
        assert_eq!(conditional.apply(-5, 3), -15); // Condition not met, execute multiplication
    }

    #[test]
    fn test_closure_when_with_single_condition() {
        // Test when with single condition
        let add = |x: i32, y: i32| x + y;
        let subtract = |x: i32, y: i32| x - y;

        let conditional = add.when(|x: &i32, _y: &i32| *x > 0).or_else(subtract);

        assert_eq!(conditional.apply(10, 3), 13); // x > 0, execute addition
        assert_eq!(conditional.apply(-10, 3), -13); // x <= 0, execute subtraction
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
        assert_eq!(composed.apply(3, 5), 16);
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

        assert_eq!(conditional.apply(5, 3), 8);
        assert_eq!(conditional.apply(-5, 3), -15);
    }

    #[test]
    fn test_chained_and_then() {
        // Test chained and_then - Note: first and_then returns BoxBiTransformer,
        // which doesn't have and_then method, so need to do step by step
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        let step1 = add.and_then(double);
        let result = step1.apply(3, 5);
        assert_eq!(result, 16); // (3 + 5) * 2 = 16
    }

    #[test]
    fn test_and_then_with_string_types() {
        // Test string type composition
        let concat = |x: String, y: String| format!("{}{}", x, y);
        let uppercase = |s: String| s.to_uppercase();

        let composed = concat.and_then(uppercase);
        assert_eq!(
            composed.apply("hello".to_string(), "world".to_string()),
            "HELLOWORLD"
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

        assert_eq!(conditional.apply(5, 3), 8); // Condition met
        assert_eq!(conditional.apply(15, 10), 150); // Condition not met (sum >= 20)
        assert_eq!(conditional.apply(-5, 3), -15); // Condition not met (x <= 0)
    }

    #[test]
    fn test_multiple_operations() {
        // Test composition of multiple operations
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        // First do and_then composition
        let composed = add.and_then(double);
        assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2 = 16

        // Test another composition
        let multiply = |x: i32, y: i32| x * y;
        let triple = |x: i32| x * 3;
        let composed2 = multiply.and_then(triple);
        assert_eq!(composed2.apply(2, 3), 18); // (2 * 3) * 3 = 18
    }
}
