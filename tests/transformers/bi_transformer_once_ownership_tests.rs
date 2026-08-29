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
mod ownership_tests {
    use super::BiTransformerOnce;
    use super::BoxBiTransformerOnce;

    #[test]
    fn test_consumes_owned_values() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let s1 = String::from("hello");
        let s2 = String::from("world");
        let result = concat.apply(s1, s2);
        assert_eq!(result, "hello-world");
        // s1 and s2 are consumed and cannot be used here
    }

    #[test]
    fn test_consumes_vectors() {
        let merge = BoxBiTransformerOnce::new(|mut x: Vec<i32>, y: Vec<i32>| {
            x.extend(y);
            x
        });
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5, 6];
        let result = merge.apply(v1, v2);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
        // v1 and v2 are consumed
    }

    #[test]
    fn test_closure_captures_and_consumes() {
        let prefix = String::from("Result: ");
        let concat = BoxBiTransformerOnce::new(move |x: String, y: String| format!("{}{}-{}", prefix, x, y));
        let result = concat.apply("hello".to_string(), "world".to_string());
        assert_eq!(result, "Result: hello-world");
        // prefix is moved into closure
    }
}

// ============================================================================
// Conditional BiTransformerOnce Display/Debug Tests
// ============================================================================
