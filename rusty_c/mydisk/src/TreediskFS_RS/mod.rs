#![cfg_attr(not(unix), no_std)]

extern crate alloc;

use crate::common::*;
use crate::bindings::*;
use core::mem::size_of;
use alloc::boxed::Box;

const SUPERBLOCK_WIDTH: usize = 4;
const INODE_TABLE_ENTRY_WIDTH: usize = 8;
const FAT_TABLE_ENTRY_WIDTH: usize = 4;
const NEXT_BLOCK: i32 = -2;
const NULL_POINTER: i32 = -1;
/// Snapshot superblock data, contains true disk block offset, and fat head value.
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
    /// Returns the number of blocks needed for the inode table.
    fn calc_inode_table(num_inodes: u32) -> u32 {
        let total_bytes_needed_for_inode_table = num_inodes * INODE_TABLE_ENTRY_WIDTH as u32;
        let num_blocks_needed_for_inode_table = libm::ceil(
            total_bytes_needed_for_inode_table as f64 / BLOCK_SIZE as f64
        ) as u32;
        return num_blocks_needed_for_inode_table;
    }

    /// Returns the number of blocks on the disk.
    fn calc_num_blocks_of_disk(below: &T) -> Result<u32, Error> {
        let num_blocks_of_disk = below.get_size(0)? / BLOCK_SIZE;
        return Ok(num_blocks_of_disk);
    }

    /// Returns the number of blocks needed for the fat table.
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
            (num_entries * FAT_TABLE_ENTRY_WIDTH as u32) as f64 / BLOCK_SIZE as f64
        ) as u32;
        num_blocks_needed_for_fat_table = libm::ceil(
            ((4 * num_blocks_of_disk) - (4 * num_blocks_needed_for_inode_table) - 4) as f64 / 516 as f64
        ) as u32;
        return num_blocks_needed_for_fat_table;
    }

    /// new calculates info from setup_disk and stores it, assumes setup_disk is called first
    pub fn new(below: T, below_ino: u32, num_inodes: u32) -> Self {
        let num_blocks_inode_table = Self::calc_inode_table(num_inodes);
        let num_blocks_superblock = 1;
        let num_blocks_of_disk = match Self::calc_num_blocks_of_disk(&below) {
            Ok(ans) => ans,
            Err(e) => 0
        };
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
                disk_block_index: num_blocks_superblock
            },
            fat_table: FatTable {
                num_blocks: Self::calc_fat_table(
                    num_blocks_of_disk, 
                    num_blocks_inode_table,
                    num_blocks_superblock
                ),
                disk_block_index: num_blocks_superblock + num_blocks_inode_table
            }
        }
    }

    /// see https://www.cs.cornell.edu/courses/cs4411/2020sp/schedule/slides/06-Filesystems-FAT.pdf
    /// setup writes to disk with empty data
    pub fn setup_disk(below: &mut T, below_ino: u32, num_inodes: u32) -> Result<i32, Error> {
        // the superblock is first 4 bytes and first block on the disk
        let mut superblock_data = Block::new();
        #[cfg(unix)] {
            let bytes = unix_fix_u8_to_i8(&(0 as u32).to_le_bytes());
            superblock_data.write_bytes(&bytes, 0, SUPERBLOCK_WIDTH);
        }
        #[cfg(not(unix))] {
            let bytes = (0 as u32).to_le_bytes();
            superblock_data.write_bytes(&bytes, 0, SUPERBLOCK_WIDTH);
        }
        below.write(0, 0, &superblock_data)?;

        // inode table starts at second block
        // each entry is 8 bytes wide
        // every 4 bytes is the le representation of -1 starting out
        // write num_blocks_needed_for_inode_table blocks 
        let mut inode_table_data = Block::new();
        let bytes = i32::to_le_bytes(-1);
        for i in 0..BLOCK_SIZE as usize / SUPERBLOCK_WIDTH as usize {
            #[cfg(unix)] {
                let bytes = unix_fix_u8_to_i8(&bytes);
                inode_table_data.write_bytes(
                    &bytes, 
                    i * SUPERBLOCK_WIDTH, 
                    (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH
                );
            }
            #[cfg(not(unix))] {
                inode_table_data.write_bytes(
                    &bytes, 
                    i * SUPERBLOCK_WIDTH, 
                    (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH
                );
            }
        }
        let num_blocks_needed_for_inode_table = Self::calc_inode_table(num_inodes);
        for block_num in 1..num_blocks_needed_for_inode_table {
            below.write(block_num, 0, &inode_table_data)?;
        }

        // fat table starts at the block after the inode table
        // each entry is 4 bytes wide, maps directly to a block
        // every 4 bytes is the le representation of -2 to represent pointing to the next block
        // -1 means null
        // how many entries needed? depends on number of blocks we have left
        let num_blocks_needed_for_fat_table = Self::calc_fat_table(
            Self::calc_num_blocks_of_disk(&below)?, 
            num_blocks_needed_for_inode_table,
            1
        );
        let mut fat_table_data = Block::new();
        let bytes = i32::to_le_bytes(NEXT_BLOCK);
        for i in 0..BLOCK_SIZE as usize / SUPERBLOCK_WIDTH {
            #[cfg(unix)] {
                let bytes = unix_fix_u8_to_i8(&bytes);
                fat_table_data.write_bytes(&bytes, i * SUPERBLOCK_WIDTH, (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH);
            }
            #[cfg(not(unix))] {
                fat_table_data.write_bytes(&bytes, i * SUPERBLOCK_WIDTH, (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH);
            }
        }
        let beg = 1 + num_blocks_needed_for_inode_table;
        let end = 1 + num_blocks_needed_for_inode_table + num_blocks_needed_for_fat_table;
        for block_num in beg..end {
            below.write(block_num, 0, &fat_table_data)?;
        }
        Ok(0)
    }

    /// Return the block number and offset inside the block within the inode table.
    /// Does not account for the true disk block offset.
    fn calc_inode_table_indices(ino: u32) -> (u32, u32) {
        // if start_byte is 8, block num is 0
        // if start_byte is 512, block num is 1
        let start_byte = ino * INODE_TABLE_ENTRY_WIDTH as u32;
        let inner_block_num = start_byte / BLOCK_SIZE;
        let start_byte_within_block = start_byte % BLOCK_SIZE;
        return (inner_block_num, start_byte_within_block);
    }

    /// Return the block number and offset inside the block within the fat table.
    /// Does not account for the true disk block offset.
    fn calc_fat_table_indices(idx: u32) -> (u32, u32) {
        // if start_byte is 4, block num is 0
        // if start_byte is 512, block num is 1
        let start_byte = idx * FAT_TABLE_ENTRY_WIDTH as u32;
        let inner_block_num = start_byte / BLOCK_SIZE;
        let start_byte_within_block = start_byte % BLOCK_SIZE;
        return (inner_block_num, start_byte_within_block);
    }

    /// Return the next block number index in the fat table for a given index.
    fn get_fat_table_info(&self, idx: u32) -> i32 {
        let (inner_block_num, start_byte_within_block) = Self::calc_fat_table_indices(idx);

        let mut block = Block::new();
        self.below.read(0, self.fat_table.disk_block_index + inner_block_num, &mut block);

        let bytes = block.read_bytes();
        let beg = start_byte_within_block as usize;
        let end = start_byte_within_block as usize + FAT_TABLE_ENTRY_WIDTH;
        let next_ = &bytes[beg..end];
        let next = i32::from_le_bytes(unix_fix_i8_to_u8(next_));
        if next == NEXT_BLOCK {
            return (idx + 1) as i32;
        } else if next == NULL_POINTER {
            return NULL_POINTER;
        } 
        return next;
    }

    fn update_fat_table_info(&mut self, idx: u32, next: u32) {
        let (inner_block_num, start_byte_within_block) = Self::calc_fat_table_indices(idx);

        let mut block = Block::new();
        self.below.read(0, self.fat_table.disk_block_index + inner_block_num, &mut block);

        let next_bytes = u32::to_le_bytes(next);
        let beg = start_byte_within_block as usize;
        let end = start_byte_within_block as usize + FAT_TABLE_ENTRY_WIDTH as usize;
        #[cfg(unix)] {
            let next_bytes = unix_fix_u8_to_i8(&next_bytes);
            block.write_bytes(&next_bytes, beg, end);
        }
        #[cfg(not(unix))] {
            block.write_bytes(&next_bytes, beg, end);
        }
        self.below.write(0, self.fat_table.disk_block_index + inner_block_num, &block);
    }

    /// Return the head and size of the inode.
    fn get_inode_info(&self, ino: u32) -> Result<(u32, u32), Error> {
        let (inner_block_num, start_byte_within_block) = Self::calc_inode_table_indices(ino);

        let mut block = Block::new();
        self.below.read(0, self.inode_table.disk_block_index + inner_block_num, &mut block);

        let bytes = block.read_bytes();
        // if start byte is 513, block num is 1, % 512 is 1
        let beg = start_byte_within_block as usize;
        let end = start_byte_within_block as usize + SUPERBLOCK_WIDTH;
        let head_ = &bytes[beg..end];
        let size_ = &bytes[end..end + SUPERBLOCK_WIDTH];
        let head = i32::from_le_bytes(unix_fix_i8_to_u8(head_));
        let size = i32::from_le_bytes(unix_fix_i8_to_u8(size_));
        if head <= -1 || size <= -1 {
            return Err(Error::UnknownFailure);
        }
        return Ok((head as u32, size as u32));
    }

    fn update_inode_info(&mut self, ino: u32, head: u32, size: u32) {
        let (inner_block_num, start_byte_within_block) = Self::calc_inode_table_indices(ino);

        let mut block = Block::new();
        self.below.read(0, self.inode_table.disk_block_index + inner_block_num, &mut block);

        let head_bytes = u32::to_le_bytes(head);
        let size_bytes = u32::to_le_bytes(size);
        let beg = start_byte_within_block as usize;
        let end = start_byte_within_block as usize + SUPERBLOCK_WIDTH;
        block.write_bytes(&unix_fix_u8_to_i8(&head_bytes), beg, end);
        block.write_bytes(&unix_fix_u8_to_i8(&size_bytes), end, end + SUPERBLOCK_WIDTH);
        self.below.write(0, self.inode_table.disk_block_index + inner_block_num, &block);
    }

    /// Update the superblock with the next block number, returning the current head.
    fn get_and_update_superblock(&mut self) -> Result<u32, Error> {
        let cur_head = self.superblock.fat_head_value;

        let mut block = Block::new();
        self.below.read(0, 0, &mut block);
        let new_head = Self::get_fat_table_info(cur_head);
        let new_head_bytes = u32::to_le_bytes(new_head);
        block.write_bytes(new_head_bytes, 0, SUPERBLOCK_WIDTH);
        self.below.write(-1, 0, block);

        self.superblock.fat_head_value = new_head;

        Ok(cur_head)
    }
}

