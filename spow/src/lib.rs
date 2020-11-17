extern crate serialization as ser;
#[macro_use]
extern crate serialization_derive;
#[macro_use]
extern crate lazy_static;

pub mod vdf;

mod config;
mod util;

pub use crate::config::MODULUS;
