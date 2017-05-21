#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![allow(non_camel_case_types)]
#![warn(missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unused_import_braces, unused_qualifications)]
#![deny(unused_must_use, overflowing_literals)]

type GeneralError = Box<std::error::Error>;
type GeneralResult<T> = std::result::Result<T, GeneralError>;

mod consts;
use consts::msgs;

#[cfg(test)]
mod unit_tests;

pub fn lib_main(_args: Vec<String>) -> GeneralResult<()> {
    Ok(())
}
