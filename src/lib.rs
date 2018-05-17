#![feature(lang_items)]
#![feature(unique, const_unique_new)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(const_atomic_usize_new)]
#![no_std]

#![feature(core_intrinsics)]
#![feature(coerce_unsized)]
#![feature(linkage)]
#![feature(unsize)]
#![feature(get_type_id)]
#![feature(iterator_step_by)]
#![feature(optin_builtin_traits)]

#[macro_use]
mod vga_buffer;

#[doc(hidden)]
#[macro_use] pub mod macros;
/*
#[doc(hidden)]
#[macro_use] pub mod logmacros;
*/
pub mod prelude;

/// Heavy synchronisation primitives (Mutex, Semaphore, RWLock, ...)
//#[macro_use]
//pub mod sync;
/// Logging framework
//pub mod logging;

/// Thread management
//#[macro_use]
//pub mod threads;
/// Timekeeping (timers and wall time)
//pub mod time;

/// Achitecture-specific code
//pub mod arch;

//extern crate stack_dst;
//extern crate lib;
#[macro_use]
extern crate bitflags;

//#[macro_use(foo, bar)]
//extern crate baz;
extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
extern crate x86_64;
extern crate linked_list_allocator;
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate once;
//extern crate async;
//extern crate storage_ata;

//#[macro_use]
//mod async;

#[macro_use]
mod memory;

#[macro_use] pub mod mylib;

#[macro_use]
pub mod metadevs;

/// Device to driver mapping manager
///
/// Starts driver instances for the devices it sees
//pub mod device_manager;

#[macro_use]
pub mod vfs;

/// Kernel configuration
pub mod config;

//pub mod irqs;
pub mod ata;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

// use memory::heap_allocator::BumpAllocator;
use linked_list_allocator::LockedHeap;
#[global_allocator]
// static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START,
//     HEAP_START + HEAP_SIZE);
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();


#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    // ATTENTION: we have a very small stack and no guard page

    print_name();
 //   type_name!(i8);
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    enable_nxe_bit();
    enable_write_protect_bit();
    
    // set up guard page and map the heap pages
    memory::init(boot_info);

    // initialize the heap allocator
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    use alloc::boxed::Box;
    let mut heap_test = Box::new(42);
    *heap_test -= 15;
    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test2);

    let mut vec_test = vec![1,2,3,4,5,6,7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }

    for i in 0..10000 {
        format!("Some String");
    }
    // use memory::FrameAllocator;
    // for i in 0.. {
    //     use memory::FrameAllocator;
    //     // println!("{:?}", frame_allocator.allocate_frame());
    //     if let None = frame_allocator.allocate_frame() {
    //         println!("allocated {} frames", i);
    //         break;
    //     }
    // }
    
    println!("It did not crash!");
	ata_test();
    fsinit();

    loop{}
}

fn ata_test(){
	//log_trace!("PhysicalVolumeInfo::read(first={},{} bytes)", first, dst.len());
	use alloc::string::String;
	let mut dst: [u32;512]=[0;512];
	let block_size = 128;
	let sata: ata::AtaVolume=ata::AtaVolume::new(String::from("test"),0,2048);
	sata.init();
	// Read up to 'block_step' blocks in each read call
	// - TODO: Request a read of as much as possible, and be told by the device how many were serviced
	{
		//let mut buf = dst;
		let blk_id = 0;
		for i in 0..dst.len(){
			dst[i]=(i*2) as u32;
		}
		//while buf.len() > 0
		{
			assert!(dst.len() % block_size == 0);
			let prio = 0;
			let blocks = dst.len() / block_size;
			
			//write test
			match sata.write(prio, blk_id, blocks, &dst)//.wait()
			{
				Ok(v) => v,
				Err(e) => todo!("Error when PV fails to read: {:?}", e),
			};

			//read test
			// TODO: Async! (maybe return a composite read handle?)
			let real_count = match sata.read(prio, blk_id, blocks, &mut dst)//.wait()
				{
				Ok(v) => v,
				Err(e) => todo!("Error when PV fails to read: {:?}", e),
				};
			println!("real_count:{} blocks:{}",real_count,blocks);
			for i in 1..9{
				println!("dst[{}] = {}",i*64-1,dst[i*64-1]);
			}
			assert!(real_count <= blocks);
		}
	}
}


