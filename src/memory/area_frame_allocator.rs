// This file has been antiquated

use memory::{Frame, FrameAllocator, PhysicalAddress};
use multiboot2::{MemoryAreaIter, MemoryArea};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl FrameAllocator for AreaFrameAllocator {
    fn used_frames(& self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            let end_frame = Frame::containing_address((area.base_addr + area.length - 1) as usize);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // Inside of kernel range
                    count += 1
                } else if frame >= self.next_free_frame {
                    // Frame is in free range
                } else {
                    count += 1;
                }
            }
        }

        count
    }

    fn free_frames(& self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            let end_frame = Frame::containing_address((area.base_addr + area.length - 1) as usize);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // Inside of kernel range
                } else if frame >= self.next_free_frame {
                    // Frame is in free range
                    count += 1;
                } else {
                    // Inside of used range
                }
            }
        }

        count
    }

    fn allocate_frames(&mut self, count: usize) -> Option<Frame> {
        if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let frame = Frame{ number: self.next_free_frame.number };

            // the last frame of the current area
            let current_area_last_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(address as usize)
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by the kernel
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1
                };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += 1;
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frames(1)
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frames(&mut self, _frame: Frame, count: usize) {
        unimplemented!()
    }
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: PhysicalAddress, kernel_end: PhysicalAddress,
        multiboot_start: PhysicalAddress, multiboot_end: PhysicalAddress,
        memory_areas: MemoryAreaIter) -> AreaFrameAllocator
    {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start.0 as usize),
            kernel_end: Frame::containing_address(kernel_end.0 as usize),
            multiboot_start: Frame::containing_address(multiboot_start.0 as usize),
            multiboot_end: Frame::containing_address(multiboot_end.0 as usize),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.base_addr + area.length - 1;
            Frame::containing_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
