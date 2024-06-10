# SHA1
A basic sha1 implementation for `#![no_std]` environments without the `alloc` crate.
Heavily based on the [rfc3174](https://datatracker.ietf.org/doc/html/rfc3174) example c implementation.

## TODO
- add rfc cases to unit tests
- make the heapless dependency optional

## Why another sha1 implementation?
Since there are already some sha1 rust crates out there, this is a valid question. The simple answer is: I couldn't find one that works with the embedded `#[no_std]` environment without `alloc` fast enough and thought "How hard can it be?".
If you know one suitable for embedded i would appreciate if you create an issue where you tell me about it :)

