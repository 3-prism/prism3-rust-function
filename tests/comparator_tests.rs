/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use prism3_function::comparator::{ArcComparator, BoxComparator, Comparator, FnComparatorOps};
use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure_as_comparator() {
        let cmp = |a: &i32, b: &i32| a.cmp(b);
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cmp.compare(&3, &5), Ordering::Less);
        assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_box_comparator_basic() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_box_comparator_reversed() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
    }

    #[test]
    fn test_arc_comparator_clone() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let cloned = cmp.clone();
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_arc_comparator_reversed() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_then_comparing() {
        let cmp1 = BoxComparator::new(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));
        let cmp2 = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let chained = cmp1.then_comparing(cmp2);
        assert_eq!(chained.compare(&4, &2), Ordering::Greater);
        assert_eq!(chained.compare(&3, &1), Ordering::Greater);
    }

    #[test]
    fn test_fn_ops_reversed() {
        let rev = (|a: &i32, b: &i32| a.cmp(b)).reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
    }
}