fn fsinit()
{
    println!("fs init");
	use metadevs::storage::VolumeHandle;
	use vfs::{mount,handle};
	use vfs::Path;

    metadevs::storage::init();
    vfs::init();
	// TODO: Should I automount at startup, then use chroot magic?
	//automount();
	
	// 1. Mount /system to the specified volume
	let sysdisk = config::get_string(config::Value::SysDisk);
    println!("  Mount system to the specified volume:{}",sysdisk);
	//VolumeHandle::new_ramdisk(sysdisk);
	/*match VolumeHandle::open_named(sysdisk)
	{
	Err(e) => {
		panic!("Unable to open /system volume {}: {}", sysdisk, e);
		},
	Ok(vh) => match mount::mount("/system".as_ref(), vh, "", &[])
		{
		Ok(_) => {},
		Err(e) => {
			panic!("Unable to mount /system from {}: {:?}", sysdisk, e);
			},
		},
	}*/
	
	// 2. Symbolic link /sysroot to the specified folder
    println!("  Symbolic link /sysroot to the specified folder");
	let sysroot = config::get_string(config::Value::SysRoot);
	//log_debug!("sysroot = \"{}\"", sysroot);
    println!("debug: sysroot = \"{}\"", sysroot);
	handle::Dir::open(Path::new("/")).unwrap()
		.symlink("sysroot", Path::new(&sysroot[..])).unwrap();
	
	vfs_test();
	/*
	// 3. Start 'init' (root process) using the userland loader
	let loader = ::kernel::config::get_string(::kernel::config::Value::Loader);
	let init = ::kernel::config::get_string(::kernel::config::Value::Init);
	match spawn_init(loader, init)
	{
	Ok(_) => unreachable!(),
	Err(e) => panic!("Failed to start init: {}", e),
    }*/
}

//#[cfg(DISABLED)]
fn vfs_test()
{
    println!("vfs test:");
	use vfs::handle;
	use vfs::Path;
	
	fn ls(p: &Path) {
		// - Iterate root dir
		//log_log!("ls({:?})", p);
        println!("log: ls({:?})", p);
		match handle::Dir::open(p)
		{
		Err(e) => println!("waring: '{:?}' cannot be opened: {:?}", p, e),//log_warning!("'{:?}' cannot be opened: {:?}", p, e),
		Ok(h) =>
			for name in h.iter() {
				//log_log!("{:?}", name);
                println!("log: {:?}", name);
			},
		}
	}

	// *. Testing: open a file known to exist on the testing disk	
	if true
	{
		handle::Dir::open(Path::new("/system")).unwrap().mkfile("1.TXT", handle::FileOpenMode::SharedRO).unwrap();
		println!("ls2 !");
		ls(Path::new("/system"));
		match handle::File::open( Path::new("/system/1.TXT"), handle::FileOpenMode::SharedRO )
		{
		Err(e) => println!("waring: VFS test file can't be opened: {:?}", e),//log_warning!("VFS test file can't be opened: {:?}", e),
		Ok(mut h) => {
			//log_debug!("VFS open test = {:?}", h);
            println!("debug: VFS open test = {:?}", h);
			let mut buf :[u32; 256] = [0; 256];

			for i in 0..buf.len(){
				buf[i]=(i*3) as u32;
			}

			h.mut_write(&mut buf);
			h.mut_write(&mut buf);

			let sz = h.read(0, &mut buf).unwrap();
			//let sz = h.write(0, &mut buf).unwrap();
			
			for i in 0..buf.len() {
			 	//println!("Contents: {}",buf[i]);
			}

			//log_debug!("- Contents: {:?}", ::kernel::lib::RawString(&buf[..sz]));
            //println!("debug: - Contents: {:?}", mylib::RawString(&buf[..sz]));
			//println!("debug: - Contents:");
			},
		}
		//println!("ls1 !");
		//ls(Path::new("/"));
		
		handle::Dir::open(Path::new("/system")).unwrap().mkfile("2.TXT", handle::FileOpenMode::SharedRO).unwrap();
		match handle::File::open( Path::new("/system/2.TXT"), handle::FileOpenMode::SharedRO )
		{
			Err(e) => println!("waring: VFS test file can't be opened: {:?}", e),//log_warning!("VFS test file can't be opened: {:?}", e),
			Ok(mut h) => {
				//log_debug!("VFS open test = {:?}", h);
				println!("debug: VFS open test = {:?}", h);
				let mut buf :[u32; 256] = [0; 256];

				for i in 0..buf.len(){
					buf[i]=(i*3) as u32;
				}

				h.mut_write(&mut buf);

				let sz = h.read(0, &mut buf).unwrap();
			},
		}
		match handle::File::open( Path::new("/system/1.TXT"), handle::FileOpenMode::SharedRO )
		{
			Err(e) => println!("waring: VFS test file can't be opened: {:?}", e),//log_warning!("VFS test file can't be opened: {:?}", e),
			Ok(mut h) => {
				//log_debug!("VFS open test = {:?}", h);
				println!("debug: VFS open test = {:?}", h);
				let mut buf :[u32; 512] = [0; 512];

				for i in 0..buf.len(){
					buf[i]=(i*3) as u32;
				}

				h.mut_write(&mut buf);

				let sz = h.read(0, &mut buf).unwrap();
			},
		}
	}
	
	// *. TEST Automount
	// - Probably shouldn't be included in the final version, but works for testing filesystem and storage drivers
	
	/*
	println!("automount !");
	automount();
	println!("ls3 !");
	ls(Path::new("/mount/"));*/

	//println!("ls4 !");
	//ls(Path::new("/mount/ahci?-0p0"));
}

