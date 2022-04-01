use std::fmt;
use std::ops::Range;
use crate::parse::MemoryBlueprint;


const MEMORY_PAGE_SIZE: usize = 4096;

#[derive(Default, PartialEq, Eq)]
pub struct Memory {
	pub(crate) data: Vec<u8>,
	/// Minimum and maximum page limit.
	pub(crate) page_limit: Range<usize>,
	pub(crate) name: Option<String>,
}

impl From<MemoryBlueprint> for Memory {
	fn from(blueprint: MemoryBlueprint) -> Self {
		Self {
			data: vec![0u8; blueprint.page_limit.start],
			page_limit: blueprint.page_limit,
			name: blueprint.name
		}
	}
}

impl fmt::Debug for Memory {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Do not print self.data because it is very large
		f.debug_struct("Memory")
			.field("limit", &self.page_limit)
			.field("name", &self.name)
			.finish()
	}
}

impl Memory {
	pub(crate) fn init(&mut self) {
		self.grow(self.page_limit.start);
	}

	pub(crate) fn grow(&mut self, new_page_size: usize) {
		assert!(new_page_size >= self.page_limit.start, "Memory grow too small");
		assert!(new_page_size <= self.page_limit.end, "Memory grow too large");

		log::debug!("Memory grow to {} pages", new_page_size);
		let new_byte_size = MEMORY_PAGE_SIZE * new_page_size;
		self.data.resize(new_byte_size, 0);
	}

	fn page_size(&self) -> usize {
		self.data.len() / MEMORY_PAGE_SIZE
	}
}