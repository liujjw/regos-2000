#![cfg_attr(not(unix), no_std)]

extern crate alloc;

use crate::common::*;
use crate::bindings::*;
use core::mem::size_of;
use alloc::boxed::Box;

struct Superblock {
    fat_head: u32,
    disk_block_index: u32, 
}
struct InodeTableEntry {
    head: u32,
    size: u32,
    inode_number: u32,
}
struct InodeTable {
    num_blocks: u32,
    disk_block_index: u32,
}
struct FatTableEntry {
    next: u32,
    block_number: u32,
}
struct FatTable {
    num_blocks: u32,
    disk_block_index: u32,
}

struct FS<T: Stackable> {
    below: T,
    below_ino: u32,
    num_inodes: u32,
    superblock: Superblock,
    inode_table: InodeTable,
    fat_table: FatTable,
}

#[repr(C)]
struct FS_C {
    below: *mut inode_store_t,
    below_ino: cty::c_uint,
    num_inodes: cty::c_uint,
}

impl FS<DiskFS> {
  fn from_(inode_store: *mut inode_store_t) -> Self {
      unimplemented!()
  }

  fn take_into_(self) -> *mut inode_store_t {
      unimplemented!()
  }
}

impl<T: Stackable + IsDisk> FS<T> {
  pub fn new(below: T, below_ino: u32, num_inodes: u32) -> Self {
      unimplemented!()
  }

  // theres a fat table which contains pointers, each pointer is 32bits/4 bytes index
  // the index of the pointer is the index of the block number 

  // theres an inode table which is metadata on the fat table, each entry contains a head index 
  // and a size

  // then a superblock which contains one index into the fat table, but that entry in the 
  // in the fat table starting out just consecutively points to the next index
  pub fn setup_disk(below: &mut T, below_ino: u32, num_inodes: u32) -> Result<i32, Error> {
  }
}

impl<T: Stackable + IsDisk> Stackable for FS<T> {
  fn get_size(&self) -> Result<u32, Error> {
      unimplemented!()
  }

  fn set_size(&mut self, size: u32) -> Result<i32, Error> {
      unimplemented!()
  }


  fn read(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
      unimplemented!()
  }

  fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
      unimplemented!()
  }
}

#[no_mangle]
pub unsafe extern "C" fn fs_init(
    below: *mut inode_store_t,
    below_ino: cty::c_uint,
    num_inodes: cty::c_uint,
) -> *mut inode_store_t {
    // assume below is aligned, initialized, and valid, but can check if non-null
    if below.is_null() {
        panic!("below is null");
    }

    let myfs: *mut inode_store_t =
    (FS::new(DiskFS::from_(below), below_ino, num_inodes)).take_into_();

    #[cfg(unix)] 
    {
        let state = (*myfs).state as *mut SimpleFS_C;
        let below = (*state).below;
    }

    return myfs;
}

#[no_mangle]
unsafe extern "C" fn fs_get_size(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
) -> cty::c_int {
    match FS::from_(inode_store).get_size() {
        Ok(val) => val as i32,
        Err(_) => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fs_set_size(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    newsize: block_no,
) -> cty::c_int {
    if newsize < 0 {
        panic!("size must be non-negative");
    }
    FS::from_(inode_store)
        .set_size(newsize as u32)
        .unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn fs_read(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t,
) -> cty::c_int {
    FS::from_(inode_store)
        .read(ino, offset, Block::share_from_(block))
        .unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn fs_write(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t,
) -> cty::c_int {
    FS::from_(inode_store)
        .write(ino, offset, &mut Block::copy_from_(block))
        .unwrap_or(-1)
}

#[no_mangle]
pub unsafe extern "C" fn fs_create(
    below: *mut inode_store_t,
    below_ino: cty::c_uint,
    ninodes: cty::c_uint,
) -> cty::c_int {
    FS::setup_disk(&mut DiskFS::from_(below), below_ino, ninodes).unwrap_or(-1)
}
// in the init step we create the internal data structures we need