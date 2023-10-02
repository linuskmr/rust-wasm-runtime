use std::fmt;
use std::rc::Rc;
use crate::exec::types::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Identifier {
	pub module: String,
	pub field: String,
}

impl From<(&'static str, &'static str)> for Identifier {
	fn from(identifier: (&'static str, &'static str)) -> Self {
		Self {
			module: identifier.0.to_owned(),
			field: identifier.1.to_owned()
		}
	}
}

impl fmt::Display for Identifier {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if !self.module.is_empty() {
			write!(f, "{}.{}", self.module, self.field)
		} else {
			write!(f, "{}", self.field)
		}
	}
}