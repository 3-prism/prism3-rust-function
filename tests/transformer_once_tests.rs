/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BoxTransformerOnce, TransformerOnce};

// ============================================================================
// BoxTransformerOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_transformer_once_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));

        assert_eq!(parse.transform("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        assert_eq!(identity.transform(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.transform(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
        assert_eq!(pipeline.transform(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxTransformerOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.transform("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.transform(42), "42_suffix");
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_transformer_once_trait() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.transform(x)
        }

        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        assert_eq!(apply_transformer_once(double, 21), 42);
    }
}
