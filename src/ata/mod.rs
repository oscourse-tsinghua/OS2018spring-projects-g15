// "Tifflin" Kernel - ATA Driver
// - By John Hodge (thePowersGang)
//
// Modules/storage_ata/mylib.rs
//! x86 ATA driver
#![feature(linkage)]
#![no_std]

//#[macro_use] extern crate kernel;
//extern crate storage_scsi;

use ::prelude::*;
use ::mylib::mem::Arc;

//use ::device_manager;
use ::metadevs::storage;
//use ::async;

//module_define!{ATA, [DeviceManager, Storage], init}

//mod drivers;
pub mod io;

//pub mod volume;

#[derive(Default)]
pub struct AtaVolume
//pub trait AtaVolume: Send + 'static
{
	pub name: String,
	pub disk: u8,
	pub controller: io::DmaController,
	
	pub size: u64,
}
/*
struct AtapiVolume
{
	name: String,
	disk: u8,
	controller: io::DmaController,
}

/// Initial controller handle, owns all volumes and the first controller handle
struct ControllerRoot
{
	_controller: Arc<io::DmaController>,
	_volumes: Vec<storage::PhysicalVolumeReg>,
}

pub enum AtaClass
{
	Invalid,	// No valid response (timeout)
	None,	// No disk
	Unknown(u8,u8),	// Unknown type, values are regs 4 and 5
	Native,	// A standard ATA disk
	ATAPI,
}
impl Default for AtaClass { fn default() -> AtaClass { AtaClass::Invalid } }

/// ATA "IDENTIFY" packet data
#[repr(C)]	// All non-u16 values are aligned.
pub struct AtaIdentifyData
{
	pub flags: u16,
	_unused1: [u16; 9],
	pub serial_number: [u8; 20],
	_unused2: [u16; 3],
	pub firmware_ver: [u8; 8],
	pub model_number: [u8; 40],
	/// Maximum number of blocks per transfer
	pub sect_per_int: u16,
	_unused3: u16,
	pub capabilities: [u16; 2],
	_unused4: [u16; 2],
	/// Bitset of translation fields (next five shorts)
	pub valid_ext_data: u16,
	_unused5: [u16; 5],
	pub size_of_rw_multiple: u16,
	/// LBA 28 sector count (if zero, use 48)
	pub sector_count_28: u32,
	_unused6: [u16; 100-62],
	/// LBA 48 sector count
	pub sector_count_48: u64,
	_unused7: [u16; 2],
	/// [0:3] Physical sector size (in logical sectors
	pub physical_sector_size: u16,
	_unused8: [u16; 9],
	/// Number of words per logical sector
	pub words_per_logical_sector: u32,
	_unusedz: [u16; 256-119],
}
impl Default for AtaIdentifyData {
	fn default() -> AtaIdentifyData {
		// SAFE: Plain old data
		unsafe { ::core::mem::zeroed() }
	}
}
impl ::core::fmt::Debug for AtaIdentifyData {
	fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result
	{
		try!(write!(f, "AtaIdentifyData {{"));
		try!(write!(f, " flags: {:#x}", self.flags));
		try!(write!(f, " serial_number: {:?}", ::mylib::RawString(&self.serial_number)));
		try!(write!(f, " firmware_ver: {:?}", ::mylib::RawString(&self.firmware_ver)));
		try!(write!(f, " model_number: {:?}", ::mylib::RawString(&self.model_number)));
		try!(write!(f, " sect_per_int: {}", self.sect_per_int & 0xFF));
		try!(write!(f, " capabilities: [{:#x},{:#x}]", self.capabilities[0], self.capabilities[1]));
		try!(write!(f, " valid_ext_data: {}", self.valid_ext_data));
		try!(write!(f, " size_of_rw_multiple: {}", self.size_of_rw_multiple));
		try!(write!(f, " sector_count_28: {:#x}", self.sector_count_28));
		try!(write!(f, " sector_count_48: {:#x}", self.sector_count_48));
		try!(write!(f, " words_per_logical_sector: {}", self.words_per_logical_sector));
		try!(write!(f, "}}"));
		Ok( () )
	}
}

impl AtaVolume
{
	fn new_boxed(dma_controller: io::DmaController, disk: u8, sectors: u64) -> Box<AtaVolume>
	{
		Box::new( AtaVolume {
			name: format!("{}-{}", dma_controller.name, disk),
			disk: disk,
			controller: dma_controller,
			size: sectors,
			} )
	}
}
*/
impl AtaVolume
{
	pub fn new(sname:String, disk: u8,size:u64) -> Self {
		AtaVolume {
			name: sname.clone(),
			disk: disk,
			controller: io::DmaController{name: sname.clone()},
			size:size,
		}
	}
	fn name(&self) -> &str { &*self.name }
	fn blocksize(&self) -> usize { io::SECTOR_SIZE }
	fn capacity(&self) -> Option<u64> { Some(self.size) }
	pub fn init(&self) { self.controller.ide_init(); }
	pub fn init2(&mut self,sname:String, disk: u8,size:u64) {
		self.name= sname.clone();
		self.disk= disk;
		self.controller= io::DmaController{name: sname.clone()};
		self.size=size;
		self.init();
	}
	
	pub fn read<'a>(&'a self, _prio: u8, idx: u64, num: usize, dst: &'a mut [u32]) -> Result<usize,storage::IoError>
	{
		assert_eq!( dst.len(), num * io::SECTOR_SIZE );
		self.controller.do_dma_rd(idx, num, dst, self.disk)
	}
	pub fn write<'a>(&'a self, _prio: u8, idx: u64, num: usize, src: &'a [u32]) -> Result<usize,storage::IoError>
	{
		assert_eq!( src.len(), num * io::SECTOR_SIZE );
		//let ctrlr = &self.controller;
		self.controller.do_dma_wr(idx, num, src, self.disk)
	}
	
