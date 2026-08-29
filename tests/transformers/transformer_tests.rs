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
mod box_transformer_tests {
    use qubit_function::BoxTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_new_and_apply() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
        assert_eq!(double.apply(42), 84);
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformer::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformer::constant("hello");
        assert_eq!(constant.apply(123), "hello");
        assert_eq!(constant.apply(456), "hello");
    }

    #[test]
    fn test_with_string() {
        let len = BoxTransformer::new(|s: String| s.len());
        let text = "hello".to_string();
        assert_eq!(len.apply(text), 5);
        // Note: text is consumed by transform
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let multiply = BoxTransformer::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.apply(7), 21);
    }

    #[test]
    fn test_and_then() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = double.and_then(to_string);
        assert_eq!(composed.apply(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_display_with_name() {
        let transformer = BoxTransformer::new_with_name("double", |x: i32| x * 2);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxTransformer(double)");
    }

    #[test]
    fn test_display_without_name() {
        let transformer = BoxTransformer::new(|x: i32| x * 2);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "BoxTransformer");
    }
}

// ============================================================================
// ArcTransformer Tests - Immutable, thread-safe
// ============================================================================
