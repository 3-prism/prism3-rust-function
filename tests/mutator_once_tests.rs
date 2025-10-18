/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatorOnce Tests
//!
//! 测试 MutatorOnce trait 及其实现的完整功能。

use prism3_function::{BoxMutatorOnce, FnMutatorOnceOps, MutatorOnce};

// ============================================================================
// BoxMutatorOnce Tests
// ============================================================================

#[cfg(test)]
mod box_mutator_once_tests {
    use super::*;

    #[test]
    fn test_new_basic() {
        let data = vec![1, 2, 3];
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data);
        });

        let mut target = vec![0];
        mutator.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_new_with_string() {
        let suffix = String::from("world");
        let mutator = BoxMutatorOnce::new(move |x: &mut String| {
            x.push(' ');
            x.push_str(&suffix);
        });

        let mut target = String::from("hello");
        mutator.mutate(&mut target);
        assert_eq!(target, "hello world");
    }

    #[test]
    fn test_new_with_complex_operation() {
        let data = [10, 20, 30];
        let multiplier = 2;
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data.iter().map(|&n| n * multiplier));
        });

        let mut target = vec![0];
        mutator.mutate(&mut target);
        assert_eq!(target, vec![0, 20, 40, 60]);
    }

    #[test]
    fn test_noop() {
        let noop = BoxMutatorOnce::<i32>::noop();
        let mut value = 42;
        noop.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_with_vec() {
        let noop = BoxMutatorOnce::<Vec<i32>>::noop();
        let mut value = vec![1, 2, 3];
        noop.mutate(&mut value);
        assert_eq!(value, vec![1, 2, 3]);
    }

    #[test]
    fn test_and_then_two_operations() {
        let data1 = vec![1, 2];
        let data2 = vec![3, 4];

        let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data1);
        })
        .and_then(move |x: &mut Vec<i32>| {
            x.extend(data2);
        });

        let mut target = vec![0];
        chained.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_and_then_three_operations() {
        let data1 = vec![1, 2];
        let data2 = vec![3, 4];
        let data3 = vec![5, 6];

        let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data1);
        })
        .and_then(move |x: &mut Vec<i32>| {
            x.extend(data2);
        })
        .and_then(move |x: &mut Vec<i32>| {
            x.extend(data3);
        });

        let mut target = vec![0];
        chained.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_and_then_with_different_types() {
        let suffix = String::from("!");
        let prefix = String::from("Hello, ");

        let chained = BoxMutatorOnce::new(move |x: &mut String| {
            x.insert_str(0, &prefix);
        })
        .and_then(move |x: &mut String| {
            x.push_str(&suffix);
        });

        let mut target = String::from("world");
        chained.mutate(&mut target);
        assert_eq!(target, "Hello, world!");
    }

    #[test]
    fn test_and_then_with_closure() {
        let data = vec![1, 2, 3];

        let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data);
        })
        .and_then(|x: &mut Vec<i32>| {
            x.iter_mut().for_each(|n| *n *= 2);
        });

        let mut target = vec![0];
        chained.mutate(&mut target);
        assert_eq!(target, vec![0, 2, 4, 6]);
    }

    #[test]
    fn test_into_box() {
        let data = vec![1, 2, 3];
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data);
        });

        let boxed = mutator.into_box();
        let mut target = vec![0];
        boxed.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }
}

// ============================================================================
// MutatorOnce Trait Tests
// ============================================================================

#[cfg(test)]
mod mutator_once_trait_tests {
    use super::*;

    #[test]
    fn test_closure_implements_mutator_once() {
        let data = vec![1, 2, 3];
        let closure = move |x: &mut Vec<i32>| x.extend(data);

        let mut target = vec![0];
        closure.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_closure_into_box() {
        let data = vec![1, 2, 3];
        let closure = move |x: &mut Vec<i32>| x.extend(data);

        let boxed = closure.into_box();
        let mut target = vec![0];
        boxed.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_generic_function() {
        fn apply_once<M: MutatorOnce<Vec<i32>>>(mutator: M, initial: Vec<i32>) -> Vec<i32> {
            let mut val = initial;
            mutator.mutate(&mut val);
            val
        }

        let data = vec![1, 2, 3];
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data);
        });

        let result = apply_once(mutator, vec![0]);
        assert_eq!(result, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_generic_function_with_closure() {
        fn apply_once<M: MutatorOnce<Vec<i32>>>(mutator: M, initial: Vec<i32>) -> Vec<i32> {
            let mut val = initial;
            mutator.mutate(&mut val);
            val
        }

        let data = vec![1, 2, 3];
        let result = apply_once(move |x: &mut Vec<i32>| x.extend(data), vec![0]);
        assert_eq!(result, vec![0, 1, 2, 3]);
    }
}

// ============================================================================
// FnMutatorOnceOps Tests
// ============================================================================

#[cfg(test)]
mod fn_mutator_once_ops_tests {
    use super::*;

