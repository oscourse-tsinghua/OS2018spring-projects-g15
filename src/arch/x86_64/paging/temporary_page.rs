use super::Page;
use super::{ActivePageTable, VirtualAddress};
use memory::Frame;
use memory::FrameAllocator;
use super::table::{Table, Level1};

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

struct TinyAllocator([Option<Frame>; 3]);

impl TemporaryPage {
    pub fn new(page: Page) -> TemporaryPage
    {
        TemporaryPage {
            page: page,
            allocator: TinyAllocator::new(),
        }
    }

    pub fn start_address (&self) -> VirtualAddress {
        self.page.start_address()
    }

    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable)
        -> VirtualAddress
    {
        use super::entry::EntryFlags;

        assert!(active_table.translate_page(self.page).is_none(),
                "temporary page is already mapped");
        let result = active_table.map_to(self.page, frame, EntryFlags::WRITABLE);
        result.flush(active_table);
        self.page.start_address()
    }

    /// Unmaps the temporary page in the active table.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        let (result, _frame) = active_table.unmap_return(self.page, true);
        result.flush(active_table);
    }

    /// Maps the temporary page to the given page table frame in the active
    /// table. Returns a reference to the now mapped table.
    pub fn map_table_frame(&mut self,
                        frame: Frame,
                        active_table: &mut ActivePageTable)
                        -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }
}

impl FrameAllocator for TinyAllocator {
    #![allow(unused)]
    fn set_noncore(&mut self, noncore: bool) {}
    fn used_frames(&self) -> usize { 0 }
    fn free_frames(&self) -> usize { 0 }

    fn allocate_frames(&mut self, count: usize) -> Option<Frame> {
        assert!(count == 1);
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn deallocate_frames(&mut self, frame: Frame, count: usize) {
        assert!(count == 1);
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("Tiny allocator can hold only 3 frames.");
    }
}

impl TinyAllocator {
    fn new() -> TinyAllocator
    {
        use memory::allocate_frames;
        let f = || allocate_frames(1);
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}