/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Supplier Types Demo
//!
//! This example demonstrates the three Supplier implementations
//! (BoxSupplier, ArcSupplier, RcSupplier) and their various usage
//! patterns.

use prism3_function::{ArcSupplier, BoxSupplier, FnSupplierOps, RcSupplier, Supplier};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    println!("=== Supplier Examples ===\n");

    // ======================================================================
    // Example 1: BoxSupplier Basic Usage
    // ======================================================================
    println!("Example 1: BoxSupplier Basic Usage");
    println!("{}", "-".repeat(50));

    let mut supplier = BoxSupplier::new(|| 42);
    println!("BoxSupplier result: {}\n", supplier.get());

    // ======================================================================
    // Example 2: BoxSupplier with Constant
    // ======================================================================
    println!("Example 2: BoxSupplier with Constant");
    println!("{}", "-".repeat(50));

    let mut constant = BoxSupplier::constant(42);
    println!("First call: {}", constant.get());
    println!("Second call: {}", constant.get());
    println!("Third call: {}\n", constant.get());

    // ======================================================================
    // Example 3: BoxSupplier with Stateful Counter
    // ======================================================================
    println!("Example 3: BoxSupplier with Stateful Counter");
    println!("{}", "-".repeat(50));

    let mut counter = 0;
    let mut supplier = BoxSupplier::new(move || {
        counter += 1;
        counter
    });

    println!("Counter call 1: {}", supplier.get());
    println!("Counter call 2: {}", supplier.get());
    println!("Counter call 3: {}\n", supplier.get());

    // ======================================================================
    // Example 4: BoxSupplier with map
    // ======================================================================
    println!("Example 4: BoxSupplier with map");
    println!("{}", "-".repeat(50));

    let mut mapped = BoxSupplier::new(|| 10).map(|x| x * 2);
    println!("10 * 2 = {}\n", mapped.get());

    // ======================================================================
    // Example 5: BoxSupplier Method Chaining
    // ======================================================================
    println!("Example 5: BoxSupplier Method Chaining");
    println!("{}", "-".repeat(50));

    let mut chained = BoxSupplier::new(|| 5).map(|x| x * 2).map(|x| x + 5);

    println!("(5 * 2) + 5 = {}\n", chained.get());

    // ======================================================================
    // Example 6: BoxSupplier with filter
    // ======================================================================
    println!("Example 6: BoxSupplier with filter");
    println!("{}", "-".repeat(50));

    let mut counter = 0;
    let mut filtered = BoxSupplier::new(move || {
        counter += 1;
        counter
    })
    .filter(|x| x % 2 == 0);

    println!("Call 1 (odd): {:?}", filtered.get());
    println!("Call 2 (even): {:?}", filtered.get());
    println!("Call 3 (odd): {:?}", filtered.get());
    println!("Call 4 (even): {:?}\n", filtered.get());

    // ======================================================================
    // Example 7: BoxSupplier zip
    // ======================================================================
    println!("Example 7: BoxSupplier zip");
    println!("{}", "-".repeat(50));

    let first = BoxSupplier::new(|| 42);
    let second = BoxSupplier::new(|| "hello");
    let mut zipped = first.zip(second);

    println!("Zipped result: {:?}\n", zipped.get());

    // ======================================================================
    // Example 8: BoxSupplier memoize
    // ======================================================================
    println!("Example 8: BoxSupplier memoize");
    println!("{}", "-".repeat(50));

    let mut call_count = 0;
    let mut memoized = BoxSupplier::new(move || {
        call_count += 1;
        println!("  Underlying function called (count: {})", call_count);
        42
    })
    .memoize();

    println!("First call:");
    println!("Result: {}", memoized.get());
    println!("Second call:");
    println!("Result: {}", memoized.get());
    println!("Third call:");
    println!("Result: {}\n", memoized.get());

    // ======================================================================
    // Example 9: BoxSupplier lazy
    // ======================================================================
    println!("Example 9: BoxSupplier lazy");
    println!("{}", "-".repeat(50));

    println!("Creating lazy supplier...");
    let mut lazy = BoxSupplier::lazy(|| {
        println!("  Factory function called!");
        BoxSupplier::new(|| {
            println!("  Supplier function called!");
            42
        })
    });

    println!("First get() call:");
    println!("Result: {}", lazy.get());
    println!("Second get() call:");
    println!("Result: {}\n", lazy.get());

    // ======================================================================
    // Example 10: BoxSupplier alternate
    // ======================================================================
    println!("Example 10: BoxSupplier alternate");
    println!("{}", "-".repeat(50));

    let supplier1 = BoxSupplier::new(|| 1);
    let supplier2 = BoxSupplier::new(|| 2);
    let mut alternating = BoxSupplier::alternate(supplier1, supplier2);

    println!("Call 1: {}", alternating.get());
    println!("Call 2: {}", alternating.get());
    println!("Call 3: {}", alternating.get());
    println!("Call 4: {}\n", alternating.get());

    // ======================================================================
    // Example 11: BoxSupplier or_else
    // ======================================================================
    println!("Example 11: BoxSupplier or_else");
    println!("{}", "-".repeat(50));

    let primary = BoxSupplier::new(|| None);
    let mut supplier = primary.or_else(|| Some(42));
    println!("Primary returns None, fallback: {:?}", supplier.get());

    let primary2 = BoxSupplier::new(|| Some(10));
    let mut supplier2 = primary2.or_else(|| Some(999));
    println!("Primary returns Some(10): {:?}\n", supplier2.get());

    // ======================================================================
    // Example 12: Closure Extension Methods
    // ======================================================================
    println!("Example 12: Closure Extension Methods");
    println!("{}", "-".repeat(50));

    let mapped = (|| 10).map(|x| x * 2);
    let mut result = mapped;
    println!("Closure map: {}", result.get());

    let chained = (|| 5).map(|x| x * 2).map(|x| x + 5);
    let mut result = chained;
    println!("Closure chain: {}\n", result.get());

    // ======================================================================
    // Example 13: ArcSupplier - Thread-safe Sharing
    // ======================================================================
    println!("Example 13: ArcSupplier - Thread-safe Sharing");
    println!("{}", "-".repeat(50));

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    let supplier = ArcSupplier::new(move || {
        let mut c = counter_clone.lock().unwrap();
        *c += 1;
        *c
    });

    // Clone for another thread
    let supplier_clone = supplier.clone();
    let handle = thread::spawn(move || {
        let mut s = supplier_clone;
        let value = s.get();
        println!("Thread result: {}", value);
        value
    });

    // Use in main thread
    let mut s = supplier;
    println!("Main thread result: {}", s.get());

    let thread_result = handle.join().unwrap();
    println!("Total counter value: {}", *counter.lock().unwrap());
    println!("Thread returned: {}\n", thread_result);

    // ======================================================================
    // Example 14: ArcSupplier with map
    // ======================================================================
    println!("Example 14: ArcSupplier with map");
    println!("{}", "-".repeat(50));

    let source = ArcSupplier::new(|| 10);
    let mapped = source.map(|x| x * 2);

    // source is still usable because map borrows &self
    let mut m = mapped;
    println!("Mapped result: {}\n", m.get());

    // ======================================================================
    // Example 15: ArcSupplier memoize
    // ======================================================================
    println!("Example 15: ArcSupplier memoize");
    println!("{}", "-".repeat(50));

    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = Arc::clone(&call_count);
    let source = ArcSupplier::new(move || {
        let mut c = call_count_clone.lock().unwrap();
        *c += 1;
        println!("  ArcSupplier called (count: {})", *c);
        42
    });
    let memoized = source.memoize();

    let mut s = memoized;
    println!("First call:");
    println!("Result: {}", s.get());
    println!("Second call:");
    println!("Result: {}", s.get());
    println!("Total calls: {}\n", *call_count.lock().unwrap());

    // ======================================================================
    // Example 16: ArcSupplier zip
    // ======================================================================
    println!("Example 16: ArcSupplier zip");
    println!("{}", "-".repeat(50));

    let first = ArcSupplier::new(|| 42);
    let second = ArcSupplier::new(|| "hello");
    let zipped = first.zip(&second);

    // first and second are still usable
    let mut z = zipped;
    println!("Zipped result: {:?}\n", z.get());

    // ======================================================================
    // Example 17: RcSupplier - Single-threaded Sharing
    // ======================================================================
    println!("Example 17: RcSupplier - Single-threaded Sharing");
    println!("{}", "-".repeat(50));

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let supplier = RcSupplier::new(move || {
        let mut c = counter_clone.borrow_mut();
        *c += 1;
        *c
    });

    let mut s1 = supplier.clone();
    println!("Clone 1 result: {}", s1.get());

    let mut s2 = supplier.clone();
    println!("Clone 2 result: {}", s2.get());

    println!("Total counter value: {}\n", *counter.borrow());

    // ======================================================================
    // Example 18: RcSupplier with map
    // ======================================================================
    println!("Example 18: RcSupplier with map");
    println!("{}", "-".repeat(50));

    let source = RcSupplier::new(|| 10);
    let mapped = source.map(|x| x * 2);

    // source is still usable
    let mut m = mapped;
    println!("Mapped result: {}\n", m.get());

    // ======================================================================
    // Example 19: RcSupplier memoize
    // ======================================================================
    println!("Example 19: RcSupplier memoize");
    println!("{}", "-".repeat(50));

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = Rc::clone(&call_count);
    let source = RcSupplier::new(move || {
        let mut c = call_count_clone.borrow_mut();
        *c += 1;
        println!("  RcSupplier called (count: {})", *c);
        42
    });
    let memoized = source.memoize();

    let mut s = memoized;
    println!("First call:");
    println!("Result: {}", s.get());
    println!("Second call:");
    println!("Result: {}", s.get());
    println!("Total calls: {}\n", *call_count.borrow());

    // ======================================================================
    // Example 20: Using Supplier Trait
    // ======================================================================
    println!("Example 20: Using Supplier Trait");
    println!("{}", "-".repeat(50));

    fn use_any_supplier<S: Supplier<i32>>(supplier: &mut S) -> i32 {
        supplier.get()
    }

    let mut box_sup = BoxSupplier::new(|| 42);
    println!("BoxSupplier: {}", use_any_supplier(&mut box_sup));

    let arc_sup = ArcSupplier::new(|| 43);
    let mut s = arc_sup;
    println!("ArcSupplier: {}", use_any_supplier(&mut s));

    let rc_sup = RcSupplier::new(|| 44);
    let mut s = rc_sup;
    println!("RcSupplier: {}", use_any_supplier(&mut s));

    let mut closure = || 45;
    println!("Closure: {}", use_any_supplier(&mut closure));

    println!("\n=== Demo Complete ===");
}
