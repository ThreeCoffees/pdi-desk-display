#![cfg_attr(not(test), no_std)]

mod entity;
mod html;
pub mod parser;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
extern crate std;
