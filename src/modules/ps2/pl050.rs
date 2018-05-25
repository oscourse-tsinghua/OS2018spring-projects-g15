// Modules/input_ps2/pl050.rs
//! ARM PL050 PS/2 Controller

use spin::Mutex;
use alloc::vec::Vec;
use syscall::io::{Mmio, ReadOnly};

//const PL050_RXREADY: u32 = 0x??;
const PL050_TXBUSY: u32 = 0x20;

struct Port
{
	base: Mmio<u32>,
	dev: super::PS2Dev,
}

lazy_static! {
    pub static ref S_PORTS: Mutex< Vec<irqs::ObjectHandle> > = Mutex::new(unsafe{Vec<irqs::ObjectHandle>::new()});
}

pub fn init() {}

impl Port{}

