// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for BiTransformerOnce trait and implementations

use qubit_function::BiTransformerOnce;
use qubit_function::BoxBiTransformerOnce;

// ============================================================================
// Tests for BiTransformerOnce trait
// ============================================================================

#[cfg(test)]
mod conditional_tests {
    use super::BiTransformerOnce;
    use super::BoxBiTransformerOnce;

    #[test]
    fn test_when_with_or_else_condition_true() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
        let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
        assert_eq!(conditional.apply(5, 3), 8); // add
    }

    #[test]
    fn test_when_with_or_else_condition_false() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
        let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
        assert_eq!(conditional.apply(-5, 3), -15); // multiply
    }

    #[test]
    fn test_when_with_closure_or_else() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);
        assert_eq!(conditional.apply(5, 3), 8); // add
    }

    #[test]
    fn test_when_with_closure_or_else_false() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);
        assert_eq!(conditional.apply(-5, 3), -15); // multiply
    }

    #[test]
    fn test_when_with_complex_predicate() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let reverse_concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", y, x));
        let conditional = concat
            .when(|x: &String, y: &String| x.len() > y.len())
            .or_else(reverse_concat);

        assert_eq!(conditional.apply("hello".to_string(), "hi".to_string()), "hello-hi");
    }

    #[test]
    fn test_when_with_complex_predicate_false() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let reverse_concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", y, x));
        let conditional = concat
            .when(|x: &String, y: &String| x.len() > y.len())
            .or_else(reverse_concat);

        assert_eq!(conditional.apply("hi".to_string(), "hello".to_string()), "hello-hi");
    }

    #[test]
    fn test_when_both_inputs_zero() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let constant = BoxBiTransformerOnce::constant(0);
        let conditional = add.when(|x: &i32, y: &i32| *x != 0 || *y != 0).or_else(constant);
        assert_eq!(conditional.apply(0, 0), 0); // constant
    }

    #[test]
    fn test_when_one_input_zero() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let constant = BoxBiTransformerOnce::constant(0);
        let conditional = add.when(|x: &i32, y: &i32| *x != 0 || *y != 0).or_else(constant);
        assert_eq!(conditional.apply(5, 0), 5); // add
    }
}

// ============================================================================
// Tests for different data types
// ============================================================================
