/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use prism3_function::comparator::{
    ArcComparator, BoxComparator, Comparator, FnComparatorOps, RcComparator,
};
use std::cmp::Ordering;

#[cfg(test)]
mod box_comparator_tests {
    use super::*;

    #[test]
    fn test_new_and_compare() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cmp.compare(&3, &5), Ordering::Less);
        assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_reversed() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
        assert_eq!(rev.compare(&3, &5), Ordering::Greater);
        assert_eq!(rev.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_then_comparing() {
        let cmp1 = BoxComparator::new(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));
        let cmp2 = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let chained = cmp1.then_comparing(cmp2);
        assert_eq!(chained.compare(&4, &2), Ordering::Greater);
        assert_eq!(chained.compare(&3, &1), Ordering::Greater);
        assert_eq!(chained.compare(&2, &4), Ordering::Less);
    }

    #[test]
    fn test_then_comparing_with_equal() {
        let cmp1 = BoxComparator::new(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));
        let cmp2 = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let chained = cmp1.then_comparing(cmp2);
        // Both even, so second comparator decides
        assert_eq!(chained.compare(&4, &2), Ordering::Greater);
    }

    #[test]
    fn test_comparing() {
        #[derive(Debug)]
        struct Person {
            #[allow(dead_code)]
            name: String,
            age: i32,
        }

        let by_age = BoxComparator::comparing(|p: &Person| &p.age);
        let p1 = Person {
            name: "Alice".to_string(),
            age: 30,
        };
        let p2 = Person {
            name: "Bob".to_string(),
            age: 25,
        };
        assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    }

    #[test]
    fn test_into_fn() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let func = cmp.into_fn();
        assert_eq!(func(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_box() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let boxed = cmp.into_box();
        assert_eq!(boxed.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_rc() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rc = cmp.into_rc();
        assert_eq!(rc.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_with_strings() {
        let cmp = BoxComparator::new(|a: &String, b: &String| a.cmp(b));
        assert_eq!(
            cmp.compare(&"hello".to_string(), &"world".to_string()),
            Ordering::Less
        );
    }
}

#[cfg(test)]
mod arc_comparator_tests {
    use super::*;

    #[test]
    fn test_new_and_compare() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cmp.compare(&3, &5), Ordering::Less);
        assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_clone() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let cloned = cmp.clone();
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_reversed() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
        // Original still works
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_then_comparing() {
        let cmp1 = ArcComparator::new(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));
        let cmp2 = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let chained = cmp1.then_comparing(&cmp2);
        assert_eq!(chained.compare(&4, &2), Ordering::Greater);
        // Originals still work
        assert_eq!(cmp1.compare(&4, &2), Ordering::Equal);
        assert_eq!(cmp2.compare(&4, &2), Ordering::Greater);
    }

    #[test]
    fn test_comparing() {
        #[derive(Debug)]
        struct Person {
            #[allow(dead_code)]
            name: String,
            age: i32,
        }

        let by_age = ArcComparator::comparing(|p: &Person| &p.age);
        let p1 = Person {
            name: "Alice".to_string(),
            age: 30,
        };
        let p2 = Person {
            name: "Bob".to_string(),
            age: 25,
        };
        assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    }

    #[test]
    fn test_into_fn() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let func = cmp.into_fn();
        assert_eq!(func(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_box() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let boxed = cmp.into_box();
        assert_eq!(boxed.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_rc() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rc = cmp.into_rc();
        assert_eq!(rc.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_arc() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let arc = cmp.into_arc();
        assert_eq!(arc.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_thread_safety() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cmp_clone = cmp.clone();
                std::thread::spawn(move || {
                    assert_eq!(cmp_clone.compare(&(i + 1), &i), Ordering::Greater);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod rc_comparator_tests {
    use super::*;

    #[test]
    fn test_new_and_compare() {
        let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cmp.compare(&3, &5), Ordering::Less);
        assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_clone() {
        let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let cloned = cmp.clone();
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_reversed() {
        let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
        // Original still works
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_then_comparing() {
        let cmp1 = RcComparator::new(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));
        let cmp2 = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let chained = cmp1.then_comparing(&cmp2);
        assert_eq!(chained.compare(&4, &2), Ordering::Greater);
        // Originals still work
        assert_eq!(cmp1.compare(&4, &2), Ordering::Equal);
        assert_eq!(cmp2.compare(&4, &2), Ordering::Greater);
    }

    #[test]
    fn test_comparing() {
        #[derive(Debug)]
        struct Person {
            #[allow(dead_code)]
            name: String,
            age: i32,
        }

        let by_age = RcComparator::comparing(|p: &Person| &p.age);
        let p1 = Person {
            name: "Alice".to_string(),
            age: 30,
        };
        let p2 = Person {
            name: "Bob".to_string(),
            age: 25,
        };
        assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    }

    #[test]
    fn test_into_fn() {
        let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let func = cmp.into_fn();
        assert_eq!(func(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_box() {
        let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let boxed = cmp.into_box();
        assert_eq!(boxed.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_rc() {
        let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rc = cmp.into_rc();
        assert_eq!(rc.compare(&5, &3), Ordering::Greater);
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;

    #[test]
    fn test_closure_as_comparator() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cmp.compare(&3, &5), Ordering::Less);
        assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_closure_into_box() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        let boxed = cmp.into_box();
        assert_eq!(boxed.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_closure_into_rc() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        let rc = cmp.into_rc();
        assert_eq!(rc.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_closure_into_arc() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        let arc = cmp.into_arc();
        assert_eq!(arc.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_closure_into_fn() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        let func = cmp.into_fn();
        assert_eq!(func(&5, &3), Ordering::Greater);
    }
}

#[cfg(test)]
mod fn_ops_tests {
    use super::*;

    #[test]
    fn test_reversed() {
        let rev = (|a: &i32, b: &i32| a.cmp(b)).reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
    }

    #[test]
    fn test_then_comparing() {
        let cmp = (|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)))
            .then_comparing(BoxComparator::new(|a: &i32, b: &i32| a.cmp(b)));
        assert_eq!(cmp.compare(&4, &2), Ordering::Greater);
    }

    #[test]
    fn test_chained_operations() {
        let cmp = (|a: &i32, b: &i32| a.cmp(b))
            .reversed()
            .then_comparing(BoxComparator::new(|a: &i32, b: &i32| b.cmp(a)));
        assert_eq!(cmp.compare(&5, &3), Ordering::Less);
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_box_to_rc() {
        let box_cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rc_cmp = box_cmp.into_rc();
        assert_eq!(rc_cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_arc_to_box() {
        let arc_cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let box_cmp = arc_cmp.into_box();
        assert_eq!(box_cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_arc_to_rc() {
        let arc_cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rc_cmp = arc_cmp.into_rc();
        assert_eq!(rc_cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_rc_to_box() {
        let rc_cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let box_cmp = rc_cmp.into_box();
        assert_eq!(box_cmp.compare(&5, &3), Ordering::Greater);
    }
}

#[cfg(test)]
mod generic_tests {
    use super::*;

    fn sort_with_comparator<C: Comparator<i32>>(cmp: &C, mut vec: Vec<i32>) -> Vec<i32> {
        vec.sort_by(|a, b| cmp.compare(a, b));
        vec
    }

    #[test]
    fn test_with_box_comparator() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let sorted = sort_with_comparator(&cmp, vec![3, 1, 4, 1, 5]);
        assert_eq!(sorted, vec![1, 1, 3, 4, 5]);
    }

    #[test]
    fn test_with_arc_comparator() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let sorted = sort_with_comparator(&cmp, vec![3, 1, 4, 1, 5]);
        assert_eq!(sorted, vec![1, 1, 3, 4, 5]);
    }

    #[test]
    fn test_with_rc_comparator() {
        let cmp = RcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let sorted = sort_with_comparator(&cmp, vec![3, 1, 4, 1, 5]);
        assert_eq!(sorted, vec![1, 1, 3, 4, 5]);
    }

    #[test]
    fn test_with_closure() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        let sorted = sort_with_comparator(&cmp, vec![3, 1, 4, 1, 5]);
        assert_eq!(sorted, vec![1, 1, 3, 4, 5]);
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_with_empty_values() {
        let cmp = BoxComparator::new(|a: &String, b: &String| a.cmp(b));
        assert_eq!(
            cmp.compare(&String::new(), &"hello".to_string()),
            Ordering::Less
        );
    }

    #[test]
    fn test_with_negative_numbers() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&-5, &-3), Ordering::Less);
        assert_eq!(cmp.compare(&-3, &-5), Ordering::Greater);
    }

    #[test]
    fn test_multiple_reversals() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev1 = cmp.reversed();
        let rev2 = rev1.reversed();
        // Double reversal should be same as original
        assert_eq!(rev2.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_long_chain() {
        let cmp1 = BoxComparator::new(|a: &i32, b: &i32| (a / 10).cmp(&(b / 10)));
        let cmp2 = BoxComparator::new(|a: &i32, b: &i32| (a % 10).cmp(&(b % 10)));
        let chained = cmp1.then_comparing(cmp2);
        assert_eq!(chained.compare(&15, &12), Ordering::Greater);
        assert_eq!(chained.compare(&12, &15), Ordering::Less);
    }
}
