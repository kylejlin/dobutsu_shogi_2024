use super::*;

#[test]
fn positive1_i9_converts_to_i16_correctly() {
    let actual = i16::from_zero_padded_i9(1);
    let expected: i16 = 1;
    assert_eq!(expected, actual);
}

#[test]
fn zero_i9_converts_to_i16_correctly() {
    let actual = i16::from_zero_padded_i9(0);
    let expected: i16 = 0;
    assert_eq!(expected, actual);
}

#[test]
fn negative1_i9_converts_to_i16_correctly() {
    let actual = i16::from_zero_padded_i9(0b1_1111_1111);
    let expected: i16 = -1;
    assert_eq!(expected, actual);
}

#[test]
fn negative201_i9_converts_to_i16_correctly() {
    let actual = i16::from_zero_padded_i9(NEGATIVE_201_I9);
    let expected: i16 = -201;
    assert_eq!(expected, actual);
}

#[test]
fn positive1_i16_converts_to_i9_correctly() {
    let actual = 1i16.into_zero_padded_i9_unchecked();
    let expected: u64 = 1;
    assert_eq!(expected, actual);
}

#[test]
fn zero_i16_converts_to_i9_correctly() {
    let actual = 0i16.into_zero_padded_i9_unchecked();
    let expected: u64 = 0;
    assert_eq!(expected, actual);
}

#[test]
fn negative1_i16_converts_to_i9_correctly() {
    let actual = (-1i16).into_zero_padded_i9_unchecked();
    let expected: u64 = 0b1_1111_1111;
    assert_eq!(expected, actual);
}

#[test]
fn negative201_i16_converts_to_i9_correctly() {
    let actual = (-201i16).into_zero_padded_i9_unchecked();
    let expected: u64 = NEGATIVE_201_I9;
    assert_eq!(expected, actual);
}