	fn wipe<'a>(&'a self, _blockidx: u64, _count: usize) -> Result<(),storage::IoError>
	{
		// Do nothing, no support for TRIM
		//Box::new(async::NullResultWaiter::new( || Ok( () ) ))
		Ok(())
	}
	
}
/*
impl AtapiVolume
{
	fn new(dma_controller: Arc<io::DmaController>, disk: u8) -> Self {
		AtapiVolume {
			name: format!("{}-{}", dma_controller.name, disk),
			controller: dma_controller,
			disk: disk,
		}
	}
}
impl storage_scsi::ScsiInterface for AtapiVolume
{
	fn name(&self) -> &str {
		&self.name
	}
	fn send<'a>(&'a self, command: &[u8], data: &'a [u8]) -> storage::AsyncIoResult<'a,()> {
		self.controller.do_atapi_wr(self.disk, command, data)
	}
	fn recv<'a>(&'a self, command: &[u8], data: &'a mut [u8]) -> storage::AsyncIoResult<'a,()>  {
		println!("debug: - command=[{:?}]", command);
		match command[0] & 0xE0
		{
		0x00 => assert_eq!(command.len(), 6),
		0x20 => assert_eq!(command.len(), 10),
		0x40 => assert_eq!(command.len(), 10),
		0xA0 => assert_eq!(command.len(), 12),
		0x80 => assert_eq!(command.len(), 16),
		_ => {},
		}
		self.controller.do_atapi_rd(self.disk, command, dax86ta)
	}
}

impl ControllerRoot
{
	fn new(ata_pri: u16, sts_pri: u16, irq_pri: u32,  ata_sec: u16, sts_sec: u16, irq_sec: u32,  bm: device_manager::IOBinding) -> ControllerRoot
	{
		println!("debug: ControllerRoot::new( {:#x}, {:#x}, {},  {:#x}, {:#x}, {},  {:?}",
			ata_pri, sts_pri, irq_pri,
			ata_sec, sts_sec, irq_sec,
			bm
			);
		let dma_controller = Arc::new(io::DmaController {
			name: if ata_pri == 0x1F0 {
					String::from("ATA")
				} else {
					format!("ATA{:x}", ata_pri)
				},
			ata_controllers: [
				io::AtaController::new(ata_pri, sts_pri, irq_pri),
				io::AtaController::new(ata_sec, sts_sec, irq_sec),
				],
			dma_base: bm,
			});
		let mut volumes = Vec::new();
		
		// Send IDENTIFY to all disks
		for i in 0 .. 2
		{
			let ctrlr_pri = &dma_controller.ata_controllers[0];
			let ctrlr_sec = &dma_controller.ata_controllers[1];
			
			// Create output data (defaulted, but should be written by the output function)
			let (mut identify_pri, mut type_pri) = Default::default();
			let (mut identify_sec, mut type_sec) = Default::default();
			
			// Perform IDENTIFY requests, both controllers in pararllel
			// TODO: Include a timeout to prevent a misbehaving controller from halting the system.
			{
				use ::async::Waiter;
				
				let mut wh_pri = ctrlr_pri.ata_identify(i, &mut identify_pri, &mut type_pri);
				let mut wh_sec = ctrlr_sec.ata_identify(i, &mut identify_sec, &mut type_sec);
				//let mut wh_timer = ::async::timer::Waiter::new(2*1000);
				
				// Loop until timer fires, or both disks have read
				while /* !wh_timer.is_complete() &&*/ !(wh_pri.is_complete() && wh_sec.is_complete())
				{
					::async::wait_on_list(&mut [&mut wh_pri, &mut wh_sec/*, &mut wh_timer*/], None);
				}
			}
			
			// (ugly) Handle the relevant disk types, creating devices
			let devs = [
				(i, type_pri, identify_pri),
				(2+i, type_sec, identify_sec)
				];
			for &(disk, ref class, ref ident) in devs.iter()
			{
				match *class
				{
				AtaClass::Invalid => {
					println!("log: ATA{}: Timeout", disk);
					},
				AtaClass::None => {
					println!("log: ATA{}: No disk", disk);
					},
				AtaClass::Native => {
					let sectors = if ident.sector_count_48 == 0 { ident.sector_count_28 as u64 } else { ident.sector_count_48 };
					println!("log: ATA{}: Hard Disk, {} sectors, {}", disk, sectors, storage::SizePrinter(sectors * io::SECTOR_SIZE as u64));
					volumes.push( storage::register_pv( AtaVolume::new_boxed(dma_controller.clone(), disk, sectors) ) );
					},
				AtaClass::ATAPI => {
					println!("log: ATA{}: ATAPI", disk);
					match storage_scsi::Volume::new_boxed( AtapiVolume::new(dma_controller.clone(), disk) )
					{
					Ok(scsi_vol) => volumes.push( storage::register_pv( scsi_vol ) ),
					Err(e) => println!("error: ATA{}: Error while creating SCSI device: {:?}", disk, e),
					}
					},
				AtaClass::Unknown(r4, r5) => {
					println!("warning: ATA{}: Unknown type response ({:#x}, {:#x})", disk, r4, r5);
					},
				}
			}
		}
		
		// Return a controller handle, holding on to all handles
		ControllerRoot { _controller: dma_controller, _volumes: volumes, }
	}
}

impl device_manager::DriverInstance for ControllerRoot
{
	// Just a marker trait
}
*/