pub mod paging;
pub mod driver;
pub mod cpu;
pub mod interrupts;

pub fn init() {
	cpu::enable_nxe_bit();
	cpu::enable_write_protect_bit();
}