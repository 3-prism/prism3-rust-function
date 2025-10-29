/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for StatefulMutatingFunction types (stateful FnMut(&mut T) ->
//! R)

use prism3_function::{
    ArcStatefulMutatingFunction,
    BoxStatefulMutatingFunction,
    FnMutStatefulMutatingFunctionOps,
    RcStatefulMutatingFunction,
    StatefulMutatingFunction,
};

// ============================================================================
// StatefulMutatingFunction Default Implementation Tests
// ============================================================================

/// Test struct that implements StatefulMutatingFunction to test default methods
struct TestStatefulMutatingFunction {
    multiplier: i32,
}

impl TestStatefulMutatingFunction {
    fn new(multiplier: i32) -> Self {
        TestStatefulMutatingFunction { multiplier }
    }
}

impl StatefulMutatingFunction<i32, i32> for TestStatefulMutatingFunction {
    fn apply(&mut self, input: &mut i32) -> i32 {
        let old_value = *input;
        *input *= self.multiplier;
        old_value
    }
}

impl Clone for TestStatefulMutatingFunction {
    fn clone(&self) -> Self {
        TestStatefulMutatingFunction {
            multiplier: self.multiplier,
        }
    }
}

#[cfg(test)]
mod test_stateful_mutating_function_default_impl {
    use super::*;

    #[test]
    fn test_into_box() {
        let func = TestStatefulMutatingFunction::new(2);
        let mut boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 5);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = TestStatefulMutatingFunction::new(3);
        let mut rc = func.into_rc();

        let mut value = 4;
        assert_eq!(rc.apply(&mut value), 4);
        assert_eq!(value, 12);
    }

    #[test]
    fn test_into_arc() {
        let func = TestStatefulMutatingFunction::new(4);
        let mut arc = func.into_arc();

        let mut value = 3;
        assert_eq!(arc.apply(&mut value), 3);
        assert_eq!(value, 12);
    }

    #[test]
    fn test_into_fn() {
        let func = TestStatefulMutatingFunction::new(5);
        let mut closure = func.into_fn();

        let mut value = 2;
        assert_eq!(closure(&mut value), 2);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_box() {
        let mut func = TestStatefulMutatingFunction::new(2);
        let mut boxed = func.to_box();

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
        let mut func = TestStatefulMutatingFunction::new(3);
        let mut rc = func.to_rc();

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
        let mut func = TestStatefulMutatingFunction::new(4);
        let mut arc = func.to_arc();

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
        let func = TestStatefulMutatingFunction::new(5);

        // Test to_fn conversion
        let mut closure = func.to_fn();
        let mut value = 2;
        assert_eq!(closure(&mut value), 2);
        assert_eq!(value, 10);

        // Test that original is still usable (need to create a new instance for comparison)
        let mut func2 = TestStatefulMutatingFunction::new(5);
        let mut value2 = 1;
        assert_eq!(func2.apply(&mut value2), 1);
        assert_eq!(value2, 5);
    }
}

// ============================================================================
// BoxStatefulMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_box_stateful_mutating_function {
    use super::*;

    #[test]
    fn test_new() {
        let mut counter = {
            let mut count = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x += 1;
                count
            })
        };
        let mut value = 5;
        assert_eq!(counter.apply(&mut value), 1);
        assert_eq!(value, 6);
        assert_eq!(counter.apply(&mut value), 2);
        assert_eq!(value, 7);
    }

    #[test]
    fn test_accumulator() {
        let mut accumulator = {
            let mut sum = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                *x *= 2;
                sum += *x;
                sum
            })
        };

        let mut value = 5;
        assert_eq!(accumulator.apply(&mut value), 10);
        assert_eq!(value, 10);

        let mut value2 = 3;
        assert_eq!(accumulator.apply(&mut value2), 16); // 10 + 6
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = {
            let mut count1 = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count1 += 1;
                *x *= 2;
                count1
            })
        };
        let second = {
            let mut count2 = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count2 += 1;
                *x += 10;
                count2
            })
        };

        let mut chained = first.and_then(second);
        let mut value = 5;
        assert_eq!(chained.apply(&mut value), 1);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_identity() {
        let mut identity = BoxStatefulMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = {
            let mut count = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut mapped = func.map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_box() {
        let func = {
            let mut count = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = {
            let mut count = 0;
            BoxStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut rc = func.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }
}

// ============================================================================
// RcStatefulMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_rc_stateful_mutating_function {
    use super::*;

    #[test]
    fn test_new() {
        let mut counter = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x += 1;
                count
            })
        };
        let mut value = 5;
        assert_eq!(counter.apply(&mut value), 1);
        assert_eq!(value, 6);
        assert_eq!(counter.apply(&mut value), 2);
        assert_eq!(value, 7);
    }

    #[test]
    fn test_clone() {
        let counter = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut clone = counter.clone();

        let mut value1 = 5;
        assert_eq!(clone.apply(&mut value1), 1);
        assert_eq!(value1, 10);

        // Shared state
        let mut value2 = 3;
        assert_eq!(clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = {
            let mut count1 = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count1 += 1;
                *x *= 2;
                count1
            })
        };
        let second = {
            let mut count2 = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count2 += 1;
                *x += 10;
                count2
            })
        };

        let mut combined = first.and_then(&second);
        let mut value = 5;
        assert_eq!(combined.apply(&mut value), 1);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_identity() {
        let mut identity = RcStatefulMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut mapped = func.map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_box() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut rc = func.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_fn() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut closure = func.into_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_box() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut boxed = func.to_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut func_clone = func.clone();
        let mut value2 = 3;
        assert_eq!(func_clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_rc() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut rc = func.to_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut func_clone = func.clone();
        let mut value2 = 3;
        assert_eq!(func_clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_fn() {
        let func = {
            let mut count = 0;
            RcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut closure = func.to_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 1);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut func_clone = func.clone();
        let mut value2 = 3;
        assert_eq!(func_clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }
}

// ============================================================================
// ArcStatefulMutatingFunction Tests
// ============================================================================

#[cfg(test)]
mod test_arc_stateful_mutating_function {
    use super::*;
    use std::thread;

    #[test]
    fn test_new() {
        let mut counter = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x += 1;
                count
            })
        };
        let mut value = 5;
        assert_eq!(counter.apply(&mut value), 1);
        assert_eq!(value, 6);
        assert_eq!(counter.apply(&mut value), 2);
        assert_eq!(value, 7);
    }

    #[test]
    fn test_clone() {
        let counter = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut clone = counter.clone();

        let mut value1 = 5;
        assert_eq!(clone.apply(&mut value1), 1);
        assert_eq!(value1, 10);

        // Shared state
        let mut value2 = 3;
        assert_eq!(clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_thread_safe() {
        let counter = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut counter_clone = counter.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            counter_clone.apply(&mut value)
        });

        let result = handle.join().unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_and_then() {
        let first = {
            let mut count1 = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count1 += 1;
                *x *= 2;
                count1
            })
        };
        let second = {
            let mut count2 = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count2 += 1;
                *x += 10;
                count2
            })
        };

        let mut combined = first.and_then(&second);
        let mut value = 5;
        assert_eq!(combined.apply(&mut value), 1);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_identity() {
        let mut identity = ArcStatefulMutatingFunction::<i32, i32>::identity();
        let mut value = 42;
        assert_eq!(identity.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut mapped = func.map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_box() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut boxed = func.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut rc = func.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_arc() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut arc = func.into_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_fn() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut closure = func.into_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_box() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut boxed = func.to_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut func_clone = func.clone();
        let mut value2 = 3;
        assert_eq!(func_clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_rc() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut rc = func.to_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut func_clone = func.clone();
        let mut value2 = 3;
        assert_eq!(func_clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_arc() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut arc = func.to_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 1);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut func_clone = func.clone();
        let mut value2 = 3;
        assert_eq!(func_clone.apply(&mut value2), 2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_fn() {
        let func = {
            let mut count = 0;
            ArcStatefulMutatingFunction::new(move |x: &mut i32| {
                count += 1;
                *x *= 2;
                count
            })
        };
        let mut closure = func.to_fn();

        let mut value = 5;
        assert_eq!(closure(&mut value), 1);
        assert_eq!(value, 10);

        // Original should still be usable since it was cloned
        let mut func_clone = func.clone();
        let mut value2 = 3;
        assert_eq!(func_clone.apply(&mut value2), 2);
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
        let mut count = 0;
        let mut closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };

        let mut value = 5;
        assert_eq!(closure.apply(&mut value), 1);
        assert_eq!(value, 10);
        assert_eq!(closure.apply(&mut value), 2);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_closure_and_then() {
        let mut count1 = 0;
        let mut count2 = 0;
        let mut chained = (move |x: &mut i32| {
            count1 += 1;
            *x *= 2;
            count1
        })
        .and_then(move |x: &mut i32| {
            count2 += 1;
            *x += 10;
            count2
        });

        let mut value = 5;
        assert_eq!(chained.apply(&mut value), 1);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_closure_map() {
        let mut count = 0;
        let mut mapped = (move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        })
        .map(|count| format!("Call #{}", count));

        let mut value = 5;
        assert_eq!(mapped.apply(&mut value), "Call #1");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_box() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut boxed = closure.into_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_rc() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut rc = closure.into_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_arc() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut arc = closure.into_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_box() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut boxed = closure.to_box();

        let mut value = 5;
        assert_eq!(boxed.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_rc() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut rc = closure.to_rc();

        let mut value = 5;
        assert_eq!(rc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_arc() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut arc = closure.to_arc();

        let mut value = 5;
        assert_eq!(arc.apply(&mut value), 1);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_fn() {
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x *= 2;
            count
        };
        let mut fn_closure = closure.to_fn();

        let mut value = 5;
        assert_eq!(fn_closure(&mut value), 1);
        assert_eq!(value, 10);
    }
}
