#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
pub mod errors;
pub mod menus;
mod util;

mod day;
mod meal;
mod menu;

pub use day::*;
pub use meal::*;
pub use menu::*;
