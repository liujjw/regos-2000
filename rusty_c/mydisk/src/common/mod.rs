#![cfg_attr(not(unix), feature(alloc))]

extern crate alloc;

use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::include;
use core::panic::PanicInfo;

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use crate::bindings::*;

#[cfg_attr(not(unix), panic_handler)]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

struct EgosAllocator;

extern "C" {
    fn malloc(size: cty::size_t) -> *mut cty::c_void;
    fn free(ptr: *mut cty::c_void);
}

unsafe impl GlobalAlloc for EgosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size() as cty::size_t) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        free(ptr as *mut cty::c_void);
    }
}

#[cfg_attr(not(unix), global_allocator)]
static A: EgosAllocator = EgosAllocator;

// TODO impl core::fmt::write::write_str to use write!() macro or use the core::io version

// pub type Block = block_t;
#[cfg_attr(unix, derive(Debug))]
#[repr(C)]
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
    // &mut for compatiblity with C, since below will call read and needs a *mut 
    fn read(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error>;
    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error>;
}

// Structs implementing this trait are the disk itself
pub trait IsDisk {}

impl Block {
    pub const BLOCK_SIZE: usize = BLOCK_SIZE as usize;

    pub fn new() -> Block {
        Block {
            bytes: [0; Self::BLOCK_SIZE],
        }
    }

    pub fn read_bytes<'a>(&'a self) -> &'a [cty::c_char] {
        &self.bytes
    }

    pub fn write_bytes<'a>(&'a mut self, src: &[cty::c_char], beg: usize, end: usize) {
        if src.len() > Self::BLOCK_SIZE || end - beg != src.len() {
            panic!("src improperly sized")
        }
        let mut byte_slice = &mut self.bytes[beg..end];
        byte_slice.copy_from_slice(src);
    }

    // TODO lock
    pub fn copy_from_(block: *mut block_t) -> Self {
        unsafe {
            Block {
                bytes: (*block).bytes,
            }
        }
    }
    
    // TODO lock
    // TODO better lifetime bound
    pub fn share_from_(block: *mut block_t) -> &'static mut Self {
        unsafe {
            &mut *(block as *mut _ as *mut Block)
        }
    }

    pub fn copy_into_(&self) -> *mut block_t {
        let mut bytes_copy: [cty::c_char; Self::BLOCK_SIZE] = [0; Self::BLOCK_SIZE];
        (&mut bytes_copy).copy_from_slice(self.read_bytes());
        let new_block: *mut block_t = &mut block_t { bytes: bytes_copy };
        return new_block;
    }

    pub fn share_into_(&mut self) -> *mut block_t {
        self as *mut _ as *mut block_t
    }
}

// standard c pointers to functions
#[cfg_attr(unix, derive(Debug))]
pub struct DiskFS {
    _og: inode_intf,
    ds_read: unsafe extern "C" fn(
        bs: *mut inode_store_t,
        ino: cty::c_uint,
        offset: block_no,
        block: *mut block_t,
    ) -> cty::c_int,
    ds_write: unsafe extern "C" fn(
        bs: *mut inode_store_t,
        ino: cty::c_uint,
        offset: block_no,
        block: *mut block_t,
    ) -> cty::c_int,
    // in real diskfs.c, inconsistent function signatures with no arguments
    ds_get_size: unsafe extern "C" fn(
        this_bs: *mut inode_store, 
        ino: cty::c_uint
    ) -> cty::c_int,
    ds_set_size: unsafe extern "C" fn(
        this_bs: *mut inode_store,
        ino: cty::c_uint,
        newsize: block_no,
    ) -> cty::c_int,
}

impl IsDisk for DiskFS {}

impl DiskFS {
    pub fn take_into_(self) -> inode_intf {
        return self._og;
    }

    pub fn share_into_(&mut self) -> inode_intf {
        return self._og;
    }

    // TODO lock from >1 instantiation
    // TODO lock _og from being freed before take_into
    pub fn from_(inode_store: inode_intf) -> Self {
        DiskFS {
            _og: inode_store,
            ds_read: unsafe { (*inode_store).read.unwrap() },
            ds_write: unsafe { (*inode_store).write.unwrap() },
            ds_get_size: unsafe { (*inode_store).getsize.unwrap() },
            ds_set_size: unsafe { (*inode_store).setsize.unwrap() },
        }
    }
}

impl Stackable for DiskFS {
    pub fn get_size(&self) -> Result<u32, Error> {
        // make up dummy arguments
        // https://stackoverflow.com/questions/36005527/why-can-functions-with-no-arguments-defined-be-called-with-any-number-of-argumen
        unsafe { Ok((self.ds_get_size)(core::ptr::null_mut(), 0) as u32) }
    }

    pub fn set_size(&mut self, size: u32) -> Result<i32, Error> {
        // make up dummy arguments, the real diskfs.c has no arguments
        unsafe {
            match (self.ds_set_size)(core::ptr::null_mut(), 0, 0) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x),
            }
        }
    }

    pub fn read(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        unsafe {
            match (self.ds_read)(self.share_into_(), ino, offset, buf.share_into_()) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x),
            }
        }
    }

    pub fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        unsafe {
            match (self.ds_write)(self.share_into_(), ino, offset, buf.copy_into_()) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x),
            }
        }
    }
}