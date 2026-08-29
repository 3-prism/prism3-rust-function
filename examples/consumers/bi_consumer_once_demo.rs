// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! BiConsumerOnce demonstration
//!
//! This example demonstrates the usage of BoxBiConsumerOnce type, which
//! consumes itself on first call.

use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::BiConsumerOnce;
use qubit_function::BoxBiConsumerOnce;

fn main() {
    println!("=== BiConsumerOnce Demo ===\n");

    // 1. Basic usage
    println!("1. Basic usage:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
        println!("  Sum: {}", x + y);
    });
    consumer.accept(&10, &5);
    println!("  Log: {:?}\n", *log.lock().expect("mutex should not be poisoned"));

    // 2. Method chaining
    println!("2. Method chaining:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l1 = log.clone();
    let l2 = log.clone();
    let chained = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
        l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        println!("  First: sum={}", x + y);
    })
    .and_then(move |x: &i32, y: &i32| {
        l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        println!("  Second: product={}", x * y);
    });
    chained.accept(&5, &3);
    println!("  Log: {:?}\n", *log.lock().expect("mutex should not be poisoned"));

    // 3. Conditional execution - true case
    println!("3. Conditional execution - true case:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let conditional = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    conditional.accept(&5, &3);
    println!(
        "  Positive values: {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 4. Conditional execution - false case
    println!("4. Conditional execution - false case:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let conditional = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0);
    conditional.accept(&-5, &3);
    println!(
        "  Negative value (unchanged): {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 5. Conditional branching
    println!("5. Conditional branching:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l1 = log.clone();
    let l2 = log.clone();
    let branch = BoxBiConsumerOnce::new(move |x: &i32, _y: &i32| {
        l1.lock().expect("mutex should not be poisoned").push(*x);
    })
    .when(|x: &i32, y: &i32| *x > *y)
    .or_else(move |_x: &i32, y: &i32| {
        l2.lock().expect("mutex should not be poisoned").push(*y);
    });
    branch.accept(&15, &10);
    println!(
        "  When x > y: {:?}\n",
        *log.lock().expect("mutex should not be poisoned")
    );

    // 6. Working with closures directly
    println!("6. Working with closures directly:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let closure = move |x: &i32, y: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
        println!("  Processed: {}", x + y);
    };
    closure.accept(&10, &20);
    println!("  Log: {:?}\n", *log.lock().expect("mutex should not be poisoned"));

    // 7. Moving captured values
    println!("7. Moving captured values:");
    let data = vec![1, 2, 3, 4, 5];
    let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
        println!("  x={}, y={}", x, y);
        println!("  Captured data: {:?}", data);
        println!("  Data sum: {}", data.iter().sum::<i32>());
    });
    consumer.accept(&5, &3);
    // data is no longer available here
    println!();

    // 8. Initialization callback
    println!("8. Initialization callback:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let init_callback = BoxBiConsumerOnce::new(move |width: &i32, height: &i32| {
        println!("  Initializing with dimensions: {}x{}", width, height);
        l.lock().expect("mutex should not be poisoned").push(*width * *height);
    });
    init_callback.accept(&800, &600);
    println!("  Areas: {:?}\n", *log.lock().expect("mutex should not be poisoned"));

    // 9. Cleanup callback
    println!("9. Cleanup callback:");
    let cleanup = BoxBiConsumerOnce::new(|count: &i32, total: &i32| {
        println!("  Cleanup: processed {} out of {} items", count, total);
        println!("  Success rate: {:.1}%", (*count as f64 / *total as f64) * 100.0);
    });
    cleanup.accept(&85, &100);
    println!();

    // 10. Name support
    println!("10. Name support:");
    let mut named_consumer = BoxBiConsumerOnce::<i32, i32>::noop();
    println!("  Initial name: {:?}", named_consumer.name());

    named_consumer.set_name("init_callback");
    println!("  After setting name: {:?}", named_consumer.name());
    println!("  Display: {}", named_consumer);
    named_consumer.accept(&1, &2);
    println!();

    // 11. Print helpers
    println!("11. Print helpers:");
    let print = BoxBiConsumerOnce::new(|x: &i32, y: &i32| println!("{}, {}", x, y));
    print.accept(&42, &10);

    let print_with = BoxBiConsumerOnce::new(|x: &i32, y: &i32| println!("Dimensions: {}, {}", x, y));
    print_with.accept(&800, &600);
    println!();

    // 12. Converting to function
    println!("12. Converting to function:");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
        l.lock().expect("mutex should not be poisoned").push(*x + *y);
    });
    consumer.accept(&7, &3);
    println!("  Log: {:?}\n", *log.lock().expect("mutex should not be poisoned"));

    println!("=== Demo Complete ===");
}
