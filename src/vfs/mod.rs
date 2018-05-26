// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Core/vfs/mod.rs
//! Virtual File System
#[allow(unused_imports)]
use prelude::*;
use metadevs::storage::VolumeHandle;
use mylib::byte_str::{ByteStr,ByteString};

//module_define!(VFS, [], init);

pub type Result<T> = ::core::result::Result<T,Error>;

#[derive(Debug)]
pub enum Error
{
	/// File not found
	NotFound,
	/// Permission denied
	PermissionDenied,
	/// File exclusively locked
	Locked,
	/// The item already exists
	AlreadyExists,

	/// Path was malformed (too long, not absolute, not normalised, ... depends)
	MalformedPath,
	/// A general parameter was malformed (empty filename, offset out of range, ...)
	InvalidParameter,
	/// Node was not the requested type (or selected FS driver doesn't support that volume)
	TypeMismatch,
	/// A component of the path was not a directory
	NonDirComponent,
	/// Symbolic link recursion limit reached
	RecursionDepthExceeded,


	/// Block-level IO Error
	BlockIoError(::metadevs::storage::IoError),
	/// Filesystem is read-only
	ReadOnlyFilesystem,
	/// Filesystem driver hit an internal consistency error
	InconsistentFilesystem,
	/// Volume ran out of space
	OutOfSpace,

	/// System has run out of memory
	OutOfMemory,

	/// Operation failed due to a transient error, can can be retried
	TransientError,

	/// Unknown (misc) error
	Unknown(&'static str),
}
impl From<::metadevs::storage::IoError> for Error {
	fn from(v: ::metadevs::storage::IoError) -> Error {
		Error::BlockIoError(v)
	}
}
//impl_fmt! {
//	Display(self, f) for Error {
//		match self
//		{
//		&Error::NotFound => "File not found",
//		&Error::PermissionDenied => "Permission denied",
//		}
//	}
//}

pub use self::path::{Path,PathBuf};

pub mod node;
pub mod mount;
pub mod handle;
mod path;
mod ramfs;

pub fn init()
{
	// 1. Initialise global structures
	mount::init();
	node::init();
	//ramfs::init();
	// 2. Start the root/builtin filesystems
	//let h = mount::DriverRegistration::new("ramfs", &ramfs::S_DRIVER);
	//let h = mount::DriverRegistration::new("ramfs", &ramfs::S_DRIVER);
	mount::DriverRegistration::new("ramfs",&ramfs::S_DRIVER);
	//mount::test();
	let sv=VolumeHandle::new_ramdisk(0);
	ramfs::init();
	mount::mount("/".as_ref(), sv, "ramfs", &[]).expect("Unable to mount /");
	// 3. Initialise root filesystem layout
	let root = match handle::Dir::open( Path::new("/") )
		{
		Ok(v) => v,
		Err(e) => panic!("BUG - Opening '/' failed: {:?}", e),
		};
	root.mkdir("system").unwrap();
	root.mkdir("volumes").unwrap();
	root.mkdir("temp").unwrap();
}

pub fn readFile(path: &str, dst: &mut [u32]){
	match handle::File::open( Path::new(&path), handle::FileOpenMode::SharedRO )
	{
		Err(e) => println!("waring: VFS test file can't be opened: {:?}", e),
		Ok(mut h) => {
			println!("debug: VFS open test = {:?}", h);
			let mut buf :[u32; 128] = [0; 128];
			let sz = h.read(0, &mut buf).unwrap();

			debug!("read:{}",path);
			for i in 0..dst.len(){
				dst[i]=buf[i] as u32;
			}
		},
	}
}

pub fn writeFile(path: &str, src: &[u32]){
	match handle::File::open( Path::new(path), handle::FileOpenMode::SharedRO )
	{
		Err(e) => println!("waring: VFS test file can't be opened: {:?}", e),
		Ok(mut h) => {
			//println!("debug: VFS open test = {:?}", h);
			let mut buf :[u32; 128] = [0; 128];

			for i in 0..src.len(){
				buf[i]=src[i] as u32;
			}
			debug!("write:{}",path);
			h.mut_write(&mut buf);
		},
	}
}