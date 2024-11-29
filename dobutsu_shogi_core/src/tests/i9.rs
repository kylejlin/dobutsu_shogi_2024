use super::*;

#[test]
fn positive1_i9_converts_to_i16_correctly() {
    let actual = i9_to_i16(1);
    let expected: i16 = 1;
    assert_eq!(expected, actual);
}

#[test]
fn zero_i9_converts_to_i16_correctly() {
    let actual = i9_to_i16(0);
    let expected: i16 = 0;
    assert_eq!(expected, actual);
}

#[test]
fn negative1_i9_converts_to_i16_correctly() {
    let actual = i9_to_i16(0b1_1111_1111);
    let expected: i16 = -1;
    assert_eq!(expected, actual);
}

#[test]
fn negative201_i9_converts_to_i16_correctly() {
    let actual = i9_to_i16(NEGATIVE_201_I9);
    let expected: i16 = -201;
    assert_eq!(expected, actual);
}

#[test]
fn positive1_i16_converts_to_i9_correctly() {
    let actual = i16_to_i9(1i16);
    let expected: u64 = 1;
    assert_eq!(expected, actual);
}

#[test]
fn zero_i16_converts_to_i9_correctly() {
    let actual = i16_to_i9(0i16);
    let expected: u64 = 0;
    assert_eq!(expected, actual);
}

#[test]
fn negative1_i16_converts_to_i9_correctly() {
    let actual = i16_to_i9(-1i16);
    let expected: u64 = 0b1_1111_1111;
    assert_eq!(expected, actual);
}

#[test]
fn negative201_i16_converts_to_i9_correctly() {
    let actual = i16_to_i9(-201i16);
    let expected: u64 = NEGATIVE_201_I9;
    assert_eq!(expected, actual);
}

/// `-200`` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const NEGATIVE_201_I9: u64 = 0b1_0011_0111;
