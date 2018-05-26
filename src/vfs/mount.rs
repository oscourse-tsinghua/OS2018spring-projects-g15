// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Core/vfs/mount.rs
//! Mountpoint managment
use prelude::*;
use super::path::Path;
use super::node::{InodeId,Node,CacheHandle};
//use sync::RwLock;
use mylib::{LazyStatic,SparseVec,VecMap};

use metadevs::storage::VolumeHandle;

/// A handle to a mounted filesystem
/// 
/// Used by the node cache and maintained for a very short period of time
pub struct Handle(usize);

/// Handle to a mounted filesystem held by the filesystem itself
///
/// Allows access to the node cache
pub struct SelfHandle(usize);

/// Internal representation of a mounted volume 安装卷内部表示
struct MountedVolume
{
	mountpoint_node: CacheHandle,
	fs: Box<Filesystem>,
}


/// Filesystem instance trait (i.e. the instance
pub trait Filesystem:
	Send + Sync
{
	fn root_inode(&self) -> InodeId;
	fn get_node_by_inode(&self, InodeId) -> Option<Node>;
}

struct NullFs;
impl Filesystem for NullFs {
	fn root_inode(&self) -> InodeId { 0 }
	fn get_node_by_inode(&self, _: InodeId) -> Option<Node> { None }
}

/// Filesystem instance trait
pub trait Driver:
	Send + Sync
{
	/// Returns an integer bindng strength where 0 means "doesn't handle"
	///
	/// Levels are left unspecified, but FAT uses 1, and extN uses 2/3 (depending on if the system is fully supported)
	fn detect(&self, vol: &VolumeHandle) -> super::Result<usize>;

	/// Mount the provided volume as this filesystem
	///
	/// NOTE: `handle` isn't actually usable until after this function returns
	fn mount(&self, vol: VolumeHandle, handle: SelfHandle) -> super::Result<Box<Filesystem>>;
}

