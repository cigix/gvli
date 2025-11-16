//! # Generalized variable-length integers
//!
//! This crate implements the "generalized variable-length integers" described
//! in [RFC 3492 section 3.3](https://datatracker.ietf.org/doc/html/rfc3492#section-3.3).
//! This implementation is generic, so that it may be used outside of just RFC
//! 3492 implementations.
//!
//! This crate implements only little-endian encoding and decoding, as they are
//! the only ones described in the RFC and associated litterature.
//!
//! ## Example
//!
//! ```rust
//! let parameters = gvli::Parameters::from_parts(&gvli::BASE_OCTAL, &[2, 3, 5]).unwrap();
//!
//! let encoded = gvli::encode(&[145, 62], &parameters);
//! assert_eq!(encoded, "734251");
//!
//! let decoded = gvli::decode(&encoded, &parameters).unwrap();
//! assert_eq!(decoded, [145, 62]);
//! ```
//!
//! ## Usage
//!
//! ### Parameters
//!
//! GVLI encoding and decoding requires having:
//! * a _base_: a finite sequence of digits; a mapping of `char`s to numbers;
//! * a _thresholds sequence_: an infinite sequence of numbers which allows for
//!   discriminating terminating digits from non-terminating ones.
//!
//! For one application, both of these settings are going to be fixed, so they
//! are grouped into one struct: [`Parameters`].
//!
//! A convenience constant is provided for the specific case of Punycode:
//! [`PUNYCODE`].
//!
//! ### Encoding
//!
//! Encoding takes a sequence of numbers and turns them into a sequence of
//! `char`s, the _digits_. Four encoding functions are provided:
//! * [`encode_one()`] encodes a single number, providing a new String,
//! * [`encode()`] encodes a sequence of numbers, providing a new String,
//! * their variants [`encode_one_noalloc()`] and [`encode_noalloc()`] which
//!   extend and existing String.
//!
//! Encoding never fails: all possible violations are checked by
//! [`Parameters::from_parts()`].
//!
//! ### Decoding
//!
//! Decoding takes a sequence of `char`s, the _digits_, and turns them into a
//! sequence of numbers. Because it is impossible to know in advance how many
//! numbers are encoded, and if they are encoded correctly, only one function is
//! provided: [`decode()`].
//!
//! Decoding may fail:
//! * if a digit that is not part of the base is encountered,
//! * if the sequence of `char`s ends on a non-terminating digit.

use std::error;
use std::fmt;

/// Error type returned by the GVLI APIs.
#[derive(Debug, Clone, PartialEq)]
pub enum GVLIError {
    /// The provided `base` array is too short, it should have length >= 2.
    BaseTooShort(usize),
    /// The provided `thresholds` array is too short, it should have length >=
    /// 1.
    ThresholdsTooShort(usize),
    /// The provided `thresholds` contains a threshold larger than the base's
    /// length.
    ThresholdTooLarge(usize, usize),
    /// A digit is found during decoding which is not part of the provided
    /// `base` array.
    DigitNotInBase(char),
    /// A digit sequence ended without a terminator digit.
    EndOfDigits,
}
impl error::Error for GVLIError {}
impl fmt::Display for GVLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GVLIError::BaseTooShort(b) => {
                write!(f, "base is too short: got length {}, should be >= 2", b)
            }
            GVLIError::ThresholdsTooShort(t) => {
                write!(f,
                    "thresholds is too short: got length {}, should be >= 1", t)
            }
            GVLIError::ThresholdTooLarge(b, t) => {
                write!(f,
                    "threshold {} is too large for base of length {}", t, b)
            }
            GVLIError::DigitNotInBase(d) => {
                write!(f, "digit {} is not present in the base", d)
            }
            GVLIError::EndOfDigits => {
                write!(f, "digit sequence ended without a terminator digit")
            }
        }
    }
}


