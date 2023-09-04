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
        let num_blocks_of_disk = below.get_size(-1) / BLOCK_SIZE;
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
        let num_blocks_inode_table = calc_inode_table(num_inodes);
        let num_blocks_superblock = 1;
        FS {
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes,
            superblock: Superblock {
                fat_head_value: 0,
                disk_block_index: 0
            },
            inode_table: InodeTable {
                num_blocks: num_blocks_inode_table,
                disk_block_index: num_blocks_for_superblock
            },
            fat_table: FatTable {
                num_blocks: calc_fat_table(
                    calc_num_blocks_of_disk, 
                    num_blocks_inode_table,
                    num_blocks_superblock
                ),
                disk_block_index: num_blocks_for_superblock + num_blocks_inode_table
            }
        }
    }

    /// see https://www.cs.cornell.edu/courses/cs4411/2020sp/schedule/slides/06-Filesystems-FAT.pdf
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
        // each entry is 4 bytes wide, maps directly to a block
        // every 4 bytes is the le representation of -2 to represent pointing to the next block
        // how many entries needed? depends on number of blocks we have left
        let num_blocks_needed_for_fat_table = calc_fat_table(
            calc_num_blocks_of_disk(), 
            num_blocks_needed_for_inode_table,
            1
        );
        let fat_table_data = Block::new();
        for i in 0..BLOCK_SIZE / SUPERBLOCK_WIDTH {
            fat_table_data.write_bytes(&i32::to_le_bytes(-2), i * SUPERBLOCK_WIDTH, (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH)
        }
        let beg = 1 + num_blocks_needed_for_inode_table;
        let end = 1 + num_blocks_needed_for_inode_table + num_blocks_needed_for_fat_table;
        for block_num in beg..end {
            below.write(below_ino, block_num, 0, fat_table_data)?;
        }
        Ok(0)
    }
}

impl<T: Stackable + IsDisk> Stackable for FS<T> {
    /// TODO
    fn calc_inode_table_indices(&self, ino: u32) -> (u32, u32) {
        // if start_byte is 8, block num is 0
        // if start_byte is 512, block num is 1
        let start_byte = ino * INODE_TABLE_ENTRY_WIDTH;
        let block_num = start_byte / BLOCK_SIZE;
        let start_byte_within_block = start_byte % BLOCK_SIZE;
        return (block_num, start_byte_within_block);
    }

    /// Read the ino'th entry of the inode table 
    fn get_size(&self, ino: u32) -> Result<u32, Error> {
        let (block_num, start_byte_within_block) = calc_inode_table_indices(ino);

        let mut block = Block::new();
        self.below.read(-1, self.InodeTable.disk_block_index + block_num, block);

        let bytes = block.read_bytes();
        // if start byte is 513, block num is 1, % 512 is 1
        let beg = start_byte_within_block;
        let end = start_byte_within_block + SUPERBLOCK_WIDTH;
        let head_ = &bytes[beg..end];
        let size_ = &bytes[end..end + SUPERBLOCK_WIDTH];
        let head = i32::from_le_bytes(head_);
        let size = i32::from_le_bytes(size_);
        if head <= -1 || size <= -1 {
            return Error::UnknownFailure();
        }
        return size as u32;
    }

    // TODO what if size is invalid?
    fn set_size(&mut self, ino: u32, size: u32) -> Result<i32, Error> {
        let (block_num, start_byte_within_block) = calc_inode_table_indices(ino);
        
        let mut block = Block::new();
        self.below.read(-1, self.InodeTable.disk_block_index + block_num, block);
        let new_size = u32::to_le_bytes(size);
        // size comes after head
        let beg = start_byte_within_block + SUPERBLOCK_WIDTH;
        let end = beg + SUPERBLOCK_WIDTH;
        block.write_bytes(new_size, beg, end);
        self.below.write(-1, self.InodeTable.disk_block_index + block_num, block);
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
    match FS::from_(inode_store).get_size(ino) {
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
        .set_size(0, newsize as u32)
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