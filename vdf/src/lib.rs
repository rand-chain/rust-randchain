extern crate serialization as ser;
extern crate serialization_derive;
#[macro_use]
extern crate lazy_static;

mod config;
mod vdf;

pub use config::MODULUS;
pub use vdf::Proof;
pub use vdf::{eval, prove, verify};