pub struct DriverRegistration(&'static str);

static mut cnt:usize = 0;
/// Known drivers 已知驱动
static mut S_DRIVERS: LazyStatic< VecMap<&'static str, &'static Driver> > = lazystatic_init!();
/// Mounted volumes 已安装volume
static mut S_VOLUMES: LazyStatic< SparseVec<MountedVolume> > = lazystatic_init!();
/// Root mount 根
static mut S_ROOT_VOLUME: Option<Box<Filesystem>> = None;

pub fn init()
{
	// SAFE: Running in a single-threaded context
	unsafe {
		cnt=0;
		S_DRIVERS.prep( || Default::default() );
		S_VOLUMES.prep( || Default::default() );
	}
}

/// Mount a volume at the provided location 在提供位置安装volume
// TODO: Parse options
pub fn test(){
	//println!("test:");
	DriverRegistration::new("ramfs",&super::ramfs::S_DRIVER);
}
pub fn mount2(location: &Path, fs: &str, _options: &[&str]) -> Result<(),MountError>
{
	//println!("mount2:");
	DriverRegistration::new("ramfs",&super::ramfs::S_DRIVER);
	Ok(())
}
pub fn mount(location: &Path, vol: VolumeHandle, fs: &str, _options: &[&str]) -> Result<(),MountError>
{
	//println!("mount: ");
	//DriverRegistration::new("ramfs",&super::ramfs::S_DRIVER);
	//DriverRegistration::new("ramfs",&super::ramfs::S_DRIVER);
	unsafe{println!("cnt: {}",cnt);}
	unsafe{
		match super::mount::S_DRIVERS.get("ramfs"){
			Some(_) => println!("insert success"),
			None => println!("insert failed"),
		}
	}
	let drivers = unsafe{S_DRIVERS.ls_unsafe_mut()};
	// 1. (maybe) detect filesystem 检测文件系统
	let driver = if fs == "" {
			match drivers.iter()
				.filter_map(|(n,fs)| fs.detect(&vol).ok().map(|r| (r, n, fs)))
				.max_by_key(|&(l,_,_)| l)
			{
			Some((0,_,_)) => return Err(MountError::NoHandler),
			Some((_,_name,fs)) => fs,
			None => return Err(MountError::NoHandler),
			}
		}
		else {
			match drivers.get(fs)
			{
			Some(d) => d,
			None => {
				//log_notice!("Filesystem '{}' not registered", fs);
				println!("notice: Filesystem '{}' not registered", fs);
				return Err(MountError::UnknownFilesystem);
				},
			}
		};
	
	if location == Path::new("/")
	{
		let fs: Box<_> = match driver.mount(vol, SelfHandle(0))
			{
			Ok(v) => v,
			Err(_) => return Err(MountError::CallFailed),
			};
		let lh = unsafe{&S_ROOT_VOLUME};
		if lh.is_some() {
			//log_warning!("TODO: Support remounting /");
			return Err(MountError::MountpointUsed);
		}
		unsafe{S_ROOT_VOLUME = Some(fs);}
	}
	else
	{
		// 2. Acquire mountpoint 获得挂载点
		let nh = match CacheHandle::from_path(location)
			{
			Ok(nh) => nh,
			Err(_) => return Err(MountError::InvalidMountpoint),
			};
		if ! nh.is_dir() {
			return Err(MountError::InvalidMountpoint);
		}
		if nh.is_mountpoint() {
			return Err(MountError::MountpointUsed);
		}
		
		// 3. Reserve the mountpoint ID (using a placeholder instance)
		// NOTE: Nothing should know of this index until after mount is completed
		unsafe{S_VOLUMES.ls_unsafe_mut().insert(MountedVolume { mountpoint_node: nh, fs: Box::new(NullFs) })};
		let vidx = unsafe{S_VOLUMES.len()-1};

		// 4. Mount and register volume
		let fs = match driver.mount(vol, SelfHandle(vidx))
			{
			Ok(v) => v,
			Err(_) => return Err(MountError::CallFailed),
			};

		// 5. Store and bind to mountpoint
		{
			let lh = unsafe{S_VOLUMES.ls_unsafe_mut()};
			lh[vidx].fs = fs;
			if lh[vidx].mountpoint_node.mount(vidx + 1) == false {
				lh.remove(vidx);
				return Err(MountError::MountpointUsed);
			}
		}
	}

	Ok( () )
}
#[derive(Debug)]
pub enum MountError
{
	UnknownFilesystem,
	NoHandler,
	InvalidMountpoint,
	MountpointUsed,
	CallFailed,
}
impl_fmt! {
	Display(self,f) for MountError {
		write!(f, "{}", match self
			{
			&MountError::UnknownFilesystem => "Filesystem driver not found",
			&MountError::NoHandler => "No registered filesystem driver handles this volume",
			&MountError::InvalidMountpoint => "The specified mountpoint was invalid",
			&MountError::MountpointUsed => "The specified mountpoint was already used",
			&MountError::CallFailed => "Driver's mount call failed",
			})
	}
}


impl DriverRegistration
{
	pub fn new(name: &'static str, fs: &'static Driver) -> Option<DriverRegistration> {
		unsafe{/*
			println!("new driver:");
			cnt=cnt+1;
			println!("cnt: {}",cnt);
			match S_DRIVERS.get("ramfs"){
				Some(_) => println!("insert success"),
				None => println!("insert failed"),
			};*/
			let res=match S_DRIVERS.ls_unsafe_mut().entry(name)
			{
			::mylib::vec_map::Entry::Vacant(e) => {
				e.insert(fs);
				Some(DriverRegistration(name))
				},
			::mylib::vec_map::Entry::Occupied(_) => {
				println!("entry failed");
				None
				},
			};/*
			match S_DRIVERS.get("ramfs"){
				Some(_) => println!("insert success"),
				None => println!("insert failed"),
			};*/
			println!("");
			res
		}
	}
}

impl Handle
{
	pub fn from_id(id: usize) -> Handle {
		if id == 0 {
			Handle(0)
		}
		else {
			if ! unsafe{S_VOLUMES.ls_unsafe_mut().get(id-1).is_some()} {
				panic!("Handle::from_id - ID {} not valid", id);
			}
			Handle(id)
		}
	}
	
	pub fn id(&self) -> usize {
		self.0
	}
	pub fn root_inode(&self) -> InodeId {
		self.with_fs(|fs| fs.root_inode())
	}
	
	pub fn get_node(&self, id: InodeId) -> Option<Node> {
		self.with_fs(|fs| fs.get_node_by_inode(id))
	}

	fn with_fs<R, F: FnOnce(&Filesystem)->R>(&self, f: F) -> R {
		if self.0 == 0 {
			unsafe{f(&**S_ROOT_VOLUME.as_ref().unwrap())}
		}
		else {
			unsafe{f(&*S_VOLUMES.get(self.0 - 1).unwrap().fs)}
		}
	}
}


impl SelfHandle
{
	pub fn get_node(&self, inode: InodeId) -> super::Result<super::node::CacheHandle> {
		super::node::CacheHandle::from_ids(self.0, inode)
	}
}