    #[test]
    fn test_closure_and_then() {
        let data1 = vec![1, 2];
        let data2 = vec![3, 4];

        let chained = (move |x: &mut Vec<i32>| x.extend(data1))
            .and_then(move |x: &mut Vec<i32>| x.extend(data2));

        let mut target = vec![0];
        chained.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_closure_chain_multiple() {
        let data1 = vec![1];
        let data2 = vec![2];
        let data3 = vec![3];

        let chained = (move |x: &mut Vec<i32>| x.extend(data1))
            .and_then(move |x: &mut Vec<i32>| x.extend(data2))
            .and_then(move |x: &mut Vec<i32>| x.extend(data3));

        let mut target = vec![0];
        chained.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_closure_and_then_with_box_mutator() {
        let data1 = vec![1, 2];
        let data2 = vec![3, 4];

        let boxed = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data2);
        });

        let chained = (move |x: &mut Vec<i32>| x.extend(data1)).and_then(boxed);

        let mut target = vec![0];
        chained.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3, 4]);
    }
}

// ============================================================================
// Real-World Usage Tests
// ============================================================================

#[cfg(test)]
mod real_world_tests {
    use super::*;

    // 模拟初始化器模式
    struct Initializer {
        on_complete: Option<BoxMutatorOnce<Vec<i32>>>,
    }

    impl Initializer {
        fn new<F>(callback: F) -> Self
        where
            F: FnOnce(&mut Vec<i32>) + 'static,
        {
            Self {
                on_complete: Some(BoxMutatorOnce::new(callback)),
            }
        }

        fn run(mut self, data: &mut Vec<i32>) {
            // 执行初始化逻辑
            data.push(42);

            // 调用回调
            if let Some(callback) = self.on_complete.take() {
                callback.mutate(data);
            }
        }
    }

    #[test]
    fn test_initializer_pattern() {
        let data_to_add = vec![1, 2, 3];
        let init = Initializer::new(move |x| {
            x.extend(data_to_add);
        });

        let mut result = Vec::new();
        init.run(&mut result);
        assert_eq!(result, vec![42, 1, 2, 3]);
    }

    #[test]
    fn test_resource_transfer() {
        let large_data = vec![1; 1000];
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(large_data); // 移动大型数据而非克隆
        });

        let mut target = Vec::new();
        mutator.mutate(&mut target);
        assert_eq!(target.len(), 1000);
    }

    #[test]
    fn test_config_builder_with_callback() {
        struct Config {
            values: Vec<String>,
        }

        impl Config {
            fn new() -> Self {
                Self { values: Vec::new() }
            }

            fn build<F>(mut self, finalizer: F) -> Self
            where
                F: FnOnce(&mut Vec<String>) + 'static,
            {
                finalizer.mutate(&mut self.values);
                self
            }
        }

        let extra_values = vec!["extra1".to_string(), "extra2".to_string()];
        let config = Config::new().build(move |values| {
            values.extend(extra_values);
        });

        assert_eq!(config.values, vec!["extra1", "extra2"]);
    }

    #[test]
    fn test_chain_with_side_effects() {
        let mut counter = 0i32;
        let data = vec![1, 2, 3];

        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(data);
        })
        .and_then(move |x: &mut Vec<i32>| {
            counter += x.len() as i32; // 注意：这里 counter 被移动，但不会产生预期效果
            x.push(counter); // 实际上 counter 在闭包内是独立的
        });

        let mut target = vec![0];
        mutator.mutate(&mut target);
        // counter 在闭包内递增，但外部 counter 不受影响
        assert_eq!(target.len(), 5); // 0, 1, 2, 3, 加上闭包内的 counter
    }

    #[test]
    fn test_string_builder_pattern() {
        let prefix = String::from("Hello, ");
        let suffix = String::from("!");

        let builder = BoxMutatorOnce::new(move |s: &mut String| {
            s.insert_str(0, &prefix);
        })
        .and_then(move |s: &mut String| {
            s.push_str(&suffix);
        })
        .and_then(|s: &mut String| {
            *s = s.to_uppercase();
        });

        let mut text = String::from("world");
        builder.mutate(&mut text);
        assert_eq!(text, "HELLO, WORLD!");
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_empty_vec_move() {
        let empty_vec: Vec<i32> = Vec::new();
        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(empty_vec);
        });

        let mut target = vec![1, 2, 3];
        mutator.mutate(&mut target);
        assert_eq!(target, vec![1, 2, 3]);
    }

    #[test]
    fn test_empty_string_move() {
        let empty = String::new();
        let mutator = BoxMutatorOnce::new(move |x: &mut String| {
            x.push_str(&empty);
        });

        let mut target = String::from("test");
        mutator.mutate(&mut target);
        assert_eq!(target, "test");
    }

    #[test]
    fn test_noop_chain() {
        let noop = BoxMutatorOnce::<i32>::noop().and_then(BoxMutatorOnce::<i32>::noop());

        let mut value = 42;
        noop.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_large_chain() {
        let mut chained = BoxMutatorOnce::new(|x: &mut i32| *x += 1);

        for _ in 0..10 {
            chained = chained.and_then(|x: &mut i32| *x += 1);
        }

        let mut value = 0;
        chained.mutate(&mut value);
        assert_eq!(value, 11); // 初始 +1，然后 10 次 +1
    }

    #[test]
    fn test_move_multiple_values() {
        let vec1 = vec![1, 2];
        let vec2 = vec![3, 4];
        let vec3 = vec![5, 6];

        let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
            x.extend(vec1);
            x.extend(vec2);
            x.extend(vec3);
        });

        let mut target = vec![0];
        mutator.mutate(&mut target);
        assert_eq!(target, vec![0, 1, 2, 3, 4, 5, 6]);
    }
}
