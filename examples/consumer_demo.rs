/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Consumer type demonstration
//!
//! This example demonstrates the three implementations of Consumer (BoxConsumer, ArcConsumer, RcConsumer)
//! and their various usage patterns.
//!
//! Consumer is used to consume (read) values without modifying the original value.
//! If you need to modify values, please refer to mutator_demo.rs

use prism3_function::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    println!("=== Consumer Examples ===\n");
    println!("Note: Consumer only reads values, does not modify the original value");
    println!("If you need to modify values, please refer to mutator_demo.rs\n");

    // ========================================================================
    // Example 1: BoxConsumer basic usage
    // ========================================================================
    println!("Example 1: BoxConsumer basic usage");
    println!("{}", "-".repeat(50));

    let mut consumer = BoxConsumer::new(|x: &i32| {
        println!("Read and calculate: {} * 2 = {}", x, x * 2);
    });
    let value = 5;
    println!("Value: {}", value);
    consumer.accept(&value);
    println!("Value remains: {} (not modified)\n", value);

    // ========================================================================
    // Example 2: BoxConsumer method chaining
    // ========================================================================
    println!("Example 2: BoxConsumer method chaining");
    println!("{}", "-".repeat(50));

    let results = Arc::new(Mutex::new(Vec::new()));
    let r1 = results.clone();
    let r2 = results.clone();
    let r3 = results.clone();

    let mut chained = BoxConsumer::new(move |x: &i32| {
        r1.lock().unwrap().push(*x * 2);
    })
    .and_then(move |x: &i32| {
        r2.lock().unwrap().push(*x + 10);
    })
    .and_then(move |x: &i32| {
        r3.lock().unwrap().push(*x);
        println!("Processing value: {}", x);
    });

    let value = 5;
    println!("Initial value: {}", value);
    chained.accept(&value);
    println!("Collected results: {:?}", *results.lock().unwrap());
    println!("Original value: {} (not modified)\n", value);

    // ========================================================================
    // Example 3: Closure extension methods
    // ========================================================================
    println!("Example 3: Direct use of extension methods on closures");
    println!("{}", "-".repeat(50));

    let result = Arc::new(Mutex::new(0));
    let r1 = result.clone();
    let r2 = result.clone();

    let mut closure_chain = (move |x: &i32| {
        *r1.lock().unwrap() = *x * 2;
    })
    .and_then(move |_x: &i32| {
        *r2.lock().unwrap() += 10;
    });

    let value = 5;
    println!("Initial value: {}", value);
    closure_chain.accept(&value);
    println!("Calculation result: {}", *result.lock().unwrap());
    println!("Original value: {} (not modified)\n", value);

    // ========================================================================
    // Example 4: BoxConsumer factory methods
    // ========================================================================
    println!("Example 4: BoxConsumer factory methods");
    println!("{}", "-".repeat(50));

    // noop
    println!("noop - does nothing:");
    let mut noop = BoxConsumer::<i32>::noop();
    let value = 42;
    noop.accept(&value);
    println!("Value: {}\n", value);

    // print
    print!("print - prints value: ");
    let mut print = BoxConsumer::<i32>::print();
    let value = 42;
    print.accept(&value);
    println!();

    // print_with
    let mut print_with = BoxConsumer::<i32>::print_with("Value is: ");
    let value = 42;
    print_with.accept(&value);
    println!();

    // ========================================================================
    // Example 5: Conditional Consumer
    // ========================================================================
    println!("Example 5: Conditional Consumer");
    println!("{}", "-".repeat(50));

    // if_then
    let mut check_positive =
        BoxConsumer::if_then(|x: &i32| *x > 0, |x: &i32| println!("Positive: {}", x));

    let positive = 5;
    let negative = -5;
    print!("Check {}: ", positive);
    check_positive.accept(&positive);
    print!("Check {}: ", negative);
    check_positive.accept(&negative);
    println!("(negative numbers not printed)\n");

    // if_then_else
    let mut categorize = BoxConsumer::if_then_else(
        |x: &i32| *x > 0,
        |x: &i32| println!("Positive: {}", x),
        |x: &i32| println!("Non-positive: {}", x),
    );

    let positive = 10;
    let negative = -10;
    categorize.accept(&positive);
    categorize.accept(&negative);
    println!();

    // ========================================================================
    // Example 6: ArcConsumer - multi-threaded sharing
    // ========================================================================
    println!("Example 6: ArcConsumer - multi-threaded sharing");
    println!("{}", "-".repeat(50));

    let shared = ArcConsumer::new(|x: &i32| println!("Processing value: {}", x * 2));

    // Clone for another thread
    let shared_clone = shared.clone();
    let handle = thread::spawn(move || {
        let value = 5;
        let mut consumer = shared_clone;
        consumer.accept(&value);
        value
    });

    // Use in main thread
    let value = 3;
    let mut consumer = shared;
    consumer.accept(&value);

    let thread_result = handle.join().unwrap();
    println!("Thread result: {}\n", thread_result);

    // ========================================================================
    // Example 7: ArcConsumer composition (does not consume original consumer)
    // ========================================================================
    println!("Example 7: ArcConsumer composition (borrowing &self)");
    println!("{}", "-".repeat(50));

    let double = ArcConsumer::new(|x: &i32| println!("double: {}", x * 2));
    let add_ten = ArcConsumer::new(|x: &i32| println!("add_ten: {}", x + 10));

    // Composition does not consume original consumer
    let pipeline1 = double.and_then(&add_ten);
    let pipeline2 = add_ten.and_then(&double);

    let value1 = 5;
    let mut p1 = pipeline1;
    print!("pipeline1 processing 5: ");
    p1.accept(&value1);

    let value2 = 5;
    let mut p2 = pipeline2;
    print!("pipeline2 processing 5: ");
    p2.accept(&value2);

    // double and add_ten are still available
    let value3 = 10;
    let mut d = double;
    print!("Original double still available, processing 10: ");
    d.accept(&value3);
    println!();

    // ========================================================================
    // Example 8: RcConsumer - single-threaded sharing
    // ========================================================================
    println!("Example 8: RcConsumer - single-threaded sharing");
    println!("{}", "-".repeat(50));

    let rc_consumer = RcConsumer::new(|x: &i32| println!("Processing: {}", x * 2));

    // Clone multiple copies
    let clone1 = rc_consumer.clone();
    let clone2 = rc_consumer.clone();

    let value1 = 5;
    let mut c1 = clone1;
    print!("clone1 processing 5: ");
    c1.accept(&value1);

    let value2 = 3;
    let mut c2 = clone2;
    print!("clone2 processing 3: ");
    c2.accept(&value2);

    let value3 = 7;
    let mut c3 = rc_consumer;
    print!("Original processing 7: ");
    c3.accept(&value3);
    println!();

    // ========================================================================
    // Example 9: RcConsumer composition (borrowing &self)
    // ========================================================================
    println!("Example 9: RcConsumer composition (borrowing &self)");
    println!("{}", "-".repeat(50));

    let double = RcConsumer::new(|x: &i32| println!("double: {}", x * 2));
    let add_ten = RcConsumer::new(|x: &i32| println!("add_ten: {}", x + 10));

    let pipeline1 = double.and_then(&add_ten);
    let pipeline2 = add_ten.and_then(&double);

    let value1 = 5;
    let mut p1 = pipeline1;
    print!("pipeline1 processing 5: ");
    p1.accept(&value1);

    let value2 = 5;
    let mut p2 = pipeline2;
    print!("pipeline2 processing 5: ");
    p2.accept(&value2);
    println!();

    // ========================================================================
    // Example 10: Unified Consumer trait
    // ========================================================================
    println!("Example 10: Unified Consumer trait");
    println!("{}", "-".repeat(50));

    fn log_all<C: Consumer<i32>>(consumer: &mut C, values: &[i32]) {
        for value in values.iter() {
            consumer.accept(value);
        }
    }

    let values = vec![1, 2, 3, 4, 5];

    let mut box_con = BoxConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("BoxConsumer processing {:?}: ", values);
    log_all(&mut box_con, &values);
    println!();

    let mut arc_con = ArcConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("ArcConsumer processing {:?}: ", values);
    log_all(&mut arc_con, &values);
    println!();

    let mut rc_con = RcConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("RcConsumer processing {:?}: ", values);
    log_all(&mut rc_con, &values);
    println!();

    let mut closure = |x: &i32| print!("{} ", x * 2);
    print!("Closure processing {:?}: ", values);
    log_all(&mut closure, &values);
    println!("\n");

    // ========================================================================
    // Example 11: Data validation and logging
    // ========================================================================
    println!("Example 11: Data validation and logging");
    println!("{}", "-".repeat(50));

    let validator = BoxConsumer::new(|x: &i32| {
        let status = if *x >= 0 && *x <= 100 {
            "valid"
        } else {
            "out of range"
        };
        println!("Validate {}: {}", x, status);
    });

    let logger = BoxConsumer::new(|x: &i32| {
        println!("Log to file: value={}, square={}", x, x * x);
    });

    let mut pipeline = validator.and_then(logger);

    let test_values = vec![-50, 30, 200];
    for value in test_values {
        pipeline.accept(&value);
    }
    println!();

    // ========================================================================
    // Example 12: String analysis
    // ========================================================================
    println!("Example 12: String analysis");
    println!("{}", "-".repeat(50));

    let mut string_analyzer = BoxConsumer::new(|s: &String| {
        println!("Length: {}", s.len());
    })
    .and_then(|s: &String| {
        println!("Lowercase: {}", s.to_lowercase());
    })
    .and_then(|s: &String| {
        println!("Uppercase: {}", s.to_uppercase());
    })
    .and_then(|s: &String| {
        let word_count = s.split_whitespace().count();
        println!("Word count: {}", word_count);
    });

    let text = String::from("Hello World");
    println!("Analyzing text: \"{}\"", text);
    string_analyzer.accept(&text);
    println!("Original text: \"{}\" (not modified)\n", text);

    // ========================================================================
    // Example 13: Type conversion
    // ========================================================================
    println!("Example 13: Type conversion");
    println!("{}", "-".repeat(50));

    // Closure -> BoxConsumer
    let closure = |x: &i32| print!("Processing: {} ", x * 2);
    let mut box_con = closure.into_box();
    let value = 5;
    print!("Closure -> BoxConsumer: ");
    box_con.accept(&value);
    println!();

    // Closure -> RcConsumer
    let closure = |x: &i32| print!("Processing: {} ", x * 2);
    let mut rc_con = closure.into_rc();
    let value = 5;
    print!("Closure -> RcConsumer: ");
    rc_con.accept(&value);
    println!();

    // Closure -> ArcConsumer
    let closure = |x: &i32| print!("Processing: {} ", x * 2);
    let mut arc_con = closure.into_arc();
    let value = 5;
    print!("Closure -> ArcConsumer: ");
    arc_con.accept(&value);
    println!();

    // BoxConsumer -> RcConsumer
    let box_con = BoxConsumer::new(|x: &i32| print!("Processing: {} ", x * 2));
    let mut rc_con = box_con.into_rc();
    let value = 5;
    print!("BoxConsumer -> RcConsumer: ");
    rc_con.accept(&value);
    println!();

    // RcConsumer -> BoxConsumer
    let rc_con = RcConsumer::new(|x: &i32| print!("Processing: {} ", x * 2));
    let mut box_con = rc_con.into_box();
    let value = 5;
    print!("RcConsumer -> BoxConsumer: ");
    box_con.accept(&value);
    println!("\n");

    // ========================================================================
    // Example 14: Custom types
    // ========================================================================
    println!("Example 14: Custom types");
    println!("{}", "-".repeat(50));

    #[derive(Debug, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    let mut analyzer = BoxConsumer::new(|p: &Point| {
        println!("Point coordinates: ({}, {})", p.x, p.y);
    })
    .and_then(|p: &Point| {
        let distance = ((p.x * p.x + p.y * p.y) as f64).sqrt();
        println!("Distance from origin: {:.2}", distance);
    })
    .and_then(|p: &Point| {
        let quadrant = match (p.x >= 0, p.y >= 0) {
            (true, true) => "First quadrant",
            (false, true) => "Second quadrant",
            (false, false) => "Third quadrant",
            (true, false) => "Fourth quadrant",
        };
        println!("Quadrant: {}", quadrant);
    });

    let point = Point { x: 3, y: 4 };
    println!("Analyzing point: {:?}", point);
    analyzer.accept(&point);
    println!("Original point: {:?} (not modified)\n", point);

    // ========================================================================
    // Example 15: Data collection and statistics
    // ========================================================================
    println!("Example 15: Data collection and statistics");
    println!("{}", "-".repeat(50));

    let sum = Arc::new(Mutex::new(0));
    let count = Arc::new(Mutex::new(0));
    let sum_clone = sum.clone();
    let count_clone = count.clone();

    let mut collector = BoxConsumer::new(move |x: &i32| {
        *sum_clone.lock().unwrap() += *x;
        *count_clone.lock().unwrap() += 1;
    });

    let numbers = vec![10, 20, 30, 40, 50];
    println!("Numbers: {:?}", numbers);
    for num in &numbers {
        collector.accept(num);
    }

    let total = *sum.lock().unwrap();
    let cnt = *count.lock().unwrap();
    println!("Sum: {}", total);
    println!("Count: {}", cnt);
    println!("Average: {:.2}\n", total as f64 / cnt as f64);

    println!("=== All examples completed ===");
    println!("\nTip: For value modification functionality, please refer to mutator_demo.rs");
}
