use std::rc::Rc;
use crate::exec::types::*;
use crate::parse::Type;

#[derive(Eq, PartialEq, Debug, Default, Clone)]
pub struct FunctionSignature {
	pub params: Vec<Type>,
	pub results: Vec<Type>,
}