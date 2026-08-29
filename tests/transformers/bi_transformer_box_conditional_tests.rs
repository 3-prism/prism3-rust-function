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
mod box_conditional_tests {
    use qubit_function::BoxBiPredicate;

    use super::BiTransformer;
    use super::BoxBiTransformer;

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
