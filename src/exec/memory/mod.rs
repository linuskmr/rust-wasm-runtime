use std::{fmt, usize};
use std::ops::Range;
use crate::parse::MemoryBlueprint;
pub use mem_object::MemObject;

mod mem_object;


pub const MEMORY_PAGE_SIZE: usize = 4096;

#[derive(Default, PartialEq, Eq)]
pub struct Memory {
	pub data: Vec<u8>,
	/// Minimum and maximum page limit.
	pub page_limit: Range<usize>,
	pub name: Option<String>,
}

impl From<MemoryBlueprint> for Memory {
	fn from(blueprint: MemoryBlueprint) -> Self {
		let mut memory = Memory {
			data: Vec::new(),
			page_limit: blueprint.page_limit.clone(),
			name: blueprint.export_name
		};
		// Set initial page size
		memory.grow(blueprint.page_limit.start);

		// Copy init data from data section into memory
		for init_segment in blueprint.init {
			let memory_slice_addr = init_segment.addr..init_segment.addr+init_segment.data.len();
			memory.data[memory_slice_addr].copy_from_slice(&init_segment.data);
		}
		memory
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
	/// Grow the memory to `new_page_size` * [`MEMORY_PAGE_SIZE`] bytes.
	#[tracing::instrument(skip(self))]
	pub fn grow(&mut self, new_page_size: usize) {
		assert!(new_page_size >= self.page_limit.start, "Memory grow too small");
		assert!(new_page_size <= self.page_limit.end, "Memory grow too large");

		let new_byte_size = MEMORY_PAGE_SIZE * new_page_size;
		self.data.resize(new_byte_size, 0);
	}

	/// Get the current page size.
	pub fn page_size(&self) -> usize {
		self.data.len() / MEMORY_PAGE_SIZE
	}

	/// Immutable access to the complete memory data.
	pub fn data(&self) -> &[u8] {
		&self.data
	}

	/// Read a [`MemObject`] from an address in memory.
	pub fn read<T: MemObject>(&self, addr: usize) -> T {
		T::read_from_mem(&self, addr)
	}

	/// Write a [`MemObject`] to an address in memory.
	pub fn write<T: MemObject>(&mut self, mem_object: &T, addr: usize) {
		mem_object.write_to_mem(self, addr)
	}
}