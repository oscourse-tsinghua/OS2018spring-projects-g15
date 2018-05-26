use alloc::vec::Vec;
use super::*;
use core::fmt::{Debug, Formatter, Error};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct MemoryArea {
    start_addr: VirtualAddress,
    end_addr: VirtualAddress,
    phys_start_addr: Option<PAddr>,
    flags: u64,
    name: &'static str,
}

impl MemoryArea {
    pub fn new(start_addr: VirtualAddress, end_addr: VirtualAddress, flags: EntryFlags, name: &'static str) -> Self {
        assert!(start_addr <= end_addr, "invalid memory area");
        MemoryArea {
            start_addr,
            end_addr,
            phys_start_addr: None,
            flags: flags.bits(),
            name,
        }
    }
    pub fn new_identity(start_addr: VirtualAddress, end_addr: VirtualAddress, flags: EntryFlags, name: &'static str) -> Self {
        assert!(start_addr <= end_addr, "invalid memory area");
        MemoryArea {
            start_addr,
            end_addr,
            phys_start_addr: Some(PAddr(start_addr as u64)),
            flags: flags.bits(),
            name,
        }
    }
    pub fn new_kernel(start_addr: VirtualAddress, end_addr: VirtualAddress, flags: EntryFlags, name: &'static str) -> Self {
        assert!(start_addr <= end_addr, "invalid memory area");
        MemoryArea {
            start_addr,
            end_addr,
            phys_start_addr: Some(PAddr::from_kernel_virtual(start_addr)),
            flags: flags.bits(),
            name,
        }
    }
    pub unsafe fn as_slice(&self) -> &[u8] {
        use core::slice;
        slice::from_raw_parts(self.start_addr as *const u8, self.end_addr - self.start_addr)
    }
    pub unsafe fn as_slice_mut(&self) -> &mut [u8] {
        use core::slice;
        slice::from_raw_parts_mut(self.start_addr as *mut u8, self.end_addr - self.start_addr)
    }
    pub fn contains(&self, addr: VirtualAddress) -> bool {
        addr >= self.start_addr && addr < self.end_addr
    }
    fn is_overlap_with(&self, other: &MemoryArea) -> bool {
        let p0 = Page::containing_address(self.start_addr);
        let p1 = Page::containing_address(self.end_addr - 1) + 1;
        let p2 = Page::containing_address(other.start_addr);
        let p3 = Page::containing_address(other.end_addr - 1) + 1;
        !(p1 <= p2 || p0 >= p3)
    }
}

/// 内存空间集合，包含若干段连续空间
/// 对应ucore中 `mm_struct`
#[derive(Clone)]
pub struct MemorySet {
    areas: Vec<MemoryArea>,
//    page_table: Option<InactivePageTable>,
}

impl MemorySet {
    pub fn new() -> Self {
        MemorySet {
            areas: Vec::<MemoryArea>::new(),
//            page_table: None,
        }
    }
    /// Used for remap_kernel() where heap alloc is unavailable
    pub unsafe fn new_from_raw_space(slice: &mut [u8]) -> Self {
        use core::mem::size_of;
        let cap = slice.len() / size_of::<MemoryArea>();
        MemorySet {
            areas: Vec::<MemoryArea>::from_raw_parts(slice.as_ptr() as *mut MemoryArea, 0, cap),
//            page_table: None,
        }
    }
    pub fn find_area(&self, addr: VirtualAddress) -> Option<&MemoryArea> {
        self.areas.iter().find(|area| area.contains(addr))
    }
    pub fn push(&mut self, area: MemoryArea) {
        assert!(self.areas.iter()
            .find(|other| area.is_overlap_with(other))
                    .is_none(), "memory area overlap");
        self.areas.push(area);
    }
    pub fn map(&self, pt: &mut Mapper) {
        for area in self.areas.iter() {
            match area.phys_start_addr {
                Some(phys_start) => {
                    for page in Page::range_of(area.start_addr, area.end_addr) {
                        let frame = Frame::containing_address(phys_start.get() + page.start_address() - area.start_addr);
                        let res=pt.map_to(page, frame.clone(), EntryFlags::from_bits(area.flags.into()).unwrap());
                        unsafe{
                            res.ignore();
                        }
                    }
                },
                None => {
                    for page in Page::range_of(area.start_addr, area.end_addr) {
                        let res=pt.map(page, EntryFlags::from_bits(area.flags.into()).unwrap());
                        unsafe{
                            res.ignore();
                        }
                    }
                },
            }
        }
    }
    pub fn unmap(&self, pt: &mut Mapper) {
        for area in self.areas.iter() {
            for page in Page::range_of(area.start_addr, area.end_addr) {
                pt.unmap(page);
            }
        }
    }
    pub fn iter(&self) -> impl Iterator<Item=&MemoryArea> {
        self.areas.iter()
    }
}

impl Debug for MemorySet {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_list()
            .entries(self.areas.iter())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push_and_find() {
        let mut ms = MemorySet::new();
        ms.push(MemoryArea {
            start_addr: 0x0,
            end_addr: 0x8,
            flags: 0x0,
            name: "code",
        });
        ms.push(MemoryArea {
            start_addr: 0x8,
            end_addr: 0x10,
            flags: 0x1,
            name: "data",
        });
        assert_eq!(ms.find_area(0x6).unwrap().name, "code");
        assert_eq!(ms.find_area(0x11), None);
    }

    #[test]
    #[should_panic]
    fn push_overlap() {
        let mut ms = MemorySet::new();
        ms.push(MemoryArea {
            start_addr: 0x0,
            end_addr: 0x8,
            flags: 0x0,
            name: "code",
        });
        ms.push(MemoryArea {
            start_addr: 0x4,
            end_addr: 0x10,
            flags: 0x1,
            name: "data",
        });
    }
}
