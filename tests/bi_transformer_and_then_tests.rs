/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiTransformer and_then method tests
//!
//! Tests the and_then method for BoxBiTransformer, ArcBiTransformer and RcBiTransformer
//!
//! # Author
//!
//! Hu Haixing

#[cfg(test)]
mod tests {
    use prism3_function::{
        ArcBiTransformer, ArcTransformer, BiTransformer, BoxBiTransformer, BoxTransformer,
        RcBiTransformer, RcTransformer,
    };

    // ========================================================================
    // BoxBiTransformer::and_then tests
    // ========================================================================

    #[test]
    fn test_box_bi_transformer_and_then_with_closure() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = add.and_then(double);

        assert_eq!(composed.transform(3, 5), 16); // (3 + 5) * 2
    }

    #[test]
    fn test_box_bi_transformer_and_then_with_transformer() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = add.and_then(to_string);

        assert_eq!(composed.transform(20, 22), "42");
    }

    #[test]
    fn test_box_bi_transformer_and_then_chain() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();

        let composed = add.and_then(double).and_then(to_string);

        assert_eq!(composed.transform(10, 11), "42"); // ((10 + 11) * 2).to_string()
    }

    #[test]
    fn test_box_bi_transformer_and_then_type_conversion() {
        let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
        let to_float = |x: i32| x as f64;
        let composed = multiply.and_then(to_float);

        assert_eq!(composed.transform(6, 7), 42.0);
    }

    // ========================================================================
    // ArcBiTransformer::and_then tests
    // ========================================================================

    #[test]
    fn test_arc_bi_transformer_and_then_with_closure() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = add.and_then(double);

        // Original bi-transformer still usable
        assert_eq!(add.transform(20, 22), 42);
        assert_eq!(composed.transform(3, 5), 16); // (3 + 5) * 2
    }

    #[test]
    fn test_arc_bi_transformer_and_then_with_transformer() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let to_string = ArcTransformer::new(|x: i32| x.to_string());
        let composed = add.and_then(to_string);

        assert_eq!(add.transform(20, 22), 42);
        assert_eq!(composed.transform(20, 22), "42");
    }

    #[test]
    fn test_arc_bi_transformer_and_then_clone() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = add.and_then(double);

        let composed_clone = composed.clone();

        assert_eq!(composed.transform(3, 5), 16);
        assert_eq!(composed_clone.transform(3, 5), 16);
    }

    #[test]
    fn test_arc_bi_transformer_and_then_chain() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();

        let composed = add.and_then(double).and_then(to_string);

        assert_eq!(composed.transform(10, 11), "42");
    }

    #[test]
    fn test_arc_bi_transformer_and_then_thread_safe() {
        use std::sync::Arc as StdArc;
        use std::thread;

        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = StdArc::new(add.and_then(double));

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let composed_clone = StdArc::clone(&composed);
                thread::spawn(move || composed_clone.transform(i, i + 1))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(results, vec![2, 6, 10, 14]); // [(0+1)*2, (1+2)*2, (2+3)*2, (3+4)*2]
    }

    // ========================================================================
    // RcBiTransformer::and_then tests
    // ========================================================================

    #[test]
    fn test_rc_bi_transformer_and_then_with_closure() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = add.and_then(double);

        // Original bi-transformer still usable
        assert_eq!(add.transform(20, 22), 42);
        assert_eq!(composed.transform(3, 5), 16); // (3 + 5) * 2
    }

    #[test]
    fn test_rc_bi_transformer_and_then_with_transformer() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let to_string = RcTransformer::new(|x: i32| x.to_string());
        let composed = add.and_then(to_string);

        assert_eq!(add.transform(20, 22), 42);
        assert_eq!(composed.transform(20, 22), "42");
    }

    #[test]
    fn test_rc_bi_transformer_and_then_clone() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = add.and_then(double);

        let composed_clone = composed.clone();

        assert_eq!(composed.transform(3, 5), 16);
        assert_eq!(composed_clone.transform(3, 5), 16);
    }

    #[test]
    fn test_rc_bi_transformer_and_then_chain() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();

        let composed = add.and_then(double).and_then(to_string);

        assert_eq!(composed.transform(10, 11), "42");
    }

    // ========================================================================
    // Mixed type tests
    // ========================================================================

    #[test]
    fn test_box_bi_transformer_and_then_with_arc_transformer() {
        let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
        let double = ArcTransformer::new(|x: i32| x * 2);
        let composed = add.and_then(double);

        assert_eq!(composed.transform(3, 5), 16);
    }

    #[test]
    fn test_arc_bi_transformer_and_then_with_arc_transformer() {
        let add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = ArcTransformer::new(|x: i32| x * 2);
        let composed = add.and_then(double);

        assert_eq!(composed.transform(3, 5), 16);
    }

    #[test]
    fn test_rc_bi_transformer_and_then_with_box_transformer() {
        let add = RcBiTransformer::new(|x: i32, y: i32| x + y);
        let double = BoxTransformer::new(|x: i32| x * 2);
        let composed = add.and_then(double);

        assert_eq!(composed.transform(3, 5), 16);
    }

    // ========================================================================
    // Complex scenario tests
    // ========================================================================

    #[test]
    fn test_bi_transformer_and_then_with_complex_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct Person {
            name: String,
            age: i32,
        }

        let create_person = BoxBiTransformer::new(|name: String, age: i32| Person { name, age });

        let get_description = |p: Person| format!("{} is {} years old", p.name, p.age);

        let composed = create_person.and_then(get_description);

        assert_eq!(
            composed.transform("Alice".to_string(), 30),
            "Alice is 30 years old"
        );
    }

    #[test]
    fn test_bi_transformer_and_then_with_option() {
        let divide =
            BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });

        let unwrap_or_zero = |opt: Option<i32>| opt.unwrap_or(0);

        let composed = divide.and_then(unwrap_or_zero);

        assert_eq!(composed.transform(10, 2), 5);
        assert_eq!(composed.transform(10, 0), 0);
    }

    #[test]
    fn test_bi_transformer_and_then_with_result() {
        let divide = BoxBiTransformer::new(|x: i32, y: i32| -> Result<i32, String> {
            if y == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(x / y)
            }
        });

        let to_string = |res: Result<i32, String>| match res {
            Ok(v) => format!("Success: {}", v),
            Err(e) => format!("Error: {}", e),
        };

        let composed = divide.and_then(to_string);

        assert_eq!(composed.transform(10, 2), "Success: 5");
        assert_eq!(composed.transform(10, 0), "Error: Division by zero");
    }
}
