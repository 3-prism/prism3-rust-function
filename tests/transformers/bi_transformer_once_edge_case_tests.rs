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
mod edge_case_tests {
    use super::BiTransformerOnce;
    use super::BoxBiTransformerOnce;

    #[test]
    fn test_with_empty_strings() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}{}", x, y));
        assert_eq!(concat.apply(String::new(), String::new()), String::new());
    }

    #[test]
    fn test_with_zero_values() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(0, 0), 0);
    }

    #[test]
    fn test_with_negative_values() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(-5, -3), -8);
    }

    #[test]
    fn test_with_large_values() {
        let add = BoxBiTransformerOnce::new(|x: i64, y: i64| x + y);
        assert_eq!(add.apply(1_000_000_000, 2_000_000_000), 3_000_000_000);
    }

    #[test]
    fn test_constant_ignores_inputs() {
        let constant = BoxBiTransformerOnce::constant(42);
        assert_eq!(constant.apply(999, 888), 42);
    }

    #[test]
    fn test_with_unicode_strings() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}{}", x, y));
        assert_eq!(concat.apply("Hello".to_string(), "World".to_string()), "HelloWorld");
    }
}

// ============================================================================
// Tests for complex transformations
// ============================================================================
