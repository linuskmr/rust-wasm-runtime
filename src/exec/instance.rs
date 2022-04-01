use crate::exec::memory::Memory;
use crate::exec::{Callable, Module};
use crate::exec::runtime::Runtime;


/// A module in execution.
#[derive(Debug)]
pub struct Instance<'a> {
	functions: Vec<Callable>,
	memories: Vec<Memory>,
}