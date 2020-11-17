extern crate serialization as ser;
#[macro_use]
extern crate serialization_derive;
#[macro_use]
extern crate lazy_static;

mod vdf;
mod config;
mod util;

pub use crate::config::MODULUS;
pub use crate::vdf::Proof;
pub use crate::vdf::{eval, prove, verify};
