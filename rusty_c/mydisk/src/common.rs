#![feature(alloc)]

extern crate alloc;

use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};
use core::include;
use alloc::boxed::Box;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

struct EgosAllocator;

// use egos allocator or another crates.io impl, then Box, since other C code
// relies on pointers to heap data we cannot use heapless or just the stack
unsafe impl GlobalAlloc for EgosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size() as cty::size_t) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        free(ptr as *mut cty::c_void);
    }
}

#[global_allocator]
static A: EgosAllocator = EgosAllocator;

// TODO impl core::fmt::write::write_str to use write!() macro or use the core::io version

// pub type Block = block_t;
pub struct Block {
  pub bytes: [u8; BLOCK_SIZE as usize]
}

#[derive(Debug)]
pub enum Error {
  UnknownFailure
}

pub trait Stackable {
  fn get_size(&self) -> 
    Result<u32, Error>;
  fn set_size(&mut self, size: u32) -> 
    Result<i32, Error>;
  fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> 
    Result<i32, Error>;
  fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> 
    Result<i32, Error>;
}

// Structs implementing this trait are the disk itself
pub trait IsDisk {}
