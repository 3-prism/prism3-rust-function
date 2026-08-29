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
mod complex_composition_tests {
    use qubit_function::ArcTransformer;
    use qubit_function::BoxTransformer;
    use qubit_function::RcTransformer;
    use qubit_function::Transformer;

    #[test]
    fn test_multiple_and_then() {
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let double = BoxTransformer::new(|x: i32| x * 2);
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.apply(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_and_then_with_box() {
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let double = BoxTransformer::new(|x: i32| x * 2);
        let square = BoxTransformer::new(|x: i32| x * x);
        let composed = add_one.and_then(double).and_then(square);
        assert_eq!(composed.apply(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_arc_multiple_and_then() {
        let add_one = ArcTransformer::new(|x: i32| x + 1);
        let double = ArcTransformer::new(|x: i32| x * 2);
        let to_string = ArcTransformer::new(|x: i32| x.to_string());
        let composed = add_one.and_then(double.clone()).and_then(to_string.clone());
        assert_eq!(composed.apply(5), "12");
        // Original transformers still usable
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(double.apply(5), 10);
    }

    #[test]
    fn test_rc_multiple_and_then() {
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let double = RcTransformer::new(|x: i32| x * 2);
        let square = RcTransformer::new(|x: i32| x * x);
        let composed = add_one.and_then(double.clone()).and_then(square.clone());
        assert_eq!(composed.apply(5), 144); // (5 + 1) * 2 = 12, then 12 * 12 = 144
        // Original transformers still usable
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(double.apply(5), 10);
        assert_eq!(square.apply(5), 25);
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================
