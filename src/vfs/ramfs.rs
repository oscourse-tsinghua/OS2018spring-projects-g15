// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Core/vfs/mod.rs
//! Virtual File System
use prelude::*;
use vfs;
use super::{mount, node};
use metadevs::storage::VolumeHandle;
use spin::Mutex;
use mylib::{VecMap,SparseVec};
use mylib::byte_str::{ByteStr,ByteString};
use mylib::mem::aref::{Aref,ArefInner,ArefBorrow};
use mylib::mem::Arc;
use mylib::borrow::Borrow;
use mylib::{LazyStatic};
use ata;

pub struct Driver;
pub static S_DRIVER: Driver = Driver;
static mut SATA: LazyStatic<ata::AtaVolume> = lazystatic_init!();
static mut SDISK: LazyStatic<VecMap<usize,(usize,usize)>> = lazystatic_init!();

pub const PRIO: u8 = 0;

enum RamFile
{
	File(RamFileFile),
	Dir(RamFileDir),
	Symlink(RamFileSymlink),
}
#[derive(Default)]
struct RamFileDir
{
	ents: Mutex<VecMap<ByteString,usize>>,
}
#[derive(Default)]
struct RamFileSymlink
{
	target: super::PathBuf,
}
#[derive(Default)]
struct RamFileFile
{
	ofs: usize,
	size: usize,
}
struct FileRef(ArefBorrow<RamFSInner>,ArefBorrow<RamFile>);

struct RamFS
{
	inner: ArefInner<RamFSInner>,
}
struct RamFSInner
{
	_vh: VolumeHandle,
	// TODO: Store as much data (and metadata) as possible on the volume
	// - Possibly by using an allocation pool backed onto the volume
	nodes: Mutex<SparseVec<Aref<RamFile>>>,
}

pub fn init()
{
	let h = mount::DriverRegistration::new("ramfs", &S_DRIVER);
	::core::mem::forget(h);
	unsafe{
		SATA.prep( || Default::default() );
		SATA.ls_unsafe_mut().init2(String::from("sata"),0,2048);
		SDISK.prep( || Default::default() );
		SDISK.ls_unsafe_mut().insert(65535,(0,0));
	}
}

impl mount::Driver for Driver
{
	fn detect(&self, _vol: &VolumeHandle) -> super::Result<usize> {
		// RAMFS should never bind to an arbitary volume
		Ok(0)
	}
	fn mount(&self, vol: VolumeHandle, _: mount::SelfHandle) -> super::Result<Box<mount::Filesystem>> {
		let rv = Box::new(RamFS {
			// SAFE: ArefInner must not change addresses, but because you can't move out of a boxed trait, we're good
			inner: unsafe { ArefInner::new( RamFSInner {
				_vh: vol,
				nodes: Default::default(),
				}) },
			});
		let root_inode = rv.inner.nodes.lock().insert( Aref::new(RamFile::Dir(Default::default())) );
		assert_eq!(root_inode, 0);
		Ok(rv)
	}
}

impl mount::Filesystem for RamFS
{
	fn root_inode(&self) -> node::InodeId {
		0
	}
	fn get_node_by_inode(&self, id: node::InodeId) -> Option<node::Node> {
		//log_trace!("RamFS::get_node_by_inode({})", id);
		let nodes = self.inner.nodes.lock();
		if id >= nodes.len() as node::InodeId {
			//log_log!("RamFile::get_node_by_inode - Inode {} out of range", id);
			None
		}
		else {
			let fr = Box::new(FileRef(
				self.inner.borrow(),
				nodes[id as usize].borrow()
				));
			match *nodes[id as usize]
			{
			RamFile::Dir(_) => Some(node::Node::Dir(fr)),
			RamFile::Symlink(_) => Some(node::Node::Symlink(fr)),
			RamFile::File(_) => Some(node::Node::File(fr)),//todo!("normal files"),
			}
		}
	}
}

impl FileRef {
	fn dir(&self) -> &RamFileDir {
		match &*self.1
		{
		&RamFile::Dir(ref e) => e,
		_ => panic!("Called FileRef::dir() on non-dir"),
		}
	}/*
	fn dir2(&mut self) -> &mut RamFileDir {
		unsafe{
			match &mut *self.1
			{
			&mut RamFile::Dir(ref mut e) => e,
			_ => panic!("Called FileRef::dir() on non-dir"),
			}
		}
	}*/
	fn symlink(&self) -> &RamFileSymlink {
		match &*self.1
		{
		&RamFile::Symlink(ref e) => e,
		_ => panic!("Called FileRef::symlink() on non-symlink"),
		}
	}
	fn file(&self) -> &RamFileFile {
		match &*self.1
		{
		&RamFile::File(ref e) => e,
		_ => panic!("Called FileRef::file() on non-file"),
		}
	}
	fn mut_file(&mut self) -> &mut RamFileFile {
		unsafe{
			match &mut (*(self.1.__ptr.as_ptr() as *mut ArefInner<RamFile>)).data// &mut self.1.get_data()
			{
			&mut RamFile::File(ref mut e) => e,
			_ => panic!("Called FileRef::file() on non-file"),
			}
		}
	}

	fn check_insert(&self, x1:usize,y1:usize,x2:usize,y2:usize)->bool{
		if y1<=x2 || y2<=x1{
			true
		}else{
 			false
		}
	}

