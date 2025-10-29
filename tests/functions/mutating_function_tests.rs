/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for MutatingFunction types (stateless Fn(&mut T) -> R)

use prism3_function::{
    ArcMutatingFunction,
    BoxMutatingFunction,
    FnMutatingFunctionOps,
    MutatingFunction,
    RcMutatingFunction,
};

// ============================================================================
// BoxMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_box_mutating_function {
    use super::*;

    #[test]
    fn test_new() {
        let func = BoxMutatingFunction::new(|x: &mut i32| {
            *x += 1;
            *x
        });
        let mut value = 5;
        assert_eq!(func.apply(&mut value), 6);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let string_func = BoxMutatingFunction::new(|s: &mut String| {
            let old_len = s.len();
            s.push('!');
            old_len
        });
        let mut text = String::from("hello");
        assert_eq!(string_func.apply(&mut text), 5);
        assert_eq!(text, "hello!");

        // Vec
        let vec_func = BoxMutatingFunction::new(|v: &mut Vec<i32>| {
            let old_len = v.len();
            v.push(42);
            old_len
        });
        let mut numbers = vec![1, 2, 3];
        assert_eq!(vec_func.apply(&mut numbers), 3);
        assert_eq!(numbers, vec![1, 2, 3, 42]);

        // bool
        let bool_func = BoxMutatingFunction::new(|b: &mut bool| {
            let old = *b;
            *b = !*b;
            old
        });
        let mut flag = true;
        assert!(bool_func.apply(&mut flag));
        assert!(!flag);
    }

    #[test]
    fn test_and_then() {
        let func = BoxMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        })
        .and_then(|x: &mut i32| {
            *x += 10;
            *x
        });

        let mut value = 5;
        assert_eq!(func.apply(&mut value), 20); // (5 * 2) + 10
        assert_eq!(value, 20);
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let func = BoxMutatingFunction::new(|x: &mut i32| {
            *x += 1;
            *x
        })
        .and_then(|x: &mut i32| {
            *x *= 2;
            *x
        })
        .and_then(|x: &mut i32| {
            *x -= 5;
            *x
        });

        let mut value = 10;
        assert_eq!(func.apply(&mut value), 17); // ((10 + 1) * 2) - 5
        assert_eq!(value, 17);
    }

    #[test]
    fn test_and_then_with_box_mutating_function() {
        let f1 = BoxMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let f2 = BoxMutatingFunction::new(|x: &mut i32| {
            *x += 10;
            *x
        });
        let combined = f1.and_then(f2);

        let mut value = 5;
        assert_eq!(combined.apply(&mut value), 20);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_identity() {
        let identity = BoxMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = BoxMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let mapped = func.map(|result| result.to_string());

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "10");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_fn() {
        let func = BoxMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let closure = func.into_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_box() {
        let func = BoxMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = BoxMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let rc = func.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }
}

// ============================================================================
// RcMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_rc_mutating_function {
    use super::*;

    #[test]
    fn test_new() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x += 1;
            *x
        });
        let mut value = 5;
        assert_eq!(func.apply(&mut value), 6);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let clone = func.clone();

        let mut value1 = 5;
        assert_eq!(func.apply(&mut value1), 10);

        let mut value2 = 3;
        assert_eq!(clone.apply(&mut value2), 6);
    }

    #[test]
    fn test_and_then() {
        let f1 = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let f2 = RcMutatingFunction::new(|x: &mut i32| {
            *x += 10;
            *x
        });
        let combined = f1.and_then(&f2);

        // f1 and f2 are still usable
        let mut value = 5;
        assert_eq!(combined.apply(&mut value), 20);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_identity() {
        let identity = RcMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let mapped = func.map(|result| result.to_string());

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "10");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_box() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let boxed = func.to_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_box() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let rc = func.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_fn() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let closure = func.into_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_rc() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let rc = func.to_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 10);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut value2 = 3;
        assert_eq!(func.apply(&mut value2), 6);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_fn() {
        let func = RcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let closure = func.to_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 10);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut value2 = 3;
        assert_eq!(func.apply(&mut value2), 6);
        assert_eq!(value2, 6);
    }
}

