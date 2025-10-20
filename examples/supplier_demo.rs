/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Demonstration of Supplier types usage

use prism3_function::{
    ArcSupplier, BoxSupplier, BoxSupplierOnce, RcSupplier, Supplier, SupplierOnce,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    println!("=== Supplier Demo ===\n");

    demo_closure_supplier();
    demo_box_supplier_basics();
    demo_box_supplier_methods();
    demo_box_supplier_once();
    demo_arc_supplier();
    demo_arc_supplier_threading();
    demo_rc_supplier();
    demo_type_conversions();
}

fn demo_closure_supplier() {
    println!("--- Closure as Supplier ---");

    // Simple closure
    let closure = || 42;
    let mut boxed = Supplier::into_box(closure);
    println!("Closure: {}", boxed.get());

    // Stateful closure
    let mut counter = 0;
    let stateful = move || {
        counter += 1;
        counter
    };
    let mut boxed_stateful = Supplier::into_box(stateful);
    println!("Counter: {}", boxed_stateful.get());
    println!("Counter: {}", boxed_stateful.get());
    println!("Counter: {}", boxed_stateful.get());
    println!();
}

fn demo_box_supplier_basics() {
    println!("--- BoxSupplier Basics ---");

    // Basic usage
    let mut supplier = BoxSupplier::new(|| 42);
    println!("Basic: {}", supplier.get());

    // Constant supplier
    let mut constant = BoxSupplier::constant(100);
    println!("Constant: {}", constant.get());
    println!("Constant: {}", constant.get());

    // Stateful counter
    let mut counter = 0;
    let mut counter_supplier = BoxSupplier::new(move || {
        counter += 1;
        counter
    });
    println!("Counter: {}", counter_supplier.get());
    println!("Counter: {}", counter_supplier.get());
    println!("Counter: {}", counter_supplier.get());
    println!();
}

fn demo_box_supplier_methods() {
    println!("--- BoxSupplier Methods ---");

    // Map
    let mut mapped = BoxSupplier::new(|| 10).map(|x| x * 2).map(|x| x + 5);
    println!("Mapped: {}", mapped.get());

    // Filter
    let mut counter = 0;
    let mut filtered = BoxSupplier::new(move || {
        counter += 1;
        counter
    })
    .filter(|x| x % 2 == 0);
    println!("Filtered (odd): {:?}", filtered.get());
    println!("Filtered (even): {:?}", filtered.get());

    // Zip
    let first = BoxSupplier::new(|| 42);
    let second = BoxSupplier::new(|| "hello");
    let mut zipped = first.zip(second);
    println!("Zipped: {:?}", zipped.get());

    // Memoize
    let mut call_count = 0;
    let mut memoized = BoxSupplier::new(move || {
        call_count += 1;
        println!("  Expensive computation #{}", call_count);
        42
    })
    .memoize();
    println!("First call: {}", memoized.get());
    println!("Second call (cached): {}", memoized.get());
    println!();
}

fn demo_box_supplier_once() {
    println!("--- BoxSupplierOnce ---");

    // Basic usage
    let once = BoxSupplierOnce::new(|| {
        println!("  Expensive initialization");
        42
    });
    println!("Value: {}", once.get());

    // Moving captured values
    let data = String::from("Hello, World!");
    let once = BoxSupplierOnce::new(move || data);
    println!("Moved data: {}", once.get());
    println!();
}

fn demo_arc_supplier() {
    println!("--- ArcSupplier ---");

    // Basic usage
    let supplier = ArcSupplier::new(|| 42);
    let mut s = supplier;
    println!("Basic: {}", s.get());

    // Reusable transformations
    let source = ArcSupplier::new(|| 10);
    let doubled = source.map(|x| x * 2);
    let tripled = source.map(|x| x * 3);

    let mut s = source;
    let mut d = doubled;
    let mut t = tripled;
    println!("Source: {}", s.get());
    println!("Doubled: {}", d.get());
    println!("Tripled: {}", t.get());

    // Memoization
    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = Arc::clone(&call_count);
    let source = ArcSupplier::new(move || {
        let mut c = call_count_clone.lock().unwrap();
        *c += 1;
        println!("  Computation #{}", *c);
        42
    });
    let memoized = source.memoize();

    let mut m = memoized;
    println!("First call: {}", m.get());
    println!("Second call (cached): {}", m.get());
    println!();
}

fn demo_arc_supplier_threading() {
    println!("--- ArcSupplier Threading ---");

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    let supplier = ArcSupplier::new(move || {
        let mut c = counter_clone.lock().unwrap();
        *c += 1;
        *c
    });

    let mut s1 = supplier.clone();
    let mut s2 = supplier.clone();
    let mut s3 = supplier;

    let h1 = thread::spawn(move || {
        let v1 = s1.get();
        let v2 = s1.get();
        println!("Thread 1: {} {}", v1, v2);
        (v1, v2)
    });

    let h2 = thread::spawn(move || {
        let v1 = s2.get();
        let v2 = s2.get();
        println!("Thread 2: {} {}", v1, v2);
        (v1, v2)
    });

    let v1 = s3.get();
    let v2 = s3.get();
    println!("Main thread: {} {}", v1, v2);

    h1.join().unwrap();
    h2.join().unwrap();

    println!("Final counter: {}", *counter.lock().unwrap());
    println!();
}

fn demo_rc_supplier() {
    println!("--- RcSupplier ---");

    // Basic usage
    let supplier = RcSupplier::new(|| 42);
    let mut s = supplier;
    println!("Basic: {}", s.get());

    // Shared state
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let supplier = RcSupplier::new(move || {
        let mut c = counter_clone.borrow_mut();
        *c += 1;
        *c
    });

    let mut s1 = supplier.clone();
    let mut s2 = supplier.clone();

    println!("First clone: {}", s1.get());
    println!("Second clone: {}", s2.get());
    println!("First clone again: {}", s1.get());

    // Reusable transformations
    let source = RcSupplier::new(|| 10);
    let doubled = source.map(|x| x * 2);
    let tripled = source.map(|x| x * 3);
    let squared = source.map(|x| x * x);

    let mut s = source;
    let mut d = doubled;
    let mut t = tripled;
    let mut sq = squared;

    println!("Source: {}", s.get());
    println!("Doubled: {}", d.get());
    println!("Tripled: {}", t.get());
    println!("Squared: {}", sq.get());
    println!();
}

fn demo_type_conversions() {
    println!("--- Type Conversions ---");

    // Closure to Box
    let closure = || 42;
    let mut boxed = Supplier::into_box(closure);
    println!("Closure -> Box: {}", boxed.get());

    // Closure to Rc
    let closure = || 100;
    let mut rc = closure.into_rc();
    println!("Closure -> Rc: {}", rc.get());

    // Closure to Arc
    let closure = || 200;
    let mut arc = closure.into_arc();
    println!("Closure -> Arc: {}", arc.get());

    // Box to Rc
    let boxed = BoxSupplier::new(|| 42);
    let mut rc = boxed.into_rc();
    println!("Box -> Rc: {}", rc.get());

    // Arc to Box
    let arc = ArcSupplier::new(|| 42);
    let mut boxed = arc.into_box();
    println!("Arc -> Box: {}", boxed.get());

    // Rc to Box
    let rc = RcSupplier::new(|| 42);
    let mut boxed = rc.into_box();
    println!("Rc -> Box: {}", boxed.get());

    println!();
}
