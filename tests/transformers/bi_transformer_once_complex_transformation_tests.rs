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
mod complex_transformation_tests {
    use super::BiTransformerOnce;
    use super::BoxBiTransformerOnce;

    #[test]
    fn test_nested_structure_transformation() {
        let merge_nested = BoxBiTransformerOnce::new(|x: Vec<Vec<i32>>, y: Vec<Vec<i32>>| -> Vec<Vec<i32>> {
            let mut result = x;
            result.extend(y);
            result
        });
        assert_eq!(
            merge_nested.apply(vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]),
            vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]]
        );
    }

    #[test]
    fn test_transformation_with_calculation() {
        let calculate = BoxBiTransformerOnce::new(|x: i32, y: i32| {
            let sum = x + y;
            let product = x * y;
            (sum, product)
        });
        assert_eq!(calculate.apply(5, 3), (8, 15));
    }

    #[test]
    fn test_transformation_with_string_manipulation() {
        let process = BoxBiTransformerOnce::new(|x: String, y: String| {
            format!("{} {} {}", x.to_uppercase(), "and", y.to_lowercase())
        });
        assert_eq!(
            process.apply("Hello".to_string(), "WORLD".to_string()),
            "HELLO and world"
        );
    }

    #[test]
    fn test_conditional_with_complex_logic() {
        let complex_add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y + 10);
        let complex_multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y - 5);
        let conditional = complex_add
            .when(|x: &i32, y: &i32| (*x + *y) % 2 == 0)
            .or_else(complex_multiply);

        assert_eq!(conditional.apply(4, 6), 20); // (4 + 6) is even, so add: 4 + 6 + 10 = 20
    }

    #[test]
    fn test_conditional_with_complex_logic_odd() {
        let complex_add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y + 10);
        let complex_multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y - 5);
        let conditional = complex_add
            .when(|x: &i32, y: &i32| (*x + *y) % 2 == 0)
            .or_else(complex_multiply);

        assert_eq!(conditional.apply(3, 4), 7); // (3 + 4) is odd, so multiply: 3 * 4 - 5 = 7
    }
}

// ============================================================================
// Tests for ownership and consumption
// ============================================================================
