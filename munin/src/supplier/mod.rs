use stor::{menu::Patch, Day};

pub mod kleins;
pub mod matilda;
pub mod mpi;
pub mod sabis;
pub mod skolmaten;
pub mod sodexo;

#[derive(Debug)]
pub struct ListDays {
    pub menu: Patch,
    pub days: Vec<Day>,
}
