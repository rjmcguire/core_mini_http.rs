[![Build Status](https://travis-ci.org/hashmismatch/core_mini_http.rs.svg?branch=master)](https://travis-ci.org/hashmismatch/core_mini_http.rs)

# core_mini_http

A small HTTP server.

This crate doesn’t use the standard library, and so requires the nightly Rust
channel.

## Usage

Get the source:

```bash
$ git clone https://github.com/hashmismatch/core_mini_http.rs
$ cd core_mini_http
```

Then build:

```bash
$ cargo build
```

And test:

```bash
$ cargo test
```

You can also run a small demo HTTP server on your localhost:

```bash
$ cargo run
```