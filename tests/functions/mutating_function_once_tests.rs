/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for MutatingFunctionOnce types (one-time FnOnce(&mut T) -> R)

use prism3_function::{
    BoxMutatingFunctionOnce,
    FnOnceMutatingFunctionOps,
    MutatingFunctionOnce,
};

// ============================================================================
// BoxMutatingFunctionOnce Tests
// ============================================================================

#[cfg(test)]
mod test_box_mutating_function_once {
    use super::*;

    #[test]
    fn test_new() {
        let data = vec![1, 2, 3];
        let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
            let old_len = x.len();
            x.extend(data);
            old_len
        });

        let mut target = vec![0];
        let old_len = func.apply_once(&mut target);
        assert_eq!(old_len, 1);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_with_string() {
        let data = String::from(" world");
        let func = BoxMutatingFunctionOnce::new(move |x: &mut String| {
            let old_len = x.len();
            x.push_str(&data);
            old_len
        });

        let mut target = String::from("hello");
        let old_len = func.apply_once(&mut target);
        assert_eq!(old_len, 5);
        assert_eq!(target, "hello world");
    }

    #[test]
    fn test_and_then() {
        let data1 = vec![1, 2];
        let data2 = vec![3, 4];

        let chained = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data1);
            x.len()
        })
        .and_then(move |x: &mut Vec<i32>| {
            x.extend(data2);
            x.len()
        });

        let mut target = vec![0];
        let final_len = chained.apply_once(&mut target);
        assert_eq!(final_len, 5);
        assert_eq!(target, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let data1 = vec![1, 2];
        let data2 = vec![3, 4];
        let data3 = vec![5, 6];

        let chained = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data1);
            x.len()
        })
        .and_then(move |x: &mut Vec<i32>| {
            x.extend(data2);
            x.len()
        })
        .and_then(move |x: &mut Vec<i32>| {
            x.extend(data3);
            x.len()
        });

        let mut target = vec![0];
        let final_len = chained.apply_once(&mut target);
        assert_eq!(final_len, 7);
        assert_eq!(target, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_identity() {
        let identity = BoxMutatingFunctionOnce::<i32, i32>::identity();
        let mut value = 42;
        let result = identity.apply_once(&mut value);
        assert_eq!(result, 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let data = vec![1, 2, 3];
        let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
            let old_len = x.len();
            x.extend(data);
            old_len
        });
        let mapped = func.map(|old_len| format!("Old length: {}", old_len));

        let mut target = vec![0];
        let result = mapped.apply_once(&mut target);
        assert_eq!(result, "Old length: 1");
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_validation_pattern() {
        struct Data {
            value: i32,
        }

        let validator = BoxMutatingFunctionOnce::new(|data: &mut Data| {
            if data.value < 0 {
                data.value = 0;
                Err("Fixed negative value")
            } else {
                Ok("Valid")
            }
        });

        let mut data = Data { value: -5 };
        let result = validator.apply_once(&mut data);
        assert_eq!(data.value, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_transfer() {
        let resource = vec![1, 2, 3, 4, 5];
        let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
            let old_sum: i32 = x.iter().sum();
            x.extend(resource);
            old_sum
        });

        let mut target = vec![10, 20];
        let old_sum = func.apply_once(&mut target);
        assert_eq!(old_sum, 30);
        assert_eq!(target, vec![10, 20, 1, 2, 3, 4, 5]);
    }
}

// ============================================================================
// Closure Tests
// ============================================================================

#[cfg(test)]
mod test_closure {
    use super::*;

    #[test]
    fn test_closure_implements_trait() {
        let data = vec![1, 2, 3];
        let closure = move |x: &mut Vec<i32>| {
            let old_len = x.len();
            x.extend(data);
            old_len
        };

        let mut target = vec![0];
        let old_len = closure.apply_once(&mut target);
        assert_eq!(old_len, 1);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_closure_and_then() {
        let data1 = vec![1, 2];
        let data2 = vec![3, 4];

        let chained = (move |x: &mut Vec<i32>| {
            x.extend(data1);
            x.len()
        })
        .and_then(move |x: &mut Vec<i32>| {
            x.extend(data2);
            x.len()
        });

        let mut target = vec![0];
        let final_len = chained.apply_once(&mut target);
        assert_eq!(final_len, 5);
        assert_eq!(target, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_closure_map() {
        let data = vec![1, 2, 3];
        let mapped = (move |x: &mut Vec<i32>| {
            let old_len = x.len();
            x.extend(data);
            old_len
        })
        .map(|old_len| format!("Old length: {}", old_len));

        let mut target = vec![0];
        let result = mapped.apply_once(&mut target);
        assert_eq!(result, "Old length: 1");
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_closure_into_box_once() {
        let data = vec![1, 2, 3];
        let closure = move |x: &mut Vec<i32>| {
            let old_len = x.len();
            x.extend(data);
            old_len
        };
        let box_func = closure.into_box_once();

        let mut target = vec![0];
        let old_len = box_func.apply_once(&mut target);
        assert_eq!(old_len, 1);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_move_semantics() {
        let data = vec![1, 2, 3];
        let closure = move |x: &mut Vec<i32>| {
            let old_len = x.len();
            x.extend(data); // data is moved into closure
            old_len
        };
        // data is no longer accessible here

        let mut target = vec![0];
        let old_len = closure.apply_once(&mut target);
        assert_eq!(old_len, 1);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }
}
