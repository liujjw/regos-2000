#![cfg_attr(not(unix), feature(alloc))]

extern crate alloc;

use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::include;
use core::panic::PanicInfo;
use linked_list_allocator::Heap;
use spin::Mutex;  

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg_attr(not(unix), panic_handler)]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// defined in .lds file, linker MUST resolve before runtime
extern "C" {
    pub static mut __heap_start: u8;
    pub static mut __heap_end: u8;
}

// C Code relies on pointers to heap data we cannot use heapless or just the stack
// TODO prefer static var and LockedHeap, but rsicv32i/rustc has issues with the needed atomics
#[cfg_attr(not(unix), global_allocator)]
pub static ALLOCATOR: Mutex<Heap> = Mutex::new(Heap::empty());

// TODO impl core::fmt::write::write_str to use write!() macro or use the core::io version

// pub type Block = block_t;
pub struct Block {
    pub bytes: [cty::c_char; BLOCK_SIZE as usize],
}

#[derive(Debug)]
pub enum Error {
    UnknownFailure,
}

pub trait Stackable {
    fn get_size(&self) -> Result<u32, Error>;
    fn set_size(&mut self, size: u32) -> Result<i32, Error>;
    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error>;
    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error>;
}

// Structs implementing this trait are the disk itself
pub trait IsDisk {}
