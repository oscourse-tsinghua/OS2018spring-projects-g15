
#[derive(Debug, Copy, Clone)]
pub enum Type
{
	Std,
	Scroll,
	QuintBtn,	// 5 buttons
}

#[derive(Debug, Copy, Clone)]
enum State
{
	Expect(usize),
	Idle,
	// TODO: Initialise mouse to have a know config
	// TODO: Support magic to switch types up to scroll / five-button
	WaitByte2(u8),
	WaitByte3(u8,u8),
}

#[derive(Debug, Copy, Clone)]
pub struct Dev
{
	ty: Type,
	state: State,
	// guidev: gui_mouse::Instance,
	btns: u8,
}

impl Dev
{
	pub fn new(ty: Type) -> (Option<u8>,Dev) {
		// Enable scanning
		(Some(0xF4), Dev {
			ty: ty,
			state: State::Expect(0),
			// guidev: gui_mouse::Instance::new(),
			btns: 0x00,
			})
	}
	
	pub fn recv_byte(&mut self, byte: u8) -> Option<u8> {
		let (rv, ns) = match self.state
			{
			State::Expect(extra) =>
				if extra == 0 {
					(None, State::Idle)
				}
				else {
					(None, State::Expect(extra-1))
				},
			State::Idle =>
				if byte & 0x08 != 0 {
					(None, State::WaitByte2(byte))
				}
				else {
					(None, State::Idle)
				},
			State::WaitByte2(b1) =>
				(None, State::WaitByte3(b1, byte)),
			State::WaitByte3(b1, b2) => {
				assert!(is!(self.ty, Type::Std));
				let newbtns = b1 & 0b111;
				let dx = Self::get_signed_9( ((b1 >> 6) & 1) != 0, ((b1 >> 4) & 1) != 0, b2 );
				let dy = Self::get_signed_9( ((b1 >> 7) & 1) != 0, ((b1 >> 5) & 1) != 0, byte );
				println!("btns = {:#x}, (dx,dy) = ({},{})", newbtns, dx, dy);

				if dx != 0 || dy != 0 {
					// self.guidev.move_cursor(dx, -dy);
					println!("mouse: dx {}, dy {}", dx, dy);
				}
				let changed = newbtns ^ self.btns;
				if changed != 0 {
					for i in 0 .. 8 {
						let mask = 1 << i;
						if (changed & mask) != 0 {
							if (newbtns & mask) != 0 {
								// self.guidev.press_button(i as u8);
								println!("mouse: press buttion {}", i);
							}
							else {
								// self.guidev.release_button(i as u8);
								println!("mouse: release button {}", i);
							}
						}
					}
				}
				self.btns = newbtns;
				(None, State::Idle)
				},
			};

		self.state = ns;
		rv
	}


	fn get_signed_9(overflow: bool, sign: bool, val: u8) -> i16 {
		if sign {
			if overflow {
				-256
			}
			else {
				val as i16 - 0x100
			}
		}
		else {
			if overflow {
				256
			}
			else {
				(val as i16)
			}
		}
	}
}


