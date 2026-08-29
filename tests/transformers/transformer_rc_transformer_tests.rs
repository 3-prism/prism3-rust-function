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
mod rc_transformer_tests {
    use qubit_function::RcTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_new_and_apply() {
        let double = RcTransformer::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_clone() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(21), 42);
        assert_eq!(cloned.apply(21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = RcTransformer::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcTransformer::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let to_upper = RcTransformer::new(|s: String| s.to_uppercase());

        let func1 = to_upper.clone();
        let func2 = to_upper.clone();

        assert_eq!(to_upper.apply("hello".to_string()), "HELLO");
        assert_eq!(func1.apply("world".to_string()), "WORLD");
        assert_eq!(func2.apply("rust".to_string()), "RUST");
    }

    #[test]
    fn test_and_then() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let to_string = RcTransformer::new(|x: i32| x.to_string());
        let composed = double.and_then(to_string);

        // Original double transformer still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(composed.apply(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let composed = add_one.and_then(double);

        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_display_with_name() {
        let transformer = RcTransformer::new_with_name("double", |x: i32| x * 2);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "RcTransformer(double)");
    }

    #[test]
    fn test_display_without_name() {
        let transformer = RcTransformer::new(|x: i32| x * 2);
        let display_str = format!("{}", transformer);
        assert_eq!(display_str, "RcTransformer");
    }
}

// ============================================================================
// Conditional Transformer Tests
// ============================================================================
