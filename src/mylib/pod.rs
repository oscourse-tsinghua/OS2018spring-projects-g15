// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Core/mylib/pod.rs
//! Plain-old-data support

/// Plain-old-data trait
use alloc::boxed::Box;
pub unsafe auto trait POD {}
//impl<T: ::core::ops::Drop> !POD for T {}  // - I would love this, but it collides with every other !POD impl
impl<T> !POD for ::core::cell::UnsafeCell<T> {}
impl<T> !POD for ::core::ptr::NonNull<T> {}
impl<T> !POD for Box<T> {}
impl<T> !POD for *const T {}
impl<T> !POD for *mut T {}
impl<'a, T> !POD for &'a T {}
impl<'a, T> !POD for &'a mut T {}

pub fn as_byte_slice<T: ?Sized + POD>(s: &T) -> &[u8] {
	// SAFE: Plain-old-data
	unsafe { ::core::slice::from_raw_parts(s as *const _ as *const u8, ::core::mem::size_of_val(s)) }
}
pub fn as_byte_slice_mut<T: ?Sized + POD>(s: &mut T) -> &mut [u8] {
	// SAFE: Plain-old-data
	unsafe { ::core::slice::from_raw_parts_mut(s as *mut _ as *mut u8, ::core::mem::size_of_val(s)) }
}
