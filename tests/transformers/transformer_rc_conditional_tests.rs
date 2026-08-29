// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// ============================================================================
// BoxTransformer Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod rc_conditional_tests {
    use qubit_function::RcTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_when_or_else() {
        let double_fn = |x: i32| x * 2;
        let negate_fn = |x: i32| -x;
        let result = RcTransformer::new(double_fn).when(|x: &i32| *x > 0).or_else(negate_fn);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double_fn = |x: i32| x * 2;
        let result = RcTransformer::new(double_fn)
            .when(|x: &i32| *x > 0)
            .or_else(|x: i32| -x);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
        assert_eq!(result.apply(0), 0);
    }

    #[test]
    fn test_conditional_or_else() {
        let double_fn = |x: i32| x * 2;
        let result = RcTransformer::new(double_fn)
            .when(|x: &i32| *x > 0)
            .or_else(|x: i32| -x);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================

// ============================================================================
// Non-consuming Conversion Tests (to_xxx methods)
// ============================================================================
