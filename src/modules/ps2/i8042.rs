use x86_64::instructions::port::{inb, outb};
use spin::Mutex;

use arch::driver::acpi;

struct Port {
	is_second: bool,
	dev: super::PS2Dev,
}

#[derive(Default)]
pub struct Ctrlr8042 {
	// port1: Option< irqs::ObjectHandle >,
	// port2: Option< irqs::ObjectHandle >,
}

lazy_static! {
    pub static ref S_8042_CTRLR: Mutex<Ctrlr8042> = Mutex::new(unsafe{Ctrlr8042::new()}.expect("No 8042 Controller!"));
}

pub fn init() {
	// 1. Check with ACPI is this machine has a PS2 controller
	let sdt = acpi::find_sdt("FACP");
	let enabled = if sdt.len() == 1 {
		let fadt = acpi::fadt::Fadt::new(sdt[0]).expect("No FACP");
		let boot_architecture_flags = fadt.boot_architecture_flags;
		if fadt.header.revision > 1 {
			debug!("FADT boot_architecture_flags = {:#x}", boot_architecture_flags);
			boot_architecture_flags & 2 != 0
		} else {
			debug!("FADT revision 1, assuming 8042 present");
			true
		}
	} else {
		debug!("No FADT, assuming 8042 present");
		true
	};
	
	if !enabled {
		debug!("8042 PS/2 Controller disabled due to ACPI");
	}
}

impl Port {
	fn new(is_second: bool) -> Port {
		Port {
			is_second: is_second,
			dev: Default::default(),
		}
	}
	unsafe fn send_byte(&mut self, b: u8) {
		// EVIL: Obtains a new instance of the controller to use its methods
		// - Should be safe to do, as long as we don't get two IRQs running at the same time
		debug!("PS2 TX {} {:#02x}", if self.is_second { "sec" } else { "pri" }, b);
		let mut c = S_8042_CTRLR.lock();
		if self.is_second {
			c.write_cmd(0xD4);
		}
		c.write_data(b);
	}
}

impl Ctrlr8042
{
	unsafe fn new() -> Result<Ctrlr8042,()> {
		let mut ctrlr = Ctrlr8042::default();
		// 1. Disable the controller during setup
		ctrlr.write_cmd(0xAD);	// Disable primary channel
		ctrlr.write_cmd(0xA7);	// Disable secondary channel (ignored if none)
		// - Flush the input FIFO
		ctrlr.flush();
		
		// Read, Modify, Write the controller's config
		ctrlr.write_cmd(0x20);
		let mut config = ctrlr.read_data().ok().expect("Timeout reading PS/2 config");
		// - Clear 0: Port1 Enable, 1: Port2 Enable, 6: Port1 Translation
		config &= !( (1<<0)|(1<<1)|(1<<6) );
		let can_have_second_port = config & (1<<5) != 0;
		ctrlr.write_cmd(0x60);
		ctrlr.write_data(config);
		
		// Self-test
		ctrlr.write_cmd(0xAA);
		match ctrlr.read_data() {
		Ok(0x55) => {},
		Ok(v) => panic!("PS/2 self-test failed ({:#x} exp 0x55)", v),
		Err(_) => panic!("Timeout waiting for PS/2 self-test"),
		}
		
		let has_second_port = if can_have_second_port {
				ctrlr.write_cmd(0xA8);	// Enable second port
				ctrlr.write_cmd(0x20);
				let config = ctrlr.read_data().ok().expect("Timeout reading PS/2 config (2)");
				ctrlr.write_cmd(0xA7);	// Disable secondary channel (ignored if none)
				// If bit is still set, then the second channel is absent
				config & (1 << 5) == 0
			}
			else {
				false
			};
		
		// - Flush the input FIFO (again)
		//  > Just in case data arrived while twiddling with ports
		ctrlr.flush();
		
		let port1_works = {
			ctrlr.write_cmd(0xAB);
			ctrlr.read_data().unwrap() == 0x00
			};
		let port2_works = if has_second_port {
				ctrlr.write_cmd(0xA9);
				ctrlr.read_data().unwrap() == 0x00
			} else {
				false
			};
		debug!("can_have_second_port={:?}, has_second_port={:?}, port1_works={:?}, port2_works={:?}",
			can_have_second_port, has_second_port, port1_works, port2_works);
		
		if !port1_works && !port2_works {
			// nothing works, give up
			debug!("Handle no ports working");
		}
		
		// Enable working ports.
		// - Enable interrupts first
		ctrlr.write_cmd(0x20);
		let mut config = ctrlr.read_data().ok().expect("Timeout reading PS/2 config (2)");
		if port1_works {
			config |= 1 << 0;	// Enable interrupt
		}
		if port2_works {
			config |= 1 << 1;	// Enable interrupt
		}
		debug!("Controller config = 0b{:08b}", config);
		ctrlr.write_cmd(0x60);
		ctrlr.write_data(config);
		// - Enable ports second
		if port1_works {
			let mut port = Port::new(false);
			debug!("Enabling port 1");
			// ctrlr.port1 = Some( irq::bind_object(1, Box::new(move || port.handle_irq())) );
			ctrlr.write_cmd(0xAE);
			ctrlr.write_data(0xFF);
		}
		if port2_works {
			let mut port = Port::new(true);
			debug!("Enabling port 2");
			// ctrlr.port2 = Some( irq::bind_object(12, Box::new(move || port.handle_irq())) );
			ctrlr.write_cmd(0xA8);
			ctrlr.write_cmd(0xD4);
			ctrlr.write_data(0xFF);
		}
		
		Ok( ctrlr )
	}
	
	/// true if write is possible
	unsafe fn poll_out(&mut self) -> bool {
		inb(0x64) & 2 == 0
	}
	/// true if read is possible
	unsafe fn poll_in(&mut self) -> bool {
		inb(0x64) & 1 != 0
	}
	
	unsafe fn wait_out(&mut self) -> Result<(),()> {
		const MAX_SPINS: usize = 1000;
		let mut spin_count = 0;
		while !self.poll_out() {
			spin_count += 1;
			if spin_count == MAX_SPINS {
				return Err( () );
			}
		}
		Ok( () )
	}
	unsafe fn wait_in(&mut self) -> Result<(),()> {
		const MAX_SPINS: usize = 100*1000;
		let mut spin_count = 0;
		while !self.poll_in() {
			spin_count += 1;
			if spin_count == MAX_SPINS {
				return Err( () );
			}
		}
		Ok( () )
	}
	
	pub unsafe fn write_cmd(&mut self, byte: u8) {
		if let Err(_) = self.wait_out() {
			debug!("Handle over-spinning in PS2 controller write");
		}
		outb(0x64, byte);
	}
	pub unsafe fn write_data(&mut self, byte: u8) {
		if let Err(_) = self.wait_out() {
			debug!("Handle over-spinning in PS2 controller write");
		}
		outb(0x60, byte);
	}
	pub unsafe fn read_data(&mut self) -> Result<u8,()> {
		try!( self.wait_in() );
		Ok( inb(0x60) )
	}
	pub unsafe fn flush(&mut self) {
		while self.poll_in() {
			inb(0x60);
		}
	}
}


