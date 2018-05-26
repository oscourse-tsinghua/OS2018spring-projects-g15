mod i8042;
mod keyboard;
#[macro_use]
mod mouse;
mod keycodes;

use x86_64::instructions::port::{inb, outb};
use spin::Mutex;
use self::i8042::Port;

#[derive(Debug, Copy, Clone)]
enum PS2Dev
{
	None,
	Unknown,
	Enumerating(EnumWaitState),
	Keyboard(keyboard::Dev),
	Mouse(mouse::Dev),
}
impl Default for PS2Dev { fn default() -> Self { PS2Dev::None } }

#[derive(Copy,Clone,Debug)]
enum EnumWaitState
{
	DSAck,
	IdentAck,
	IdentB1,
	IdentB2(u8),
}

static mouse_std: mouse::Type = mouse::Type::Std;
static mouse_scr: mouse::Type = mouse::Type::Scroll;
static mouse_qbt: mouse::Type = mouse::Type::QuintBtn;
static kbd_MF2: keyboard::Type = keyboard::Type::MF2;
static kbd_MF2Emul: keyboard::Type = keyboard::Type::MF2Emul;


static MOUSE_DEV: Mutex<Option<PS2Dev>> = Mutex::new(None);
static KBD_DEV: Mutex<Option<PS2Dev>> = Mutex::new(None);

static port1: Mutex<Option<Port>> = Mutex::new(None);
static port2: Mutex<Option<Port>> = Mutex::new(None);

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
pub fn init() {
	i8042::init();
	*KBD_DEV.lock() = PS2Dev::new_keyboard(&kbd_MF2).1;
	*MOUSE_DEV.lock() = PS2Dev::new_mouse(&mouse_std).1;

	*port1.lock() = Some(Port::new(false));
	*port2.lock() = Some(Port::new(true));

	// let (mouse_byte, mouse_dev) = PS2Dev::new_mouse(mouse::Type::Std);
	// let (kbd_byte, kbd_dev) = PS2Dev::new_keyboard(keyboard::Type::MF2);
	// use arch::idt::IDT;
	// use consts::irq::*;
	// IDT.set_irq_handler(IRQ_KBD as usize, handle_irq_kbd);
	// IDT.interrupts[IRQ_KBD as usize].set_handler_fn(handle_irq_kbd);
	// IDT.interrupts[IRQ_MOUSE as usize].set_handler_fn(handle_irq_mouse);
}

use self::i8042::{write_cmd, write_data};
pub fn handle_irq_kbd()
{
	// SAFE: Current impl avoids most races, but can misbehave (returnign bad data) if an IRQ happens between the inb calls
	unsafe {
		// let mask = 0x01;
		// if inb(0x64) & mask == 0 {
		// 	return
		// }
		// else {
		// 	let b = inb(0x60);
		// 	debug!("PS2 RX pri {:#02x}", b);
		// 	if let Some(ob) = KBD_DEV.lock().expect("No kbd dev").recv_byte(b) {
		// 		write_data(ob);
		// 	}
		// }
		if let Some(ref mut rp) = *port1.lock() {
			rp.handle_irq();
		}
	}
}

pub fn handle_irq_mouse()
{
	// SAFE: Current impl avoids most races, but can misbehave (returnign bad data) if an IRQ happens between the inb calls
	unsafe {
		// NOTE: This matches qemu's behavior, but the wiki says it's chipset dependent
		// let mask = 0x20;
		// if inb(0x64) & mask == 0 {
		// 	return
		// }
		// else {
		// 	let b = inb(0x60);
		// 	debug!("PS2 RX second {:#02x}", b);
		// 	if let Some(ob) = MOUSE_DEV.lock().expect("No mouse dev").recv_byte(b) {
		// 		write_cmd(0xD4);
		// 	}
		// }
		if let Some(ref mut rp) = *port2.lock() {
			rp.handle_irq();
		}
	}
}