fn automount()
{
	use metadevs::storage::VolumeHandle;
	use vfs::{Path,mount,handle};

	let mountdir = handle::Dir::open( Path::new("/") ).and_then(|h| h.mkdir("mount")).unwrap();
	handle::Dir::open( Path::new("/mount/") ).and_then(|h| h.mkdir("test1")).unwrap();
	handle::Dir::open( Path::new("/mount/") ).and_then(|h| h.mkdir("test2")).unwrap();
	handle::Dir::open( Path::new("/mount/") ).and_then(|h| h.mkdir("test3")).unwrap();
	for (_,v) in metadevs::storage::enum_lvs()
	{
		println!("v:{}",v);
		let vh = match VolumeHandle::open_named(&v)
			{
			Err(e) => {
				//log_log!("Unable to open '{}': {}", v, e);
                println!("log: Unable to open '{}': {}", v, e);
				continue;
				},
			Ok(v) => v,
			};
		mountdir.mkdir(&v).unwrap();
		let mountpt = format!("/mount/{}",v);
		match mount::mount( Path::new(&mountpt), vh, "", &[] )
		{
		Ok(_) => println!("log: Auto-mounted to {}", mountpt),//log_log!("Auto-mounted to {}", mountpt),
		Err(e) => println!("notice: Unable to automount '{}': {:?}", v, e),//log_notice!("Unable to automount '{}': {:?}", v, e),
		}
	}
}

fn enable_write_protect_bit() {
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}

fn enable_nxe_bit() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}

#[lang = "panic_fmt"] #[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\n !! KERNEL PANIC !!");
    println!("{} at line {}:", file, line);
    println!("    {}", fmt);
    loop{}
}

fn print_name() {
    vga_buffer::clear_screen();
    println!(" _______     __     __    _______    ______     _______     ________  ");
    println!("|  ____  \\  |  |   |  |  /  _____|  / _____ \\  |  _____ \\  |  ______| ");
    println!("| |____  |  |  |   |  | |  |       | |     | | | |_____ |  | |______  ");
    println!("|  ___  _/  |  |   |  | |  |       | |     | | |  ___  _/  |  ______| ");
    println!("| |   \\ \\   |  |   |  | |  |       | |     | | | |   \\ \\   | |        ");
    println!("| |    \\ \\  |  \\___/  | |  |_____  | |_____| | | |    \\ \\  | |______  ");
    println!("|_/     \\_\\  \\_______/   \\_______|  \\_______/  |_/     \\_\\ |________| ");
}