impl<T: Stackable + IsDisk> Stackable for FS<T> {
    /// Read the ino'th entry of the inode table 
    fn get_size(&self, ino: u32) -> Result<u32, Error> {
        let (_, size) = get_inode_info(ino);
        return size;
    }

    // TODO what if size is invalid?
    fn set_size(&mut self, ino: u32, size: u32) -> Result<i32, Error> {
        let (inner_block_num, start_byte_within_block) = calc_inode_table_indices(ino);
        
        let mut block = Block::new();
        self.below.read(-1, self.InodeTable.disk_block_index + inner_block_num, block);
        let new_size = u32::to_le_bytes(size);
        // size comes after head
        let beg = start_byte_within_block + SUPERBLOCK_WIDTH;
        let end = beg + SUPERBLOCK_WIDTH;
        block.write_bytes(new_size, beg, end);
        self.below.write(-1, self.InodeTable.disk_block_index + inner_block_num, block);
    }

    /// Read the block at the offset'th block of the ino'th inode.
    fn read(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        let (head, size) = get_inode_info(ino);
        if offset >= size {
            return Err(Error::UnknownFailure);
        }

        let mut block_idx = head;
        for i in 0..offset + 1 {
            let next = get_fat_table_info(block_idx);
            if next == NULL_POINTER && i != offset {
                return Err(Error::UnknownFailure);
            }
            block_idx = next;
        }

        let data_block_start_index = self.FatTable.disk_block_index + self.FatTable.num_blocks;
        self.below.read(-1, data_block_start_index + block_idx, buf);
        Ok(0)
    }

