// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::thread;

use qubit_function::ArcBiTransformer;
use qubit_function::BiTransformer;
use qubit_function::BoxBiTransformer;
use qubit_function::RcBiTransformer;

// ============================================================================
// BoxBiTransformer Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod arc_conditional_tests {
    use qubit_function::ArcBiPredicate;

    use super::ArcBiTransformer;
    use super::BiTransformer;

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