// ============================================================================
// ArcMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_arc_mutating_function {
    use super::*;
    use std::thread;

    #[test]
    fn test_new() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x += 1;
            *x
        });
        let mut value = 5;
        assert_eq!(func.apply(&mut value), 6);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let clone = func.clone();

        let mut value1 = 5;
        assert_eq!(func.apply(&mut value1), 10);

        let mut value2 = 3;
        assert_eq!(clone.apply(&mut value2), 6);
    }

    #[test]
    fn test_thread_safe() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let func_clone = func.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            func_clone.apply(&mut value)
        });

        let result = handle.join().unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn test_and_then() {
        let f1 = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let f2 = ArcMutatingFunction::new(|x: &mut i32| {
            *x += 10;
            *x
        });
        let combined = f1.and_then(&f2);

        // f1 and f2 are still usable
        let mut value = 5;
        assert_eq!(combined.apply(&mut value), 20);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_identity() {
        let identity = ArcMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let mapped = func.map(|result| result.to_string());

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "10");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_box() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let boxed = func.to_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_rc() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let rc = func.to_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_box() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let rc = func.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_arc() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let arc = func.into_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_fn() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let closure = func.into_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_arc() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let arc = func.to_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 10);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut value2 = 3;
        assert_eq!(func.apply(&mut value2), 6);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_fn() {
        let func = ArcMutatingFunction::new(|x: &mut i32| {
            *x *= 2;
            *x
        });
        let closure = func.to_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 10);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut value2 = 3;
        assert_eq!(func.apply(&mut value2), 6);
        assert_eq!(value2, 6);
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
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };

        let mut value = 5;
        assert_eq!(closure.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_and_then() {
        let chained = (|x: &mut i32| {
            *x *= 2;
            *x
        })
        .and_then(|x: &mut i32| {
            *x += 10;
            *x
        });

        let mut value = 5;
        assert_eq!(chained.apply(&mut value), 20);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_closure_map() {
        let mapped = (|x: &mut i32| {
            *x *= 2;
            *x
        })
        .map(|result| result.to_string());

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "10");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_box() {
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };
        let boxed = closure.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_rc() {
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };
        let rc = closure.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_arc() {
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };
        let arc = closure.into_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_box() {
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };
        let boxed = closure.to_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_rc() {
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };
        let rc = closure.to_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_arc() {
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };
        let arc = closure.to_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 10);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_fn() {
        let closure = |x: &mut i32| {
            *x *= 2;
            *x
        };
        let fn_closure = closure.to_fn();

        let mut value = 5;
        assert_eq!(fn_closure(&mut value), 10);
        assert_eq!(value, 10);
    }
}

// ============================================================================
// MutatingFunction Default Implementation Tests
// ============================================================================

/// Test struct that implements MutatingFunction to test default methods
struct TestMutatingFunction {
    multiplier: i32,
}

impl TestMutatingFunction {
    fn new(multiplier: i32) -> Self {
        TestMutatingFunction { multiplier }
    }
}

impl MutatingFunction<i32, i32> for TestMutatingFunction {
    fn apply(&self, input: &mut i32) -> i32 {
        let old_value = *input;
        *input *= self.multiplier;
        old_value
    }
}

impl Clone for TestMutatingFunction {
    fn clone(&self) -> Self {
        TestMutatingFunction {
            multiplier: self.multiplier,
        }
    }
}

#[cfg(test)]
mod test_mutating_function_default_impl {
    use super::*;

    #[test]
    fn test_into_box() {
        let func = TestMutatingFunction::new(2);
        let boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 5);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = TestMutatingFunction::new(3);
        let rc = func.into_rc();

        let mut value = 4;
        assert_eq!(rc.apply(&mut value), 4);
        assert_eq!(value, 12);
    }

    #[test]
    fn test_into_arc() {
        let func = TestMutatingFunction::new(4);
        let arc = func.into_arc();

        let mut value = 3;
        assert_eq!(arc.apply(&mut value), 3);
        assert_eq!(value, 12);
    }

    #[test]
    fn test_into_fn() {
        let func = TestMutatingFunction::new(5);
        let closure = func.into_fn();

        let mut value = 2;
        assert_eq!(closure(&mut value), 2);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_box() {
        let func = TestMutatingFunction::new(2);
        let boxed = func.to_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 5);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut value2 = 3;
        assert_eq!(func.apply(&mut value2), 3);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_rc() {
        let func = TestMutatingFunction::new(3);
        let rc = func.to_rc();

        let mut value = 4;
        assert_eq!(rc.apply(&mut value), 4);
        assert_eq!(value, 12);

        // Original should still be usable since it was cloned
        let mut value2 = 2;
        assert_eq!(func.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_arc() {
        let func = TestMutatingFunction::new(4);
        let arc = func.to_arc();

        let mut value = 3;
        assert_eq!(arc.apply(&mut value), 3);
        assert_eq!(value, 12);

        // Original should still be usable since it was cloned
        let mut value2 = 2;
        assert_eq!(func.apply(&mut value2), 2);
        assert_eq!(value2, 8);
    }

    #[test]
    fn test_to_fn() {
        let func = TestMutatingFunction::new(5);
        let closure = func.to_fn();

        let mut value = 2;
        assert_eq!(closure(&mut value), 2);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut value2 = 1;
        assert_eq!(func.apply(&mut value2), 1);
        assert_eq!(value2, 5);
    }
}