/// The parameters of the GVLI encoding/decoding process: the base and
/// thresholds.
///
/// The base is a finite sequence of `char`s to encode to and decode from.
///
/// The thresholds are an infinite sequence of numbers, represented by a finite
/// sequence whose last element is repeated forever.
#[derive(Debug, PartialEq)]
pub struct Parameters<const B: usize, const T: usize> {
    base: [char; B],
    thresholds: [usize; T],
    // invariants:
    // * 2 <= B
    // * 1 <= T
    // * thresholds[0..T] < B
}
impl<const B: usize, const T: usize> Parameters<B, T> {
    /// Build a new [`Parameters`].
    ///
    /// The last element of `thresholds` is repeated infinitely.
    ///
    /// # Errors
    /// * [`GVLIError::BaseTooShort`] if `B < 2`
    /// * [`GVLIError::ThresholdsTooShort`] if `T < 1`
    /// * [`GVLIError::ThresholdTooLarge`] if `thresholds` contains a value
    ///   `>= B`
    pub fn from_parts(base: &[char;B], thresholds: &[usize;T])
        -> Result<Self, GVLIError>
    {
        if B < 2 {
            return Err(GVLIError::BaseTooShort(B));
        }
        if T < 1 {
            return Err(GVLIError::ThresholdsTooShort(T));
        }
        if let Some(t) = thresholds.iter().filter(|t| B <= **t).next() {
            return Err(GVLIError::ThresholdTooLarge(B, *t))
        }
        Ok(Parameters {
            base: base.clone(),
            thresholds: thresholds.clone()
        })
    }

    /// Get the size of the base, i.e. `B`.
    pub fn base_len(&self) -> usize { B }

    /// Get the digit from the base at the given position.
    ///
    /// Returns `None` if `position >= B`.
    pub fn digit(&self, position: usize) -> Option<char>
    {
        self.base.get(position).copied()
    }

    /// Get the position of `digit` in the base, i.e. its numerical value.
    ///
    /// # Error
    /// * [`GVLIError::DigitNotInBase`] if `digit` not in `base`
    pub fn position(&self, digit: char) -> Result<usize, GVLIError>
    {
        self.base.iter()
            .position(|c| *c == digit)
            // Option<usize>
            .ok_or(GVLIError::DigitNotInBase(digit))
    }

    /// Get the applicable threshold at index `i`.
    ///
    /// Returns `thresholds[i]` if `i < thresholds.len()`, otherwise
    /// `thresholds.last()`.
    pub fn threshold(&self, i: usize) -> usize
    {
        // thresholds.last() is guaranteed to exist, as per the Parameters
        // invariants.
        *self.thresholds.get(i).or(self.thresholds.last()).unwrap()
    }
}

/// Convenience `base` array for a base defining its digits as `0` to `7`.
pub const BASE_OCTAL: [char; 8] = ['0', '1', '2', '3', '4', '5', '6', '7'];
/// Convenience `base` array for a base defining its digits as `0` to `9`.
pub const BASE_DECIMAL: [char; 10] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
/// Convenience `base` array for a base defining its digits as `0` to `9`, then
/// `A` to `F`.
pub const BASE_HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
/// Convenience `base` array for Punycode encoding: `a` to `z`, then `0` to `9`.
pub const BASE_PUNYCODE: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Convenience `thresholds` array for Punicode encoding: 1 twice, then 26
/// repeated infinitely.
pub const THRESHOLDS_PUNYCODE: [usize; 3] = [1, 1, 26];

/// Convenience [`Parameters`] for Punycode encoding. See [`BASE_PUNYCODE`] and
/// [`THRESHOLDS_PUNYCODE`].
pub const PUNYCODE: Parameters<36, 3> = Parameters {
    base: BASE_PUNYCODE,
    thresholds: THRESHOLDS_PUNYCODE
};

/// Encode a single number, appending to an existing String.
///
/// # Example
/// ```rust
/// let mut result = String::new();
/// gvli::encode_one_noalloc(745, &gvli::PUNYCODE, &mut result);
/// assert_eq!(result, "kva");
/// ```
pub fn encode_one_noalloc<const B: usize, const T: usize>
    (mut n: usize, params: &Parameters<B, T>, result: &mut String)
    -> ()
{
    let mut i = 0;
    loop {
        let t: usize = params.threshold(i);
        // t < B, as per the Parameters invariants
        if n < t {
            // n < t < B => the digit always exists
            result.push(params.digit(n).unwrap());
            return;
        }
        // we have t <= n
        // t <= n, t < B => subtractions can never underflow
        // t < B => 0 < B - t => modulo's RHS is never zero
        let digit = t + ((n - t) % (B - t));
        // (n - t) % (B - t) < B - t
        // => t + " < B => digit < B => the digit always exists
        result.push(params.digit(digit).unwrap());
        // 0 < B - t => division's RHS is never zero
        n = (n - t) / (B - t);
        i += 1;
    }
}

/// Encode a single number, creating a new String.
///
/// # Example
/// ```rust
/// let result: String = gvli::encode_one(745, &gvli::PUNYCODE);
/// assert_eq!(result, "kva");
/// ```
pub fn encode_one<const B: usize, const T: usize>
    (n: usize, params: &Parameters<B, T>)
    -> String
{
    let mut result = String::new();
    encode_one_noalloc(n, params, &mut result);
    result
}

