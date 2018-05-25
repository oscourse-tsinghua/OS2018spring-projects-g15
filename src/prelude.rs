// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Core/prelude.rs
/// Common global definitons
///
/// All files have 'use prelude::*' as the first line, which imports the names from this module
// Recreate std::prelude
pub use core::prelude::*;

pub use alloc::boxed::Box;
pub use mylib::borrow::ToOwned;
pub use alloc::vec::Vec;
pub use alloc::string::String;


// - Not in core::prelude, but I like them
pub use core::any::Any;

pub use mylib::collections::{MutableSeq};
//pub use logging::HexDump;

// vim: ft=rust
