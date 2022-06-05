#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

mod day;
mod errors;
mod meal;
mod menu;
pub mod menus;
mod util;

pub use day::*;
pub use errors::*;
pub use meal::*;
pub use menu::*;
