use super::Memory;

/// Something that can be read and written to an address in a [`Memory`].
pub trait MemObject {
	/// Creates a [MemObject] from an address in [Memory].
	fn read_from_mem(mem: &Memory, addr: usize) -> Self;

	/// Writes a [MemObject] to an address in [Memory].
	fn write_to_mem(&self, mem: &mut Memory, addr: usize);
}

impl MemObject for u32 {
	fn read_from_mem(mem: &Memory, addr: usize) -> Self {
		const BYTE_WIDTH: usize = (u32::BITS / 8) as usize;
		let mut buf = [0u8; BYTE_WIDTH];
		buf.copy_from_slice(&mem.data[addr..addr+ BYTE_WIDTH]);
		Self::from_le_bytes(buf)
	}

	fn write_to_mem(&self, mem: &mut Memory, addr: usize) {
		const BYTE_WIDTH: usize = (u32::BITS / 8) as usize;
		mem.data[addr..addr+BYTE_WIDTH].copy_from_slice(&self.to_le_bytes());
	}
}