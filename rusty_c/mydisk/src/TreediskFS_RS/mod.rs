#![cfg_attr(not(unix), no_std)]

extern crate alloc;

use crate::common::*;
use crate::bindings::*;
use core::mem::size_of;
use alloc::boxed::Box;

const SUPERBLOCK_WIDTH: usize = 4;
const INODE_TABLE_ENTRY_WIDTH: usize = 8;
const FAT_TABLE_ENTRY_WIDTH: usize = 4;
struct Superblock {
    fat_head_value: u32,
    disk_block_index: u32, 
}
struct InodeTableEntryData {
    head: u32,
    size: u32,
    inode_number: u32
}
struct InodeTable {
    num_blocks: u32,
    disk_block_index: u32
}
struct FatTableEntry {
    next: u32,
    index: u32
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
    fn calc_inode_table(num_inodes: u32) -> u32 {
        let total_bytes_needed_for_inode_table = num_inodes * INODE_TABLE_ENTRY_WIDTH as u32;
        let num_blocks_needed_for_inode_table = libm::ceil(
            total_bytes_needed_for_inode_table / BLOCK_SIZE
        ) as u32;
        return num_blocks_needed_for_inode_table;
    }

    fn calc_num_blocks_of_disk() -> u32 {
        let num_blocks_of_disk = below.get_size() / BLOCK_SIZE;
        return num_blocks_of_disk;
    }

    fn calc_fat_table(
        num_blocks_of_disk: u32, 
        num_blocks_needed_for_inode_table: u32,
        num_blocks_for_superblock: u32
    ) -> u32 {
        // x = a - b - 1 - y
        // y = (x * 4) / 512
        // solve for y:
        // y = (4a - 4b - 4) / 516

        let num_entries = num_blocks_of_disk - num_blocks_needed_for_inode_table - 1;
        let mut num_blocks_needed_for_fat_table = libm::ceil(
            (num_entries * FAT_TABLE_ENTRY_WIDTH as u32) / BLOCK_SIZE
        ) as u32;
        num_blocks_for_superblock = libm::ceil(
            ((4 * num_blocks_of_disk) - (4 * num_blocks_needed_for_inode_table) - 4) / 516
        ) as u32;
        return num_blocks_needed_for_fat_table;
    }

    /// new calculates info from setup_disk and stores it, assumes setup_disk is called first
    pub fn new(below: T, below_ino: u32, num_inodes: u32) -> Self {

    }

    /// 1. theres a fat table which contains pointers, each pointer is 32bits/4 bytes index
    /// the index of the pointer is the index of the block number 
    /// 2. theres an inode table which is metadata on the fat table, each entry contains a head index and a size
    /// 3. then a superblock which contains one index into the fat table, but that entry in the in the fat table starting out just consecutively points to the next index
    /// setup writes to disk with empty data
    pub fn setup_disk(below: &mut T, below_ino: u32, num_inodes: u32) -> Result<i32, Error> {
        // the superblock is first 4 bytes and first block on the disk
        let mut superblock_data = Block::new();
        superblock_data.write_bytes(&u32::to_le_bytes(0), 0, SUPERBLOCK_WIDTH)?;
        below.write(below_ino, 0, 0, superblock_data)?;

        // inode table starts at second block
        // each entry is 8 bytes wide
        // every 4 bytes is the le representation of -1 starting out
        // write num_blocks_needed_for_inode_table blocks 
        let mut inode_table_data = Block::new();
        for i in 0..BLOCK_SIZE / SUPERBLOCK_WIDTH {
            inode_table_data.write_bytes(&i32::to_le_bytes(-1), i * SUPERBLOCK_WIDTH, (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH)?;
        }
        let num_blocks_needed_for_inode_table = calc_inode_table(num_inodes);
        for block_num in 1..num_blocks_needed_for_inode_table {
            below.write(below_ino, block_num, 0, inode_table_data)?;
        }

        // fat table starts at the block after the inode table
        // each entry is 4 bytes wide, aaps directly to a block
        // every 4 bytes is the le representation of -1 to represent null
        // how many entries needed? depends on number of blocks we have left
        let num_blocks_needed_for_fat_table = calc_fat_table(
            calc_num_blocks_of_disk(), 
            num_blocks_needed_for_inode_table,
            1
        );
        let fat_table_data = inode_table_data;
        for block_num in (1 + num_blocks_needed_for_inode_table)..(1 + num_blocks_needed_for_inode_table + num_blocks_needed_for_fat_table) {
            below.write(below_ino, block_num, 0, fat_table_data)?;
        }
        Ok(0)
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