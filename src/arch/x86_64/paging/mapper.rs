use super::{Page, ENTRY_COUNT, EntryFlags};
use super::table::{self, Table, Level4};
use memory::*;
use core::ptr::Unique;

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PAddr> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| PAddr((frame.start_address().get() + offset) as u64))
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.start_address().get() % (ENTRY_COUNT * ENTRY_COUNT * PAGE_SIZE) == 0);
                        return Some(Frame::containing_address(
                            start_frame.start_address().get() + 
                            (page.p2_index() * ENTRY_COUNT + page.p1_index()) * PAGE_SIZE
                        ));
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert!(start_frame.start_address().get() % ENTRY_COUNT == 0);
                            return Some(Frame::containing_address(
                                start_frame.start_address().get() + page.p1_index() * PAGE_SIZE
                            ));
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].pointed_frame())
        .or_else(huge_page)
    }

    pub fn map_to(&mut self, page: Page, frame: Frame, flags: EntryFlags) -> MapperFlush
    {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index());
        let p2 = p3.next_table_create(page.p3_index());
        let p1 = p2.next_table_create(page.p2_index());

        assert!(p1[page.p1_index()].is_unused(),
            "{:X}: Set to {:X}: {:?}, requesting {:X}: {:?}",
            page.start_address(),
            p1[page.p1_index()].address(), p1[page.p1_index()].flags(),
            frame.start_address().get(), flags);
        p1.increment_entry_count();
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
        MapperFlush::new(page)
    }

    pub fn map(&mut self, page: Page, flags: EntryFlags) -> MapperFlush
    {
        use memory::allocate_frames;
        let frame = allocate_frames(1).expect("out of frames");
        self.map_to(page, frame, flags)
    }

    /// Update flags for a page
    pub fn remap(&mut self, page: Page, flags: EntryFlags) -> MapperFlush {
        let p3 = self.p4_mut().next_table_mut(page.p4_index()).expect("failed to remap: no p3");
        let p2 = p3.next_table_mut(page.p3_index()).expect("failed to remap: no p2");
        let p1 = p2.next_table_mut(page.p2_index()).expect("failed to remap: no p1");
        let frame = p1[page.p1_index()].pointed_frame().expect("failed to remap: not mapped");
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
        MapperFlush::new(page)
    }

    pub fn identity_map(&mut self, frame: Frame, flags: EntryFlags) -> MapperFlush
    {
        let page = Page::containing_address(frame.start_address().to_identity_virtual());
        self.map_to(page, frame, flags)
    }

    pub(super) fn entry_mut(&mut self, page: Page) -> &mut Entry {
        use core::ops::IndexMut;
        let p4 = self.p4_mut();
        let mut p3 = p4.next_table_create(page.p4_index());
        let mut p2 = p3.next_table_create(page.p3_index());
        let mut p1 = p2.next_table_create(page.p2_index());
        p1.index_mut(page.p1_index())
    }

    pub fn map_to2(&mut self, page: Page, frame: Frame, flags: EntryFlags) {
        let entry = self.entry_mut(page);
        assert!(entry.is_unused());
        entry.set(frame, flags | EntryFlags::PRESENT);
    }

    pub fn identity_map2(&mut self, frame: Frame, flags: EntryFlags)
    {
        let page = Page::containing_address(frame.start_address().to_identity_virtual());
        self.map_to2(page, frame, flags)
    }

    fn unmap_inner(&mut self, page: &Page, keep_parents: bool) -> Frame {
        let frame;

        let p4 = self.p4_mut();
        if let Some(p3) = p4.next_table_mut(page.p4_index()) {
            if let Some(p2) = p3.next_table_mut(page.p3_index()) {
                if let Some(p1) = p2.next_table_mut(page.p2_index()) {
                    frame = if let Some(frame) = p1[page.p1_index()].pointed_frame() {
                        frame
                    } else {
                        panic!("unmap_inner({:X}): frame not found", page.start_address())
                    };

                    p1.decrement_entry_count();
                    p1[page.p1_index()].set_unused();

                    if keep_parents || ! p1.is_unused() {
                        return frame;
                    }
                } else {
                    panic!("unmap_inner({:X}): p1 not found", page.start_address());
                }

                if let Some(p1_frame) = p2[page.p2_index()].pointed_frame() {
                    //println!("Free p1 {:?}", p1_frame);
                    p2.decrement_entry_count();
                    p2[page.p2_index()].set_unused();
                    deallocate_frames(p1_frame, 1);
                } else {
                    panic!("unmap_inner({:X}): p1_frame not found", page.start_address());
                }

                if ! p2.is_unused() {
                    return frame;
                }
            } else {
                panic!("unmap_inner({:X}): p2 not found", page.start_address());
            }

            if let Some(p2_frame) = p3[page.p3_index()].pointed_frame() {
                //println!("Free p2 {:?}", p2_frame);
                p3.decrement_entry_count();
                p3[page.p3_index()].set_unused();
                deallocate_frames(p2_frame, 1);
            } else {
                panic!("unmap_inner({:X}): p2_frame not found", page.start_address());
            }

            if ! p3.is_unused() {
                return frame;
            }
        } else {
            panic!("unmap_inner({:X}): p3 not found", page.start_address());
        }

        if let Some(p3_frame) = p4[page.p4_index()].pointed_frame() {
            //println!("Free p3 {:?}", p3_frame);
            p4.decrement_entry_count();
            p4[page.p4_index()].set_unused();
            deallocate_frames(p3_frame, 1);
        } else {
            panic!("unmap_inner({:X}): p3_frame not found", page.start_address());
        }

        frame
    }

    /// Unmap a page
    pub fn unmap(&mut self, page: Page) -> MapperFlush {
        let frame = self.unmap_inner(&page, false);
        deallocate_frames(frame, 1);
        MapperFlush::new(page)
    }

    /// Unmap a page, return frame without free
    pub fn unmap_return(&mut self, page: Page, keep_parents: bool) -> (MapperFlush, Frame) {
        let frame = self.unmap_inner(&page, keep_parents);
        (MapperFlush::new(page), frame)
    }
}

