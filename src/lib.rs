#![no_std]

#![feature(alloc)]
#![feature(no_std)]
#![feature(collections)]
#![feature(convert)]
#![feature(vec_push_all)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate collections;

// for tests
#[cfg(test)]
#[macro_use]
extern crate std;

mod http;
mod router;
mod parser;
mod ssdp;
mod url;

pub use http::*;
pub use router::*;
pub use parser::*;
pub use ssdp::*;
pub use url::*;
