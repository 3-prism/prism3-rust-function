/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! BiConsumer demonstration
//!
//! This example demonstrates the usage of BiConsumer types.

use prism3_function::{ArcBiConsumer, BiConsumer, BoxBiConsumer, RcBiConsumer};
use std::thread;

fn main() {
    println!("=== BiConsumer Demo ===\n");

    // 1. BoxBiConsumer - Single ownership
    println!("1. BoxBiConsumer - Single ownership:");
    let mut box_consumer = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| {
        println!("  Before: x={}, y={}", x, y);
        *x += *y;
        *y = 0;
        println!("  After: x={}, y={}", x, y);
    });
    let mut a = 10;
    let mut b = 5;
    box_consumer.accept(&mut a, &mut b);
    println!("  Result: a={}, b={}\n", a, b);

    // 2. Method chaining with BoxBiConsumer
    println!("2. BoxBiConsumer with method chaining:");
    let mut chained = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| {
        *x += *y;
        println!("  After first operation: x={}, y={}", x, y);
    })
    .and_then(|x: &mut i32, y: &mut i32| {
        *y *= 2;
        println!("  After second operation: x={}, y={}", x, y);
    });
    let mut c = 5;
    let mut d = 3;
    chained.accept(&mut c, &mut d);
    println!("  Final result: c={}, d={}\n", c, d);

    // 3. ArcBiConsumer - Thread-safe shared ownership
    println!("3. ArcBiConsumer - Thread-safe shared ownership:");
    let arc_consumer = ArcBiConsumer::new(|x: &mut i32, y: &mut i32| {
        *x = *x + *y;
        println!("  Thread {:?}: x={}, y={}", thread::current().id(), x, y);
    });

    let consumer1 = arc_consumer.clone();
    let consumer2 = arc_consumer.clone();

    let handle1 = thread::spawn(move || {
        let mut x = 10;
        let mut y = 5;
        let mut c = consumer1;
        c.accept(&mut x, &mut y);
        (x, y)
    });

    let handle2 = thread::spawn(move || {
        let mut x = 20;
        let mut y = 8;
        let mut c = consumer2;
        c.accept(&mut x, &mut y);
        (x, y)
    });

    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();
    println!("  Thread 1 result: {:?}", result1);
    println!("  Thread 2 result: {:?}\n", result2);

    // 4. RcBiConsumer - Single-threaded shared ownership
    println!("4. RcBiConsumer - Single-threaded shared ownership:");
    let rc_consumer = RcBiConsumer::new(|x: &mut i32, y: &mut i32| {
        *x = *x * 2;
        *y = *y + 1;
    });

    let mut clone1 = rc_consumer.clone();
    let mut clone2 = rc_consumer.clone();

    let mut x1 = 5;
    let mut y1 = 3;
    clone1.accept(&mut x1, &mut y1);
    println!("  First use: x1={}, y1={}", x1, y1);

    let mut x2 = 7;
    let mut y2 = 2;
    clone2.accept(&mut x2, &mut y2);
    println!("  Second use: x2={}, y2={}\n", x2, y2);

    // 5. Working with closures directly
    println!("5. Working with closures directly:");
    let mut closure = |x: &mut i32, y: &mut i32| {
        let sum = *x + *y;
        *x = sum;
        *y = sum;
    };
    let mut e = 10;
    let mut f = 20;
    closure.accept(&mut e, &mut f);
    println!("  After closure: e={}, f={}\n", e, f);

    // 6. Conditional BiConsumer
    println!("6. Conditional BiConsumer:");
    let mut conditional = BoxBiConsumer::if_then(
        |x: &i32, y: &i32| *x > 0 && *y > 0,
        |x: &mut i32, y: &mut i32| *x += *y,
    );

    let mut g = 5;
    let mut h = 3;
    conditional.accept(&mut g, &mut h);
    println!("  Positive values: g={} (5+3)", g);

    let mut i = -5;
    let mut j = 3;
    conditional.accept(&mut i, &mut j);
    println!("  Negative value: i={} (unchanged)\n", i);

    // 7. Conditional branch BiConsumer
    println!("7. Conditional branch BiConsumer:");
    let mut branch = BoxBiConsumer::if_then_else(
        |x: &i32, y: &i32| *x > *y,
        |x: &mut i32, _y: &mut i32| *x += 10,
        |_x: &mut i32, y: &mut i32| *y += 10,
    );

    let mut k = 15;
    let mut l = 10;
    branch.accept(&mut k, &mut l);
    println!("  When x > y: k={} (15+10)", k);

    let mut m = 5;
    let mut n = 10;
    branch.accept(&mut m, &mut n);
    println!("  When x <= y: n={} (10+10)\n", n);

    // 8. Using into_fn with iterators
    println!("8. Using into_fn with iterators:");
    let consumer = BoxBiConsumer::new(|x: &mut i32, y: &mut i32| *x += *y);
    let mut values1 = vec![1, 2, 3, 4, 5];
    let mut values2 = vec![10, 20, 30, 40, 50];

    values1
        .iter_mut()
        .zip(values2.iter_mut())
        .for_each(consumer.into_fn());

    println!("  values1: {:?}", values1);
    println!("  values2: {:?}", values2);

    println!("\n=== Demo Complete ===");
}
