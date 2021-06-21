# Simple helper for temporary object moving in Rust

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][license-badge]][license-url]
[![Build Status][build-badge]][build-url]

[crates-badge]: https://img.shields.io/crates/v/borrow-owned
[crates-url]: https://crates.io/crates/borrow-owned
[docs-badge]: https://docs.rs/borrow-owned/badge.svg
[docs-url]: https://docs.rs/borrow-owned
[license-badge]: https://img.shields.io/crates/l/borrow-owned
[license-url]: https://github.com/operutka/borrow-owned/blob/master/LICENSE
[build-badge]: https://travis-ci.com/operutka/borrow-owned.svg?branch=master
[build-url]: https://travis-ci.com/operutka/borrow-owned

It is useful if you need to pass your object somewhere and you cannot pass
a reference to the object because you wouldn't be able to guarantee that
the object lives long enough. For example, you don't want to use
`Arc<Mutex<_>>` for some reason but you need to pass your object to a new
thread and you need to use the object again in the original thread once the
new thread does not need the object anymore.
