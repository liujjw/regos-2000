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
    // an i8 or u8 depending on platform
    pub bytes: [cty::c_char; BLOCK_SIZE as usize],
}

#[derive(Debug)]
pub enum Error {
    UnknownFailure,
    InodeOutOfBounds,
    DiskTooSmall,
    OutOfSpace,
    ReadOffsetTooLarge,
    UnitializedInode,
    UnterminatedInode,
    UnknownCase
}

/// Interface of every virtual layer in a filesystem.Functionality is implementation specific.
pub trait Stackable {
    fn get_size(&self, ino: u32) -> Result<u32, Error>;
    fn set_size(&mut self, ino: u32, size: u32) -> Result<i32, Error>;
    // &mut is safer for compatiblity with C, since below will call read and needs a *mut
    // however, assume read does not mutate 
    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error>;
    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error>;
}

/// Structs implementing this trait are the disk itself
pub trait IsDisk {}

/// Wrapper over a physical filesystem block.
impl Block {
    pub const BLOCK_SIZE: usize = BLOCK_SIZE as usize;

    pub fn new() -> Block {
        Block {
            bytes: [0; Self::BLOCK_SIZE],
        }
    }

    /// Read all the bytes from the block.
    pub fn read_bytes<'a>(&'a self) -> &'a [cty::c_char] {
        &self.bytes
    }

    /// Write bytes to block, where beg and end is the index range to write to in the block.
    pub fn write_bytes<'a>(&'a mut self, src: &[cty::c_char], beg: usize, end: usize) {
        if src.len() > Self::BLOCK_SIZE || end - beg != src.len() {
            panic!("src improperly sized")
        }
        let mut byte_slice = &mut self.bytes[beg..end];
        byte_slice.copy_from_slice(src);
    }

    /// Wrapper methods to go from/to C.
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
        unsafe { &mut *(block as *mut _ as *mut Block) }
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

/// Wrapper of disk layer, with standard c pointers to functions.
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
    ds_get_size: unsafe extern "C" fn(this_bs: *mut inode_store, ino: cty::c_uint) -> cty::c_int,
    ds_set_size: unsafe extern "C" fn(
        this_bs: *mut inode_store,
        ino: cty::c_uint,
        newsize: block_no,
    ) -> cty::c_int,
}

impl IsDisk for DiskFS {}

/// Wrapper methods to and from Rust/C data types.
impl DiskFS {
    pub fn take_into_(self) -> inode_intf {
        return self._og;
    }

    pub fn share_into_(&mut self) -> inode_intf {
        return self._og;
    }

    /// assume whoever receives the mutable pointer does not really 
    /// mutate the data (i.e. reads)
    pub fn unchecked_share_into(&self) -> inode_intf {
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
    /// Returns total number of blocks on disk. Inodes are not a concept on disk layer.
    fn get_size(&self, ino: u32) -> Result<u32, Error> {
        // make up dummy arguments
        // https://stackoverflow.com/questions/36005527/why-can-functions-with-no-arguments-defined-be-called-with-any-number-of-argumen
        unsafe { Ok((self.ds_get_size)(core::ptr::null_mut(), 0) as u32) }
    }

    /// No-op.
    fn set_size(&mut self, ino: u32, size: u32) -> Result<i32, Error> {
        // make up dummy arguments, the real diskfs.c has no arguments
        unsafe {
            match (self.ds_set_size)(core::ptr::null_mut(), 0, 0) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x),
            }
        }
    }

    /// Read the block specified by offset, ino is unused.
    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        unsafe {
            match (self.ds_read)(self.unchecked_share_into(), ino, offset, buf.share_into_()) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x),
            }
        }
    }

    /// Write the block specified by offset, ino is unused.
    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        unsafe {
            match (self.ds_write)(self.share_into_(), ino, offset, buf.copy_into_()) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x),
            }
        }
    }
}

pub fn unix_fix_u8_to_i8(bytes: &[u8]) -> [i8; 4] {
    // fix for &[i8] instead of &[u8]
    let mut sbytes: [i8; 4] = [0; 4];
    for (idx, &byte) in bytes.iter().enumerate() {
        sbytes[idx] = byte as i8;
    }
    sbytes
}

pub fn unix_fix_i8_to_u8(bytes: &[i8]) -> [u8; 4] {
    // fix for &[i8] instead of &[u8]
    let mut sbytes: [u8; 4] = [0; 4];
    for (idx, &byte) in bytes.iter().enumerate() {
        sbytes[idx] = byte as u8;
    }
    sbytes
}

pub fn unix_fix_i8_to_u8_full<'a>(i8slice: &'a [i8]) -> &'a [u8] {
    unsafe { 
        &*(i8slice as *const [i8] as *const [u8]) 
    }
}

pub fn unix_fix_u8_to_i8_full<'a>(u8slice: &'a [u8]) -> &'a [i8] {
    unsafe { 
        &*(u8slice as *const [u8] as *const [i8]) 
    }
}