impl PS2Dev {
	fn new_mouse(ty: &'static mouse::Type) -> (Option<u8>, Option<PS2Dev>) {
		let (byte, dev) = mouse::Dev::new(*ty);
		(byte, Some(PS2Dev::Mouse(dev)))
	}
	fn new_keyboard(ty: &'static keyboard::Type) -> (Option<u8>, Option<PS2Dev>) {
		let (byte, dev) = keyboard::Dev::new(*ty);
		(byte, Some(PS2Dev::Keyboard(dev)))
	}
	
	/// Handle a recieved byte, and optionally return a byte to be sent to the device
	pub fn recv_byte(&mut self, byte: u8) -> Option<u8> {
		let (rv, new_state): (Option<_>,Option<_>) = match *self
			{
			PS2Dev::None =>
				// TODO: Clean this section up, the OSDev.org wiki is a little hazy on the ordering
				if byte == 0xFA {
					(None, None)
				}
				else if byte == 0xAA {
					// Send 0xF5 "Disable Scanning" and wait for ACK
					(Some(0xF5), Some(PS2Dev::Enumerating(EnumWaitState::DSAck)))
				}
				else {
					(None, None)
				},
			PS2Dev::Unknown => (None, None),
			PS2Dev::Enumerating(state) => match state
				{
				EnumWaitState::DSAck =>
					if byte == 0xFA {
						// Send 0xF2 "Identify"
						(Some(0xF2), Some(PS2Dev::Enumerating(EnumWaitState::IdentAck)))
					}
					else if byte == 0x00 {
						// XXX: Ignore spurrious NUL byte
						(None, None)
					}
					else {
						(None, Some(PS2Dev::Unknown))
					},
				EnumWaitState::IdentAck =>
					if byte == 0xFA {
						// TODO: Start a timeout if not enough bytes are sent
						(None, Some(PS2Dev::Enumerating(EnumWaitState::IdentB1)))
					}
					else {
						(None, Some(PS2Dev::Unknown))
					},
				EnumWaitState::IdentB1 =>
					match byte
					{
					0x00 => {
						let mut res = Self::new_mouse(&mouse_std);
						*MOUSE_DEV.lock() = res.1;
						res
						},
					0x03 => {
						let res = Self::new_mouse(&mouse_scr);
						*MOUSE_DEV.lock() = res.1;
						res
						},
					0x04 => {
						let res = Self::new_mouse(&mouse_qbt);
						*MOUSE_DEV.lock() = res.1;
						res
						},
					0xAB => (None, Some(PS2Dev::Enumerating(EnumWaitState::IdentB2(byte)))),
					_ => {
						debug!("Unknown PS/2 device {:#02x}", byte);
						(None, Some(PS2Dev::Unknown))
						},
					},
				EnumWaitState::IdentB2(b1) =>
					match (b1,byte)
					{
					(0xAB, 0x83) => {
						let res = Self::new_keyboard(&kbd_MF2);
						*KBD_DEV.lock() = res.1;
						res
						},
					(0xAB, 0x41) => {
						let res = Self::new_keyboard(&kbd_MF2Emul);
						*KBD_DEV.lock() = res.1;
						res
						},
					(0xAB, 0xC1) => {
						let res = Self::new_keyboard(&kbd_MF2Emul);
						*KBD_DEV.lock() = res.1;
						res
						},
					_ => {
						debug!("Unknown PS/2 device {:#02x} {:#02x}", b1, byte);
						(None, Some(PS2Dev::Unknown))
						},
					},
				},
			PS2Dev::Keyboard(ref mut dev) => {
				(dev.recv_byte(byte), None)
				},
			PS2Dev::Mouse(ref mut dev) => {
				(dev.recv_byte(byte), None)
				},
			};
		
		if let Some(ns) = new_state
		{
			debug!("Byte {:#02x} caused State transition {:?} to {:?}", byte, *self, ns);
			*self = ns;
		}
		rv
	}
}

