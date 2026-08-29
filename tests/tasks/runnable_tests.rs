// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for runnable task types.

use std::cell::Cell;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_function::ArcRunnable;
use qubit_function::BoxRunnable;
use qubit_function::Callable;
use qubit_function::RcRunnable;
use qubit_function::Runnable;
use qubit_function::SupplierOnce;

#[derive(Clone)]
struct ClonedRunnable {
    flag: Rc<Cell<bool>>,
}

impl Runnable<io::Error> for ClonedRunnable {
    fn run(&mut self) -> Result<(), io::Error> {
        self.flag.set(true);
        Ok(())
    }
}

#[test]
fn test_runnable_closure_run_returns_success() {
    let flag = Rc::new(Cell::new(false));
    let captured = Rc::clone(&flag);
    let mut task = move || {
        captured.set(true);
        Ok::<(), io::Error>(())
    };

    task.run().expect("runnable closure should succeed");
    assert!(flag.get());
}

#[test]
fn test_runnable_closure_run_returns_error() {
    let mut task = || Err::<(), _>(io::Error::other("failed"));

    let error = task.run().expect_err("runnable closure should fail");
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "failed");
}

#[derive(Clone)]
struct SharedCounterRunnable {
    count: Rc<Cell<u32>>,
}

impl Runnable<io::Error> for SharedCounterRunnable {
    fn run(&mut self) -> Result<(), io::Error> {
        self.count.set(self.count.get() + 1);
        Ok(())
    }
}

#[derive(Clone)]
struct SharedAtomicRunnable {
    count: Arc<AtomicUsize>,
}