	fn allocate(&mut self, id: usize, len: usize){
		unsafe{
			match SDISK.get(&id)
			{
				Some(v) => {
					SDISK.ls_unsafe_mut().remove(&id);
				},
				None => (),
			}

			for (_,blk) in SDISK.iter(){
				let mut bput=1;
				for (_,blk2) in SDISK.iter(){
					if !self.check_insert(blk.0+blk.1,blk.0+blk.1+len,blk2.0,blk2.0+blk2.1){
						bput=0;
					}
				}
				if bput==1{
					SDISK.ls_unsafe_mut().insert(id,(blk.0+blk.1,len));
					break;
				}
			}
		}
	}
}
impl node::NodeBase for FileRef {
	fn get_id(&self) -> node::InodeId {
		unimplemented!()
	}
	fn get_any(&self) -> &::core::any::Any {
		self
	}
}
impl node::Dir for FileRef {
	fn lookup(&self, name: &ByteStr) -> vfs::Result<node::InodeId> {
		let lh = &self.dir().ents.lock();
		match lh.get(name)
		{
		Some(&v) => Ok(v as node::InodeId),
		None => Err(vfs::Error::NotFound),
		}
	}
	
	fn read(&self, start_ofs: usize, callback: &mut node::ReadDirCallback) -> node::Result<usize> {
		let lh = &self.dir().ents.lock();
		let mut count = 0;
		// NOTE: This will skip/repeat entries if `create` is called between calls
		for (name, &inode) in lh.iter().skip(start_ofs)
		{
			count += 1;
			if ! callback(inode as node::InodeId, &mut name.as_bytes().iter().cloned()) {
				break ;
			}
		}
		Ok(start_ofs + count)
	}
	
	fn create(&self, name: &ByteStr, nodetype: node::NodeType) -> vfs::Result<node::InodeId> {
		use mylib::vec_map::Entry;
		//let sa = self.dir().ents;
		let mut lh = self.dir().ents.lock();
		match lh.entry(From::from(name))
		{
		Entry::Occupied(_) => Err(vfs::Error::AlreadyExists),
		Entry::Vacant(e) => {
			println!("ramfs create");
			let nn = match nodetype
				{
				node::NodeType::Dir  => RamFile::Dir (Default::default()),
				node::NodeType::File => RamFile::File(Default::default()),
				//node::NodeType::File => return Err(vfs::Error::Unknown("TODO: Files")),
				node::NodeType::Symlink(v) =>
					RamFile::Symlink(RamFileSymlink{target: From::from(v)}),
				};
			let inode = self.0.nodes.lock().insert( Aref::new(nn) );
			e.insert(inode);
			Ok(inode as node::InodeId)
			},
		}
	}
	fn link(&self, name: &ByteStr, node: &node::NodeBase) -> vfs::Result<()> {
		todo!("<FileRef as Dir>::link({:?}, inode={})", name, node.get_id())
	}
	fn unlink(&self, name: &ByteStr) -> vfs::Result<()> {
		todo!("<FileRef as Dir>::unlink({:?})", name)
	}
}
impl node::Symlink for FileRef {
	fn read(&self) -> ByteString {
		ByteString::from( ByteStr::new(&*self.symlink().target) )
	}
}

impl node::File for FileRef {
	/// Returns the size (in bytes) of this file
	fn size(&self) -> u64{
		0
	}
	/// Update the size of the file (zero padding or truncating)
	fn truncate(&self, newsize: u64) -> node::Result<u64>{
		Ok(0)
	}
	/// Clear the specified range of the file (replace with zeroes)
	fn clear(&self, ofs: u64, size: u64) -> node::Result<()>{
		Ok(())
	}
	/// Read data from the file
	fn read(&self, ofs: u64, buf: &mut [u32]) -> node::Result<usize>{
		println!("reading..");
		let sf=self.file();
		println!("ofs:{} size:{}",sf.ofs,sf.size);
		unsafe{
			match SATA.read(PRIO, ofs, buf.len()/ata::io::SECTOR_SIZE, buf)
			{
				Ok(v) => Ok(v),
				Err(e) => todo!("Error when PV fails to read: {:?}", e),
			}
		}
		//Ok(0)
	}
	/// Write data to the file, can only grow the file if ofs==size
	fn write(&self, ofs: u64, buf: &[u32]) -> node::Result<usize>{
		println!("writing..");
		let sf=self.file();
		println!("ofs:{} size:{}",sf.ofs,sf.size);
		unsafe{
			match SATA.write(PRIO, ofs, buf.len()/ata::io::SECTOR_SIZE, buf)
			{
				Ok(v) => Ok(v),
				Err(e) => todo!("Error when PV fails to read: {:?}", e),
			}
		}
		//Ok(0)
	}
	/// Write data to the file, can only grow the file if ofs==size
	fn mut_write(&mut self, id: node::InodeId, buf: &[u32]) -> node::Result<usize>{
		println!("mut writing {}..",id);
		self.allocate(id as usize, buf.len());
		for (sid,blk) in unsafe{SDISK.iter()}{
			println!("map: <{},({},{})>",sid,blk.0,blk.1);
		}
		let sf=self.mut_file();
		println!("before: ofs:{} size:{}",sf.ofs,sf.size);
		unsafe{
			match SDISK.get(&(id as usize)){
				Some(v) => {
					sf.ofs=v.0;
					sf.size=v.1;
				},
				None => (),
			}
		}
		println!("after: ofs:{} size:{}",sf.ofs,sf.size);
		unsafe{
			match SATA.write(PRIO, sf.ofs as u64, buf.len()/ata::io::SECTOR_SIZE, buf)
			{
				Ok(v) => Ok(v),
				Err(e) => todo!("Error when PV fails to read: {:?}", e),
			}
		}
		//Ok(0)
	}
}

