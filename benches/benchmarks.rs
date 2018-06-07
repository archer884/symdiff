#![feature(test)]

extern crate symdiff;
extern crate test;

use test::Bencher;
use symdiff::SymmetricDifference;

#[bench]
fn internal(b: &mut Bencher) {
    let left = build_left();
    let right = build_right();

    let left = &left;
    let right = &right;

    b.iter(|| {
        left.diff_internal(
            right,
            |x| {
                test::black_box(x);
            },
            |x| {
                test::black_box(x);
            },
        );
    });
}

#[bench]
fn internal_alt(b: &mut Bencher) {
    let left = build_left();
    let right = build_right();

    let left = &left;
    let right = &right;

    b.iter(|| {
        left.diff_internal_alt(right, |x| {
            test::black_box(x);
        });
    });
}

#[bench]
fn external(b: &mut Bencher) {
    let left = build_left();
    let right = build_right();

    let left = &left;
    let right = &right;

    b.iter(|| {
        for item in left.diff(right) {
            test::black_box(item);
        }
    });
}

#[bench]
fn stdlib(b: &mut Bencher) {
    use std::collections::HashSet;

    let left = build_left();
    let right = build_right();

    b.iter(|| {
        let left: HashSet<_> = left.iter().collect();
        let right: HashSet<_> = right.iter().collect();

        for &item in left.symmetric_difference(&right) {
            test::black_box(item);
        }
    });
}

#[bench]
fn stdlib_no_overhead(b: &mut Bencher) {
    use std::collections::HashSet;

    let left: HashSet<_> = build_left().into_iter().collect();
    let right: HashSet<_> = build_right().into_iter().collect();

    let left = &left;
    let right = &right;

    b.iter(|| {
        for &item in left.symmetric_difference(right) {
            test::black_box(item);
        }
    });
}

fn build_left() -> Vec<i32> {
    (0..1000).filter(|x| x % 13 != 0).collect()
}

fn build_right() -> Vec<i32> {
    (1..1000).filter(|x| x % 23 != 0).collect()
}