    /// Write the block at the offset'th block of the ino'th inode.
    /// If offset < size, then overwrite the block at offset.
    /// If offset == size, then append the block at offset from freelist, updating the superblock, inode table, and fat table.
    /// If offset > size, then return an error.
    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        let (head, size) = self.get_inode_info(ino)?;
        if offset > size {
            unimplemented!();
        } else if offset < size {
            let mut block_idx = head;
            for i in 0..offset + 1 {
                let next = get_fat_table_info(block_idx);
                if next == NULL_POINTER && i != offset {
                    return Error::UnknownFailure();
                }
                block_idx = next;
            }
            self.below.write(-1, self.FatTable.disk_block_index + self.FatTable.num_blocks + block_idx, buf);
        } else {
            let mut last_block_idx = head;
            for i in 0..offset + 1 {
                let next = get_fat_table_info(last_block_idx);
                if next == NULL_POINTER && i != offset {
                    return Error::UnknownFailure();
                }
                last_block_idx = next;
            }
            if last_block_idx != NULL_POINTER {
                return Error::UnknownFailure();
            }

            let new_block_idx = self.get_and_update_superblock();
            self.update_inode_info(ino, head, size + 1);
            self.update_fat_table_info(last_block_idx, new_block_idx);
            self.update_fat_table_info(new_block_idx, NULL_POINTER);
            
            self.below.write(-1, self.FatTable.disk_block_index + self.FatTable.num_blocks + new_block_idx, buf);
        }
        Ok(0)
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