impl Runnable<io::Error> for SharedAtomicRunnable {
    fn run(&mut self) -> Result<(), io::Error> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_rc_runnable_from_supplier() {
    let count = Rc::new(Cell::new(0));
    let captured = Rc::clone(&count);
    let mut task = RcRunnable::from_supplier(move || {
        captured.set(captured.get() + 1);
        Ok::<(), io::Error>(())
    });

    task.run().expect("supplier runnable should succeed");
    task.run().expect("supplier runnable should succeed again");
    assert_eq!(count.get(), 2);
}

#[test]
fn test_arc_runnable_from_supplier() {
    let count = Arc::new(AtomicUsize::new(0));
    let captured = Arc::clone(&count);
    let mut task = ArcRunnable::from_supplier(move || {
        captured.fetch_add(1, Ordering::SeqCst);
        Ok::<(), io::Error>(())
    });

    task.run().expect("supplier runnable should succeed");
    task.run().expect("supplier runnable should succeed again");
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_box_runnable_new_and_run() {
    let flag = Arc::new(AtomicBool::new(false));
    let captured = Arc::clone(&flag);
    let mut task = BoxRunnable::new(move || {
        captured.store(true, Ordering::SeqCst);
        Ok::<(), io::Error>(())
    });

    task.run().expect("box runnable should succeed");
    assert!(flag.load(Ordering::SeqCst));
}

#[test]
fn test_box_runnable_name_management() {
    let mut task = BoxRunnable::<io::Error>::new_with_name("cleanup", || Ok(()));

    assert_eq!(task.name(), Some("cleanup"));
    assert_eq!(task.to_string(), "BoxRunnable(cleanup)");
    assert!(format!("{task:?}").contains("cleanup"));

    task.set_name("renamed");
    assert_eq!(task.name(), Some("renamed"));

    task.clear_name();
    assert_eq!(task.name(), None);
    assert_eq!(task.to_string(), "BoxRunnable");
}

#[test]
fn test_shared_runnable_name_display_and_debug() {
    let rc = RcRunnable::<io::Error>::new_with_name("cleanup", || Ok(()));
    let arc = ArcRunnable::<io::Error>::new_with_name("cleanup", || Ok(()));

    assert_eq!(rc.to_string(), "RcRunnable(cleanup)");
    assert!(format!("{rc:?}").contains("cleanup"));
    assert_eq!(arc.to_string(), "ArcRunnable(cleanup)");
    assert!(format!("{arc:?}").contains("cleanup"));
}

#[test]
fn test_box_runnable_set_name_handles_empty_and_same_name() {
    let mut task = BoxRunnable::<io::Error>::new(|| Ok(()));

    assert_eq!(task.name(), None);
    task.set_name("cleanup");
    assert_eq!(task.name(), Some("cleanup"));
    task.set_name("cleanup");
    assert_eq!(task.name(), Some("cleanup"));
}

#[test]
fn test_box_runnable_from_supplier() {
    let supplier = || Ok::<(), io::Error>(());

    let mut task = BoxRunnable::from_supplier(supplier);

    task.run().expect("supplier-backed runnable should succeed");
}

#[test]
fn test_box_runnable_implements_supplier_once() {
    let task = BoxRunnable::new(|| Ok::<(), io::Error>(()));

    let result = SupplierOnce::get(task);

    result.expect("supplier runnable should succeed");
}

#[test]
fn test_box_runnable_and_then_runs_next_on_success() {
    let events = Arc::new(AtomicUsize::new(0));
    let first_events = Arc::clone(&events);
    let second_events = Arc::clone(&events);
    let first = BoxRunnable::new(move || {
        first_events.store(1, Ordering::SeqCst);
        Ok::<(), io::Error>(())
    });
    let second = move || {
        second_events.store(2, Ordering::SeqCst);
        Ok::<(), io::Error>(())
    };

    let mut chained = first.and_then(second);

    chained.run().expect("chained runnable should succeed");
    assert_eq!(events.load(Ordering::SeqCst), 2);
}

#[test]
fn test_box_runnable_and_then_skips_next_on_error() {
    let events = Arc::new(AtomicUsize::new(0));
    let second_events = Arc::clone(&events);
    let first = BoxRunnable::new(|| Err::<(), _>(io::Error::other("stop")));
    let second = move || {
        second_events.store(2, Ordering::SeqCst);
        Ok::<(), io::Error>(())
    };

    let mut chained = first.and_then(second);

    assert_eq!(
        chained
            .run()
            .expect_err("chained runnable should preserve error")
            .to_string(),
        "stop",
    );
    assert_eq!(events.load(Ordering::SeqCst), 0);
}

#[test]
fn test_box_runnable_then_callable_runs_callable_on_success() {
    let task = BoxRunnable::new_with_name("prepare", || Ok::<(), io::Error>(()));
    let callable = || Ok::<i32, io::Error>(42);

    let mut chained = task.then_callable(callable);

    assert_eq!(chained.name(), None);
    assert_eq!(chained.call().expect("callable should succeed"), 42);
}

#[test]
fn test_box_runnable_then_callable_skips_callable_on_error() {
    let callable_ran = Arc::new(AtomicBool::new(false));
    let callable_ran_capture = Arc::clone(&callable_ran);
    let task = BoxRunnable::<io::Error>::new(|| Err(io::Error::other("prepare failed")));
    let callable = move || {
        callable_ran_capture.store(true, Ordering::SeqCst);
        Ok::<i32, io::Error>(42)
    };

    let mut chained = task.then_callable(callable);
    let error = chained
        .call()
        .expect_err("then_callable should preserve runnable error");

    assert_eq!(error.to_string(), "prepare failed");
    assert!(!callable_ran.load(Ordering::SeqCst));
}

#[test]
fn test_box_runnable_then_callable_covers_both_results_for_one_callable_type() {
    let callable: fn() -> Result<i32, io::Error> = || Ok(42);
    let mut success = BoxRunnable::new(|| Ok::<(), io::Error>(())).then_callable(callable);
    let mut failure =
        BoxRunnable::new(|| Err::<(), io::Error>(io::Error::other("prepare failed"))).then_callable(callable);

    assert_eq!(success.call().expect("callable should succeed"), 42);
    assert_eq!(
        failure
            .call()
            .expect_err("then_callable should preserve runnable error")
            .to_string(),
        "prepare failed"
    );
}
