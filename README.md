# RFC 3492's Generalized Variable-Length Integers

[RFC 3492](https://datatracker.ietf.org/doc/html/rfc3492), at section [3.3](https://datatracker.ietf.org/doc/html/rfc3492#section-3.3),
gives a numbering system that allows for efficient representation of consecutive
arbitrary numbers.

This crate is an implementation of this numbering system, in a generic fashion
that can be used outside of just RFC 3492 implementations.

## Example

```rust
let parameters = gvli::Parameters::from_parts(&gvli::BASE_OCTAL, &[2, 3, 5]).unwrap();

let encoded = gvli::encode(&[145, 62], &parameters);
assert_eq!(encoded, "734251");

let decoded = gvli::decode(&encoded, &parameters).unwrap();
assert_eq!(decoded, [145, 62]);
```

## References

* [RFC 3492 "Punycode: A Bootstring encoding of Unicode for Internationalized
  Domain Names in Applications (IDNA)"](https://datatracker.ietf.org/doc/html/rfc3492)
  * Section [3.3 "Generalized variable-length integers"](https://datatracker.ietf.org/doc/html/rfc3492#section-3.3)
* [Wikipedia "Punycode"](https://en.wikipedia.org/wiki/Punycode)
  * Section ["Variable-length number encoding"](https://en.wikipedia.org/wiki/Punycode#Variable-length_number_encoding)
* [Wikipedia "Numeral system"](https://en.wikipedia.org/wiki/Numeral_system)
  * Section ["Generalized variable-length integers"](https://en.wikipedia.org/wiki/Numeral_system#Generalized_variable-length_integers)
