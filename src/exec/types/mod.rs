mod function_signature;
mod functions;
mod identifier;
mod instruction;
mod mem_arg;
mod value;

pub use function_signature::{FunctionSignature};
pub use functions::{Callable, ExternFunction, WasmFunction, Functions};
pub use identifier::Identifier;
pub use instruction::Instruction;
pub use mem_arg::MemArg;
pub use value::Value;
use crate::exec::error::Error;

pub type ExecutionResult = Result<(), Error>;











