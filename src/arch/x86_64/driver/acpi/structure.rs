use utils::{Checkable, bytes_sum};
use core::mem::size_of;

#[repr(C)]
#[derive(Debug)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oemId: [i8; 6],
    pub revision: u8,
    pub rsdtPhyAddr: u32,
    pub length: u32,
    pub xsdtPhyAddr: u64,
    pub extChecksum: u8,
    pub reserved: [u8; 3],
}

impl Checkable for Rsdp {
    fn check(&self) -> bool {
        &self.signature == b"RSD PTR " && bytes_sum(self) == 0
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Header {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub oemId: [i8; 6],
    pub oemTableId: [i8; 8],
    pub iemRevision: u32,
    pub aslCompilerID: [i8; 4],
    pub aslCompilerRevision: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Rsdt {
    pub hdr: Header,
    TableOffsetEntry: [u32; 0],
}

impl Rsdt {
    pub fn entry_count(&self) -> usize {
        (self.hdr.length as usize - size_of::<Self>()) / 4
    }
    pub fn entry_at(&self, id: usize) -> u32 {
        assert!(id < self.entry_count());
        unsafe {
            let p = (self as *const Self).offset(1) as *const u32;
            *(p.offset(id as isize))
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Madt {
    pub hdr: Header,
    pub lapicAddr: u32,
    pub flags: u32,
    Table: [u32; 0],
}

impl Checkable for Madt {
    fn check(&self) -> bool {
        &self.hdr.signature == b"APIC" && self.hdr.length >= size_of::<Self>() as u32
    }
}

#[derive(Debug)]
pub enum MadtEntry {
    Unknown(MadtEntry_Unknown),
    LocalApic(MadtEntry_LocalApic),
    IoApic(MadtEntry_IoApic),
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MadtEntry_Unknown {
    pub typ: u8,
    pub length: u8,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MadtEntry_LocalApic {
    pub typ: u8,
    pub length: u8,
    pub procID: u8,
    pub id: u8,
    pub lapicFlags: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MadtEntry_IoApic {
    pub typ: u8,
    pub length: u8,
    pub procID: u8,
    pub id: u8,
    pub lapicFlags: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MadtEntryIter<'a> {
    madt: &'a Madt,
    ptr: *const u8,
    end_ptr: *const u8,
}

impl Madt {
    pub fn entry_iter(&self) -> MadtEntryIter {
        let ptr = unsafe{ (self as *const Self).offset(1) } as *const u8;
        let end_ptr = unsafe{ ptr.offset(self.hdr.length as isize) };
        MadtEntryIter { madt: self, ptr, end_ptr }
    }
}

impl<'a> Iterator for MadtEntryIter<'a> {
    type Item = MadtEntry;
    fn next(&mut self) ->Option<Self::Item> {
        if self.ptr >= self.end_ptr {
            return None;
        }
        unsafe {
            let typeID = *self.ptr.offset(0);
            let length = *self.ptr.offset(1);
            let ret = Some(match typeID {
                0 => MadtEntry::LocalApic( (&*(self.ptr as *const MadtEntry_LocalApic)).clone() ),
                1 => MadtEntry::IoApic( (&*(self.ptr as *const MadtEntry_IoApic)).clone() ),
                _ => MadtEntry::Unknown( (&*(self.ptr as *const MadtEntry_Unknown)).clone() ),
            });
            self.ptr = self.ptr.offset(length as isize);
            ret
        }
    }
}