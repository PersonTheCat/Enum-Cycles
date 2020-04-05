#[macro_use]
extern crate enum_cycles_derive;

use enum_cycles::EnumState;
use Numbers::*;
use Letters::*;
use Outer::*;

#[default(One)]
#[derive(Debug, PartialEq, Clone, EnumState)]
enum Numbers {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine
}

#[derive(Debug, PartialEq, Clone, EnumState)]
enum Letters {
    A,
    B,
    C
}

#[auto]
#[derive(Debug, PartialEq, Clone, EnumState)]
enum Outer {
    #[last]
    NumLast(Numbers),
    #[default(B)]
    LetManual(Letters),
    NumAuto(Numbers),
    LetAuto(Letters),
}

#[test]
fn test_skip() {
    let count = 25;
    let interval = 5;
    let mut multiple = 1;

    let mut vals = Vec::with_capacity(count);
    let mut n = Numbers::Zero;

    while vals.len() < count {
        for _ in 0..interval {
            n.skip(multiple);
            vals.push(n.index())
        }
        multiple += 1;
    }

    let expected = vec![
        1, 2, 3, 4, 5, // +1
        7, 9, 0, 2, 4, // +2
        7, 9, 0, 3, 6, // +3
        9, 0, 4, 8, 9, // +4
        0, 5, 9, 0, 5, // +5
    ];

    assert_eq!(vals, expected);
}

#[test]
fn test_skip_range() {
    let range = 1000;
    let mut i = 0;
    let mut n = Numbers::Zero;

    for _ in 0..range {
        n.skip(i);
        i += 1;
    }
}

#[test]
fn test_skip_backward() {
    let count = 25;
    let interval = 5;
    let mut multiple = 1;

    let mut vals = Vec::with_capacity(count);
    let mut n = Numbers::Nine;

    while vals.len() < count {
        for _ in 0..interval {
            n.skip_backward(multiple);
            vals.push(n.index());
        }
        multiple += 1;
    }

    let expected = vec![
        8, 7, 6, 5, 4, // -1
        2, 0, 9, 7, 5, // -2
        2, 0, 9, 6, 3, // -3
        0, 9, 5, 1, 0, // -4
        9, 4, 0, 9, 4, // -5
    ];

    assert_eq!(vals, expected);
}

#[test]
fn test_skip_backward_range() {
    let range = 1000;
    let mut i = 0;
    let mut n = Numbers::Nine;

    for _ in 0..range {
        n.skip_backward(i);
        i += 1;
    }
}

#[test]
fn test_properties() {
    let names = [
        "Zero", "One", "Two", "Three", "Four",
        "Five", "Six", "Seven", "Eight", "Nine"
    ];
    let values = [
        Zero, One, Two, Three, Four,
        Five, Six, Seven, Eight, Nine
    ];

    assert_eq!(Numbers::names(), names);
    assert_eq!(Numbers::values(), values);
    assert_eq!(Numbers::first(), Zero);
    assert_eq!(Numbers::last(), Nine);
    assert_eq!(Numbers::size(), 10);
}

#[test]
fn test_defaults() {
    let values = [
        NumLast(Nine), // #[last] overrides #[auto]
        LetManual(B),  // #[default(...)] overrides #[auto]
        NumAuto(One),  // Default specified, used by #[auto]
        LetAuto(A),    // No default => #[auto] uses first
    ];

    assert_eq!(Outer::values(), values);
}