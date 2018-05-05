use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use memory::FrameAllocator;
use arch::paging::entry::*;
use arch::paging::entry::EntryFlags;
use arch::paging::ENTRY_COUNT;

pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

pub const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;

pub trait TableLevel {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

impl<L> Index<usize> for Table<L> where L: TableLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

impl<L> Table<L> where L: TableLevel {
    pub fn is_unused(&self) -> bool {
        if self.entry_count() > 0 {
            return false;
        }

        true
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_zero();
        }
    }

    /// Set number of entries in first table entry
    fn set_entry_count(&mut self, count: u64) {
        debug_assert!(count <= ENTRY_COUNT as u64, "count can't be greater than ENTRY_COUNT");
        self.entries[0].set_counter_bits(count);
    }

    /// Get number of entries in first table entry
    fn entry_count(&self) -> u64 {
        self.entries[0].counter_bits()
    }

    pub fn increment_entry_count(&mut self) {
        let current_count = self.entry_count();
        self.set_entry_count(current_count + 1);
    }

    pub fn decrement_entry_count(&mut self) {
        let current_count = self.entry_count();
        self.set_entry_count(current_count - 1);
    }
}

impl<L> Table<L> where L: HierarchicalLevel {
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE) {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table<'a>(&'a self, index: usize) -> Option<&'a Table<L::NextLevel> > {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    pub fn next_table_mut<'a>(&'a mut self, index: usize)
        -> Option<&'a mut Table<L::NextLevel> > 
    {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    pub fn next_table_create(&mut self,
                                index: usize)
        -> &mut Table<L::NextLevel>
    {
        if self.next_table(index).is_none() {
            assert!(!self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                    "mapping code does not support huge pages");
            
            use memory::allocate_frames;
            let frame = allocate_frames(1).expect("no frames available");
            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }
}


use core::fmt;
use core::fmt::Debug;

impl Debug for Table<Level4> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Ignore the 511th recursive entry
        let entries = self.entries.iter().enumerate().filter(|&(i, e)| !e.is_unused() && i != 511usize);
        for (i, e) in entries {
            write!(f, "{:3X}: {:?}\n", i, e)?;
            write!(f, "{:?}", self.next_table(i).unwrap())?;
        }
        Ok(())
    }
}

impl Debug for Table<Level3> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let entries = self.entries.iter().enumerate().filter(|&(i, e)| !e.is_unused());
        for (i, e) in entries {
            write!(f, "  {:3X}: {:?}\n", i, e)?;
            write!(f, "{:?}", self.next_table(i).unwrap())?;
        }
        Ok(())
    }
}

impl Debug for Table<Level2> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let entries = self.entries.iter().enumerate().filter(|&(i, e)| !e.is_unused());
        for (i, e) in entries {
            write!(f, "    {:3X}: {:?}\n", i, e)?;
            write!(f, "{:?}", self.next_table(i).unwrap())?;
        }
        Ok(())
    }
}

impl Debug for Table<Level1> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let entries = self.entries.iter().enumerate().filter(|&(i, e)| !e.is_unused());
        for (i, e) in entries {
            write!(f, "      {:3X}: {:?}\n", i, e)?;
        }
        Ok(())
    }
}