/// Encode a sequence of numbers, appending to an existing String.
///
/// # Example
/// ```rust
/// let mut result = String::new();
/// gvli::encode_noalloc(&[745], &gvli::PUNYCODE, &mut result);
/// assert_eq!(result, "kva");
/// ```
pub fn encode_noalloc<const B: usize, const T: usize>
    (values: &[usize], params: &Parameters<B, T>, result: &mut String)
    -> ()
{
    for n in values.iter() {
        encode_one_noalloc(*n, params, result);
    }
}

/// Encode a sequence of numbers, creating a new String.
///
/// # Example
/// ```rust
/// let result: String = gvli::encode(&[745], &gvli::PUNYCODE);
/// assert_eq!(result, "kva");
/// ```
pub fn encode<const B: usize, const T: usize>
    (values: &[usize], params: &Parameters<B, T>)
    -> String
{
    let mut result = String::new();
    encode_noalloc(values, params, &mut result);
    result
}

/// Decode a sequence of GVLI.
///
/// # Example
/// ```rust
/// let result = gvli::decode("kva", &gvli::PUNYCODE);
/// assert_eq!(result, Ok(vec![745]));
/// ```
///
/// # Errors
/// * [`GVLIError::DigitNotInBase`] if a digit is not found in the base
/// * [`GVLIError::EndOfDigits`] if `digits` ends without a terminator digit
pub fn decode<const B: usize, const T: usize>
    (digits: &str, params: &Parameters<B, T>)
    -> Result<Vec<usize>, GVLIError>
{
    let mut result: Vec<usize> = Vec::new();
    let mut i = 0;
    let mut w = 1;
    let mut n = 0;
    for digit in digits.chars() {
        let d: usize = params.position(digit)?;
        let t: usize = params.threshold(i);
        // t < B, as per the Parameters invariants

        n += d * w;
        if d < t {
            // last digit
            result.push(n);
            i = 0;
            w = 1;
            n = 0;
        } else {
            w = w * (B - t); // t < B => subtraction can never underflow
            i += 1
        }
    }
    if i != 0 {
        Err(GVLIError::EndOfDigits)
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_errors() {
        assert_eq!(
            Parameters::from_parts(&[], &[]),
            Err(GVLIError::BaseTooShort(0)));
        assert_eq!(
            Parameters::from_parts(&['0'], &[]),
            Err(GVLIError::BaseTooShort(1)));
        assert_eq!(
            Parameters::from_parts(&['0', '1'], &[]),
            Err(GVLIError::ThresholdsTooShort(0)));
        assert_eq!(
            Parameters::from_parts(&['0', '1'], &[2]),
            Err(GVLIError::ThresholdTooLarge(2, 2)));
    }

    #[test]
    fn encode_one_rfc3492() {
        // Examples taken from RFC 3492
        let params = Parameters::from_parts(&BASE_OCTAL, &[2, 3, 5]).unwrap();
        assert_eq!(encode_one(145, &params), "734");
        assert_eq!(encode_one(62, &params), "251");
    }

    #[test]
    fn encode_one_wikipedia() {
        // Example taken from Wikipedia
        assert_eq!(
            encode_one(745, &PUNYCODE), "kva");
    }

    #[test]
    fn encode_rfc3492() {
        // Example taken from RFC 3492
        let params = Parameters::from_parts(&BASE_OCTAL, &[2, 3, 5]).unwrap();
        assert_eq!(encode(&[145, 62], &params), "734251");
    }

    #[test]
    fn decode_rfc3492() {
        // Example taken from RFC 3492
        let params = Parameters::from_parts(&BASE_OCTAL, &[2, 3, 5]).unwrap();
        assert_eq!(decode("734251", &params), Ok(vec![145, 62]));
        // Example taken from Wikipedia
        assert_eq!(decode("kva", &PUNYCODE), Ok(vec![745]));
    }

    #[test]
    fn decode_errors() {
        assert_eq!(
            decode("123-", &PUNYCODE),
            Err(GVLIError::DigitNotInBase('-')));
        assert_eq!(decode("kv", &PUNYCODE), Err(GVLIError::EndOfDigits));
    }

    #[test]
    fn check_punycode_construction() {
        // For convenience, `PUNYCODE` is a `const` variable constructed without
        // going through `Parameters::from_parts()`, i.e. without the checks of
        // the invariants. This test checks that these invariants would still be
        // upheld.
        let _ =
            Parameters::from_parts(&PUNYCODE.base, &PUNYCODE.thresholds)
            .unwrap();
    }
}
