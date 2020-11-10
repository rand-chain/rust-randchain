extern crate serialization as ser;
#[macro_use]
extern crate serialization_derive;
#[macro_use]
extern crate lazy_static;

pub mod spow;

mod config;
mod util;
mod vdf;

pub use crate::spow::SPoWResult;
