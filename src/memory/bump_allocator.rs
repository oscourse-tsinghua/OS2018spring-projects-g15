use super::{Frame, FrameAllocator, MemoryArea, MemoryAreaIter};


/// A simple allocator that allocates memory linearly and ignores freed memory.
#[derive(Debug)]
pub struct BumpAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel_start: Frame,
    kernel_end: Frame
}

impl BumpAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, memory_areas: MemoryAreaIter) -> Self {
        let mut allocator = Self {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end)
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.start_address() + area.size() - 1;
            Frame::containing_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for BumpAllocator {
    #![allow(unused)]
    fn set_noncore(&mut self, noncore: bool) {}

    fn free_frames(&self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            let end_frame = Frame::containing_address((area.start_address() + area.size() - 1) as usize);
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

    fn used_frames(&self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(area.start_address() as usize);
            let end_frame = Frame::containing_address((area.start_address() + area.size() - 1) as usize);
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

    fn allocate_frames(&mut self, count: usize) -> Option<Frame> {
        if count == 0 {
            None
        } else if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let start_frame = Frame{ number: self.next_free_frame.number };
            let end_frame = Frame { number: self.next_free_frame.number + (count - 1) };

            // the last frame of the current area
            let current_area_last_frame = {
                let address = area.start_address() + area.size() - 1;
                Frame::containing_address(address as usize)
            };

            if end_frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if (start_frame >= self.kernel_start && start_frame <= self.kernel_end)
                    || (end_frame >= self.kernel_start && end_frame <= self.kernel_end) {
                // `frame` is used by the kernel
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += count;
                return Some(start_frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frames(count)
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frames(&mut self, _frame: Frame, _count: usize) {
        //panic!("BumpAllocator::deallocate_frame: not supported: {:?}", frame);
    }
}

// unsafe impl<'a> Alloc for &'a BumpAllocator {
//     unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
//         loop {
//             // load current state of the `next` field
//             let current_next = self.next.load(Ordering::Relaxed);
//             let alloc_start = align_up(current_next, layout.align());
//             let alloc_end = alloc_start.saturating_add(layout.size());

//             if alloc_end <= self.heap_end {
//                 // update the `next` pointer if it still has the value `current_next`
//                 let next_now = self.next.compare_and_swap(current_next, alloc_end,
//                     Ordering::Relaxed);
//                 if next_now == current_next {
//                     // next address was successfully updated, allocation succeeded
//                     return Ok(alloc_start as *mut u8);
//                 }
//             } else {
//                 return Err(AllocErr::Exhausted{ request: layout })
//             }
//         }
//     }

//     unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
//         // do nothing, leak memory
//     }
// }

// /// Align downwards. Returns the greatest x with alignment `align`
// /// so that x <= addr. The alignment must be a power of 2.
// pub fn align_down(addr: usize, align: usize) -> usize {
//     if align.is_power_of_two() {
//         addr & !(align - 1)
//     } else if align == 0 {
//         addr
//     } else {
//         panic!("`align` must be a power of 2");
//     }
// }

// /// Align upwards. Returns the smallest x with alignment `align`
// /// so that x >= addr. The alignment must be a power of 2.
// pub fn align_up(addr: usize, align: usize) -> usize {
//     align_down(addr + align - 1, align)
// }

// // struct LockedBumpAllocator(Mutex<BumpAllocator>);

// // impl<'a> Alloc for &'a LockedBumpAllocator {
// //     unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
// //         self.0.lock().alloc(layout)
// //     }

// //     unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
// //         self.0.lock().dealloc(ptr, layout)
// //     }
// // }