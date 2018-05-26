use memory::PAddr;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub(super) number: usize,
}

impl Frame {
    pub fn containing_address(address: usize) -> Frame {
        Frame{ number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> PAddr {
        PAddr((self.number * PAGE_SIZE) as u64)
    }

    pub fn clone(&self) -> Frame {
        Frame { number: self.number }
    }
    
    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }
}

pub trait FrameAllocator {
    fn set_noncore(&mut self, noncore: bool);
    fn used_frames(& self) -> usize;
    fn free_frames(& self) -> usize;
    fn allocate_frames(&mut self, count: usize) -> Option<Frame>;
    fn deallocate_frames(&mut self, frame: Frame, count: usize);
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}