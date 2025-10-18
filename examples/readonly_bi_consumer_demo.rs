/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! ReadonlyBiConsumer demonstration
//!
//! This example demonstrates the usage of ReadonlyBiConsumer types, which
//! neither modify their own state nor the input values.

use prism3_function::{
    ArcReadonlyBiConsumer, BoxReadonlyBiConsumer, RcReadonlyBiConsumer, ReadonlyBiConsumer,
};
use std::rc::Rc;
use std::sync::{atomic::AtomicUsize, atomic::Ordering, Arc};
use std::thread;

fn main() {
    println!("=== ReadonlyBiConsumer Demo ===\n");

    // 1. BoxReadonlyBiConsumer - Single ownership
    println!("1. BoxReadonlyBiConsumer - Single ownership:");
    let box_consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  Values: x={}, y={}, sum={}", x, y, x + y);
    });
    box_consumer.accept(&10, &5);
    println!();

    // 2. Method chaining with BoxReadonlyBiConsumer
    println!("2. BoxReadonlyBiConsumer with method chaining:");
    let chained = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  First operation: x={}, y={}", x, y);
    })
    .and_then(|x: &i32, y: &i32| {
        println!("  Second operation: sum={}", x + y);
    })
    .and_then(|x: &i32, y: &i32| {
        println!("  Third operation: product={}", x * y);
    });
    chained.accept(&5, &3);
    println!();

    // 3. ArcReadonlyBiConsumer - Thread-safe shared ownership
    println!("3. ArcReadonlyBiConsumer - Thread-safe shared ownership:");
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
        c.fetch_add(1, Ordering::SeqCst);
        println!("  Thread {:?}: sum={}", thread::current().id(), x + y);
    });

    let consumer1 = arc_consumer.clone();
    let consumer2 = arc_consumer.clone();

    let handle1 = thread::spawn(move || {
        consumer1.accept(&10, &5);
    });

    let handle2 = thread::spawn(move || {
        consumer2.accept(&20, &8);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
    println!("  Total calls: {}\n", counter.load(Ordering::SeqCst));

    // 4. RcReadonlyBiConsumer - Single-threaded shared ownership
    println!("4. RcReadonlyBiConsumer - Single-threaded shared ownership:");
    let counter = Rc::new(std::cell::Cell::new(0));
    let c = counter.clone();
    let rc_consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
        c.set(c.get() + 1);
        println!("  Call {}: sum={}", c.get(), x + y);
    });

    let clone1 = rc_consumer.clone();
    let clone2 = rc_consumer.clone();

    clone1.accept(&5, &3);
    clone2.accept(&7, &2);
    println!("  Total calls: {}\n", counter.get());

    // 5. Working with closures directly
    println!("5. Working with closures directly:");
    let closure = |x: &i32, y: &i32| {
        println!("  x={}, y={}, product={}", x, y, x * y);
    };
    closure.accept(&10, &20);
    println!();

    // 6. Pure observation - logging
    println!("6. Pure observation - logging:");
    let logger = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  [LOG] Processing pair: ({}, {})", x, y);
    });
    logger.accept(&5, &3);
    logger.accept(&10, &7);
    println!();

    // 7. Chaining observations
    println!("7. Chaining observations:");
    let log_input = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  [INPUT] x={}, y={}", x, y);
    });
    let log_sum = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  [SUM] {}", x + y);
    });
    let log_product = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  [PRODUCT] {}", x * y);
    });

    let chained = log_input.and_then(log_sum).and_then(log_product);
    chained.accept(&5, &3);
    println!();

    // 8. ArcReadonlyBiConsumer - Reusability
    println!("8. ArcReadonlyBiConsumer - Reusability:");
    let first = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  First: x={}, y={}", x, y);
    });
    let second = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
        println!("  Second: sum={}", x + y);
    });

    // Both first and second can be reused after chaining
    let chained1 = first.and_then(&second);
    let chained2 = first.and_then(&second);

    println!("  Using chained1:");
    chained1.accept(&5, &3);

    println!("  Using chained2:");
    chained2.accept(&10, &2);
    println!();

    // 9. Name support
    println!("9. Name support:");
    let mut named_consumer = BoxReadonlyBiConsumer::<i32, i32>::noop();
    println!("  Initial name: {:?}", named_consumer.name());

    named_consumer.set_name("sum_logger");
    println!("  After setting name: {:?}", named_consumer.name());
    println!("  Display: {}\n", named_consumer);

    // 10. No-op consumer
    println!("10. No-op consumer:");
    let noop = BoxReadonlyBiConsumer::<i32, i32>::noop();
    noop.accept(&42, &10);
    println!("  No-op completed (no output expected)\n");

    println!("=== Demo Complete ===");
}
