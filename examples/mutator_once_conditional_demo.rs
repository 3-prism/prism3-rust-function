/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatorOnce Conditional Execution Demo
//!
//! Demonstrates conditional execution features of BoxMutatorOnce

use prism3_function::predicate::{BoxPredicate, FnPredicateOps};
use prism3_function::{BoxMutatorOnce, MutatorOnce};

fn main() {
    println!("=== MutatorOnce Conditional Execution Examples ===\n");

    // 1. Basic conditional execution - when condition is satisfied
    println!("1. Basic conditional execution - when condition is satisfied");
    let data = vec![1, 2, 3];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   Extending vector with data: {:?}", data);
        x.extend(data);
    });
    let conditional = mutator.when(|x: &Vec<i32>| {
        println!("   Checking condition: !x.is_empty()");
        !x.is_empty()
    });

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    conditional.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 2. Conditional execution - when condition is not satisfied
    println!("2. Conditional execution - when condition is not satisfied");
    let data = vec![4, 5, 6];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   This should not be executed");
        x.extend(data);
    });
    let conditional = mutator.when(|x: &Vec<i32>| {
        println!("   Checking condition: x.len() > 10");
        x.len() > 10
    });

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    conditional.mutate(&mut target);
    println!("   Result: {:?} (unchanged)\n", target);

    // 3. Using BoxPredicate
    println!("3. Using BoxPredicate");
    let pred = BoxPredicate::new(|x: &Vec<i32>| {
        println!("   Predicate: checking if vector is not empty");
        !x.is_empty()
    });
    let data = vec![7, 8, 9];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   Adding data: {:?}", data);
        x.extend(data);
    });
    let conditional = mutator.when(pred);

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    conditional.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 4. Using composed predicate
    println!("4. Using composed predicate");
    let pred = (|x: &Vec<i32>| {
        println!("   Condition 1: !x.is_empty()");
        !x.is_empty()
    })
    .and(|x: &Vec<i32>| {
        println!("   Condition 2: x.len() < 10");
        x.len() < 10
    });
    let data = vec![10, 11, 12];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   Adding data: {:?}", data);
        x.extend(data);
    });
    let conditional = mutator.when(pred);

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    conditional.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 5. If-then-else with or_else - when branch
    println!("5. If-then-else with or_else - when branch");
    let data1 = vec![1, 2, 3];
    let data2 = vec![99];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   When branch: adding {:?}", data1);
        x.extend(data1);
    })
    .when(|x: &Vec<i32>| {
        println!("   Checking: !x.is_empty()");
        !x.is_empty()
    })
    .or_else(move |x: &mut Vec<i32>| {
        println!("   Else branch: adding {:?}", data2);
        x.extend(data2);
    });

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    mutator.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 6. If-then-else with or_else - else branch
    println!("6. If-then-else with or_else - else branch");
    let data1 = vec![4, 5, 6];
    let data2 = vec![99];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   When branch: adding {:?}", data1);
        x.extend(data1);
    })
    .when(|x: &Vec<i32>| {
        println!("   Checking: x.is_empty()");
        x.is_empty()
    })
    .or_else(move |x: &mut Vec<i32>| {
        println!("   Else branch: adding {:?}", data2);
        x.extend(data2);
    });

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    mutator.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 7. Conditional with integers
    println!("7. Conditional with integers");
    let mutator = BoxMutatorOnce::new(|x: &mut i32| {
        println!("   Multiplying by 2");
        *x *= 2;
    })
    .when(|x: &i32| {
        println!("   Checking: *x > 0");
        *x > 0
    });

    let mut positive = 5;
    println!("   Initial (positive): {}", positive);
    mutator.mutate(&mut positive);
    println!("   Result: {}\n", positive);

    // 8. Conditional with integers - not executed
    println!("8. Conditional with integers - not executed");
    let mutator = BoxMutatorOnce::new(|x: &mut i32| {
        println!("   This should not be executed");
        *x *= 2;
    })
    .when(|x: &i32| {
        println!("   Checking: *x > 0");
        *x > 0
    });

    let mut negative = -5;
    println!("   Initial (negative): {}", negative);
    mutator.mutate(&mut negative);
    println!("   Result: {} (unchanged)\n", negative);

    // 9. Chaining conditional mutators
    println!("9. Chaining conditional mutators");
    let data1 = vec![1, 2];
    let cond1 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   First mutator: adding {:?}", data1);
        x.extend(data1);
    })
    .when(|x: &Vec<i32>| {
        println!("   First condition: !x.is_empty()");
        !x.is_empty()
    });

    let data2 = vec![3, 4];
    let cond2 = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   Second mutator: adding {:?}", data2);
        x.extend(data2);
    })
    .when(|x: &Vec<i32>| {
        println!("   Second condition: x.len() < 10");
        x.len() < 10
    });

    let chained = cond1.and_then(cond2);

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    chained.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 10. Complex conditional chain
    println!("10. Complex conditional chain");
    let data1 = vec![1, 2];
    let data2 = vec![99];
    let data3 = vec![5, 6];

    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   When branch: adding {:?}", data1);
        x.extend(data1);
    })
    .when(|x: &Vec<i32>| {
        println!("   Checking: !x.is_empty()");
        !x.is_empty()
    })
    .or_else(move |x: &mut Vec<i32>| {
        println!("   Else branch: adding {:?}", data2);
        x.extend(data2);
    })
    .and_then(move |x: &mut Vec<i32>| {
        println!("   Final step: adding {:?}", data3);
        x.extend(data3);
    });

    let mut target = vec![0];
    println!("   Initial: {:?}", target);
    mutator.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 11. Real-world scenario: data validation and processing
    println!("11. Real-world scenario: data validation and processing");

    struct DataProcessor {
        on_valid: Option<BoxMutatorOnce<Vec<String>>>,
        on_invalid: Option<BoxMutatorOnce<Vec<String>>>,
    }

    impl DataProcessor {
        fn new<V, I>(on_valid: V, on_invalid: I) -> Self
        where
            V: FnOnce(&mut Vec<String>) + 'static,
            I: FnOnce(&mut Vec<String>) + 'static,
        {
            Self {
                on_valid: Some(BoxMutatorOnce::new(on_valid)),
                on_invalid: Some(BoxMutatorOnce::new(on_invalid)),
            }
        }

        fn process(mut self, data: &mut Vec<String>) {
            let is_valid = !data.is_empty() && data.iter().all(|s| !s.is_empty());
            println!(
                "   Data validation: {}",
                if is_valid { "VALID" } else { "INVALID" }
            );

            if is_valid {
                if let Some(callback) = self.on_valid.take() {
                    callback.mutate(data);
                }
            } else if let Some(callback) = self.on_invalid.take() {
                callback.mutate(data);
            }
        }
    }

    let valid_suffix = vec!["processed".to_string()];
    let invalid_marker = vec!["[INVALID]".to_string()];

    let processor = DataProcessor::new(
        move |data| {
            println!("   Valid data callback: adding suffix");
            data.extend(valid_suffix);
        },
        move |data| {
            println!("   Invalid data callback: adding error marker");
            data.clear();
            data.extend(invalid_marker);
        },
    );

    let mut valid_data = vec!["item1".to_string(), "item2".to_string()];
    println!("   Processing valid data: {:?}", valid_data);
    processor.process(&mut valid_data);
    println!("   Result: {:?}\n", valid_data);

    println!("=== Examples completed ===");
}
