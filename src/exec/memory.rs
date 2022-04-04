use std::{fmt, mem};
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
		let mut memory = Memory {
			data: Vec::new(),
			page_limit: blueprint.page_limit.clone(),
			name: blueprint.export_name
		};
		// Set initial page size
		memory.grow(blueprint.page_limit.start);

		// Copy init data from data section into memory
		for init_segment in blueprint.init {
			let memory_slice_addr = (init_segment.addr, init_segment.addr + init_segment.data.len());
			memory.data[memory_slice_addr.0..memory_slice_addr.1].copy_from_slice(&init_segment.data);
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
	pub(crate) fn grow(&mut self, new_page_size: usize) {
		assert!(new_page_size >= self.page_limit.start, "Memory grow too small");
		assert!(new_page_size <= self.page_limit.end, "Memory grow too large");

		log::debug!("Memory grow to {} pages", new_page_size);
		let new_byte_size = MEMORY_PAGE_SIZE * new_page_size;
		self.data.resize(new_byte_size, 0);
	}

	pub fn page_size(&self) -> usize {
		self.data.len() / MEMORY_PAGE_SIZE
	}

	pub fn data(&self) -> &[u8] {
		&self.data
	}

	/*pub fn get<T: Value>(&self, addr: usize) -> T {
		let size = mem::size_of::<T>();
		let buf = &self.data[addr..addr+size];
		let t_ptr = buf.as_ptr();
		unsafe { mem::transmute(t_ptr) }
	}*/
}