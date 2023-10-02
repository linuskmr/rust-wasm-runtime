#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct MemArg {
	pub align: usize,
	pub offset: usize,
}