use core::fmt;
use core::fmt::Debug;

impl Debug for Mapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.p4())
    }
}


use core::mem;

/// In order to enforce correct paging operations in the kernel, these types
/// are returned on any mapping operation to get the code involved to specify
/// how it intends to flush changes to a page table
#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct MapperFlush(Page);

impl MapperFlush {
    /// Create a new page flush promise
    pub fn new(page: Page) -> MapperFlush {
        MapperFlush(page)
    }

    /// Flush this page in the active table
    pub fn flush(self, table: &mut ActivePageTable) {
        table.flush(self.0);
        mem::forget(self);
    }

    /// Ignore the flush. This is unsafe, and a reason should be provided for use
    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}

/// A flush cannot be dropped, it must be consumed
impl Drop for MapperFlush {
    fn drop(&mut self) {
        panic!("Mapper flush was not utilized");
    }
}

/// To allow for combining multiple flushes into one, we have a way of flushing
/// the active table, which can consume `MapperFlush` structs
#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct MapperFlushAll(bool);

impl MapperFlushAll {
    /// Create a new promise to flush all mappings
    pub fn new() -> MapperFlushAll {
        MapperFlushAll(false)
    }

    /// Consume a single page flush
    pub fn consume(&mut self, flush: MapperFlush) {
        self.0 = true;
        mem::forget(flush);
    }

    /// Flush the active page table
    pub fn flush(self, table: &mut ActivePageTable) {
        if self.0 {
            table.flush_all();
        }
        mem::forget(self);
    }

    /// Ignore the flush. This is unsafe, and a reason should be provided for use
    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}

/// A flush cannot be dropped, it must be consumed
impl Drop for MapperFlushAll {
    fn drop(&mut self) {
        panic!("Mapper flush all was not utilized");
    }
}