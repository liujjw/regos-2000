#![cfg_attr(not(unix), no_std)]

extern crate alloc;

use crate::common::*;
use alloc::boxed::Box;
use crate::bindings::*;

const SUPERBLOCK_WIDTH: usize = 4;
const INODE_TABLE_ENTRY_WIDTH: usize = 8;
const FAT_TABLE_ENTRY_WIDTH: usize = 4;
const NULL_POINTER: i32 = -1;
const SUPERBLOCK_BLOCK_SIZE: u32 = 1;
/// Snapshot superblock data, contains true disk block offset, and fat head value.
/// -1 means no free space left, 
// TODO write -1 when no space is left
// TODO proper i32 vs u32: cast i32 -> u32 ok if i32 >= 0, u32 -> i32 for large u32 will be incorrect
struct Superblock {
    fat_head_value: i32,
    disk_block_index: u32, 
}
struct InodeTableEntryData {
    head: i32,
    size: i32,
    inode_number: u32
}
struct InodeTable {
    num_blocks: u32,
    disk_block_index: u32
}
struct FatTableEntry {
    next: i32,
    index: i32
}
struct FatTable {
    num_blocks: u32,
    disk_block_index: u32,
}
/// In memory snapshot of disk, immutable fields.
pub struct FS<T: Stackable> {
    below: T,
    below_ino: u32,
    num_inodes: u32,
    num_data_blocks: u32,
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
    let raw_state = unsafe { (*inode_store).state };
    let cur_state: &mut FS_C = unsafe { &mut *(raw_state as *mut FS_C) };
    let below = DiskFS::from_(cur_state.below);
    let below_ino = cur_state.below_ino;
    let num_inodes = cur_state.num_inodes;
    FS::new(below, below_ino, num_inodes)
  }

  fn take_into_(self) -> *mut inode_store_t {
    let cur_state = Box::new(FS_C {
        below: self.below.take_into_(),
        below_ino: self.below_ino,
        num_inodes: self.num_inodes,
    });

    // pointers owned by box must NOT live past their lifetime
    // TODO every into_raw'ed pointer must be freed by C, what api is cleanest?
    let raw_state: *mut FS_C = Box::into_raw(cur_state);
    let void_state_ptr = unsafe { raw_state as *mut cty::c_void };
    let inode_store = Box::new(inode_store_t {
        state: void_state_ptr,
        getsize: Some(
            fs_get_size
                as unsafe extern "C" fn(*mut inode_store_t, cty::c_uint) -> cty::c_int,
        ),
        setsize: Some(
            fs_set_size
                as unsafe extern "C" fn(
                    *mut inode_store_t,
                    cty::c_uint,
                    block_no,
                ) -> cty::c_int,
        ),
        read: Some(
            fs_read
                as unsafe extern "C" fn(
                    *mut inode_store_t,
                    cty::c_uint,
                    block_no,
                    *mut block_t,
                ) -> cty::c_int,
        ),
        write: Some(
            fs_write
                as unsafe extern "C" fn(
                    *mut inode_store_t,
                    cty::c_uint,
                    block_no,
                    *mut block_t,
                ) -> cty::c_int,
        ),
    });
    return Box::into_raw(inode_store);
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
    fn calc_num_blocks_of_disk(below: &mut T) -> Result<u32, Error> {
        let num_blocks_of_disk = below.get_size(0)?;
        return Ok(num_blocks_of_disk);
    }

    fn calc_number_of_total_data_blocks(
        num_blocks_needed_for_fat_table: u32,
        num_blocks_for_superblock: u32,
        num_blocks_needed_for_inode_table: u32,
        num_blocks_of_disk: u32
    ) -> Result<u32, Error> {
        if num_blocks_of_disk <= num_blocks_needed_for_fat_table + num_blocks_for_superblock + num_blocks_needed_for_inode_table {
            return Err(Error::DiskTooSmall);
        } else {
            return Ok(num_blocks_of_disk - (num_blocks_needed_for_fat_table + num_blocks_for_superblock + num_blocks_needed_for_inode_table));
        }
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

        if num_blocks_of_disk <= num_blocks_for_superblock - 1 {
            panic!("disk too small");
        }
        let num_entries = num_blocks_of_disk - num_blocks_needed_for_inode_table - num_blocks_for_superblock;
        let mut num_blocks_needed_for_fat_table = libm::ceil(
            (num_entries * FAT_TABLE_ENTRY_WIDTH as u32) as f64 / BLOCK_SIZE as f64
        ) as u32;
        num_blocks_needed_for_fat_table = libm::ceil(
            ((4 * num_blocks_of_disk) - (4 * num_blocks_needed_for_inode_table) - 4) as f64 / 516 as f64
        ) as u32;
        return num_blocks_needed_for_fat_table;
    }

    /// new calculates info from setup_disk and stores it in memory (snapshot)
    /// assumes setup_disk is called first
    /// invariant: all params are equal to ones called on setup_disk and immutable
    fn new(mut below: T, below_ino: u32, num_inodes: u32) -> Self {
        let num_blocks_inode_table = Self::calc_inode_table(num_inodes);
        let num_blocks_superblock = 1;
        let num_blocks_of_disk = match Self::calc_num_blocks_of_disk(&mut below) {
            Ok(ans) => ans,
            Err(e) => 0
        };
        let num_blocks_fat_table = Self::calc_fat_table(
            num_blocks_of_disk, 
            num_blocks_inode_table,
            num_blocks_superblock
        );
        FS {
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes,
            num_data_blocks: Self::calc_number_of_total_data_blocks(
                num_blocks_fat_table,
                SUPERBLOCK_BLOCK_SIZE,
                num_blocks_inode_table,
                num_blocks_of_disk
            ).unwrap(),
            superblock: Superblock {
                fat_head_value: 0,
                disk_block_index: 0
            },
            inode_table: InodeTable {
                num_blocks: num_blocks_inode_table,
                disk_block_index: SUPERBLOCK_BLOCK_SIZE
            },
            fat_table: FatTable {
                num_blocks: num_blocks_fat_table,
                disk_block_index: SUPERBLOCK_BLOCK_SIZE + num_blocks_inode_table
            }
        }
    }

    /// see https://www.cs.cornell.edu/courses/cs4411/2020sp/schedule/slides/06-Filesystems-FAT.pdf
    /// setup persists to disk with empty data
    /// invariant: all params are equal to the ones passed into init() and immuatable
    fn setup_disk(below: &mut T, below_ino: u32, num_inodes: u32) -> Result<i32, Error> {
        // the superblock is first 4 bytes and first block on the disk
        // the 4 bytes represent a i32
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
        // each entry for each inode is 8 bytes wide
        // every 4 bytes is the le representation of -1 starting out
        // the the -1 means that the size is unitialized, and the head is unitialized
        // write num_blocks_needed_for_inode_table blocks 
        // the 4 bytes represent an i32
        // if a rw operation on an out of bounds inode, snapshot (data from init) will error
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
        let beg = SUPERBLOCK_BLOCK_SIZE as usize;
        let end = (SUPERBLOCK_BLOCK_SIZE + num_blocks_needed_for_inode_table) as usize;
        for block_num in beg..end {
            below.write(0, block_num as u32, &inode_table_data)?;
        }

        // fat table starts at the block after the inode table
        // each entry is 4 bytes wide, maps directly to a block
        // every 4 bytes is the le representation of the next block
        // -1 means null
        // how many entries needed? depends on number of blocks we have left
        // the 4 bytes are an i32
        // if no more space left, snapshot (data from init) will error
        let num_blocks_needed_for_fat_table = Self::calc_fat_table(
            Self::calc_num_blocks_of_disk(below)?, 
            num_blocks_needed_for_inode_table,
            1
        );
        let mut fat_table_data = Block::new();
        let end = BLOCK_SIZE as usize / SUPERBLOCK_WIDTH;
        for i in 0..end {
            let bytes = i32::to_le_bytes(i as i32 + 1);
            #[cfg(unix)] {
                let bytes = unix_fix_u8_to_i8(&bytes);
                fat_table_data.write_bytes(&bytes, i * SUPERBLOCK_WIDTH, (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH);
            }
            #[cfg(not(unix))] {
                fat_table_data.write_bytes(&bytes, i * SUPERBLOCK_WIDTH, (i * SUPERBLOCK_WIDTH) + SUPERBLOCK_WIDTH);
            }
        }
        let beg = (SUPERBLOCK_BLOCK_SIZE + num_blocks_needed_for_inode_table) as usize;
        let end = (SUPERBLOCK_BLOCK_SIZE + num_blocks_needed_for_inode_table + num_blocks_needed_for_fat_table) as usize;
        for block_num in beg..end {
            below.write(0, block_num as u32, &fat_table_data)?;
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
    fn get_fat_table_info(&mut self, idx: i32) -> i32 {
        if idx < 0 {
            return NULL_POINTER;
        }
        let (inner_block_num, start_byte_within_block) = Self::calc_fat_table_indices(idx as u32);

        let mut block = Block::new();
        self.below.read(0, self.fat_table.disk_block_index + inner_block_num, &mut block);

        let bytes = block.read_bytes();
        let beg = start_byte_within_block as usize;
        let end = start_byte_within_block as usize + FAT_TABLE_ENTRY_WIDTH;
        let next_ = &bytes[beg..end];
        let next = i32::from_le_bytes(unix_fix_i8_to_u8(next_));
        if next == NULL_POINTER {
            return NULL_POINTER;
        } 
        return next;
    }

    /// Update the fat table with the next block number index for a given index.
    fn update_fat_table_info(&mut self, idx: u32, next: i32) {
        let (inner_block_num, start_byte_within_block) = Self::calc_fat_table_indices(idx);

        let mut block = Block::new();
        self.below.read(0, self.fat_table.disk_block_index + inner_block_num, &mut block);

        let next_bytes = i32::to_le_bytes(next);
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
    fn get_inode_info(&mut self, ino: u32) -> Result<(i32, i32), Error> {
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
        return Ok((head, size));
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

    /// Update the superblock with the next free block number, returning the current head.
    /// Error is returned if there is no space left.
    fn get_and_update_superblock(&mut self) -> Result<u32, Error> {
        let cur_head = self.superblock.fat_head_value;
        if cur_head == -1 {
            return Err(Error::UnknownFailure);
        } else if cur_head >= self.num_data_blocks as i32 {
            return Err(Error::OutOfSpace);
        }
        let mut block = Block::new();
        self.below.read(0, 0, &mut block);
        let new_head = self.get_fat_table_info(cur_head);
        let new_head_bytes = i32::to_le_bytes(new_head);
        block.write_bytes(&unix_fix_u8_to_i8(&new_head_bytes), 0, SUPERBLOCK_WIDTH);
        self.below.write(0, 0, &block);

        self.superblock.fat_head_value = new_head;

        Ok(cur_head as u32)
    }
}

impl<T: Stackable + IsDisk> Stackable for FS<T> {
    /// Read the ino'th entry of the inode table 
    fn get_size(&mut self, ino: u32) -> Result<u32, Error> {
        let (_, size) = self.get_inode_info(ino)?;
        if size <= -1 {
            return Err(Error::UnknownFailure);
        }
        return Ok(size as u32);
    }

    // TODO what if size is invalid?
    fn set_size(&mut self, ino: u32, size: u32) -> Result<i32, Error> {
        let (inner_block_num, start_byte_within_block) = Self::calc_inode_table_indices(ino);
        
        let mut block = Block::new();
        self.below.read(0, self.inode_table.disk_block_index + inner_block_num, &mut block);
        let new_size = u32::to_le_bytes(size);
        // size comes after head
        let beg = start_byte_within_block as usize + SUPERBLOCK_WIDTH;
        let end = beg + SUPERBLOCK_WIDTH;
        block.write_bytes(&unix_fix_u8_to_i8(&new_size), beg, end);
        self.below.write(0, self.inode_table.disk_block_index + inner_block_num, &block)
    }

    /// Read the block at the offset'th block of the ino'th inode.
    fn read(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        if ino >= self.num_inodes {
            return Err(Error::InodeOutOfBounds);
        }
        let (head, size) = self.get_inode_info(ino)?;
        if offset as i32 >= size {
            return Err(Error::ReadOffsetTooLarge);
        }
        if head < 0 {
            return Err(Error::UnitializedInode);
        }

        let mut block_idx = head;
        for i in 0..offset {
            let next = self.get_fat_table_info(block_idx);
            if next == NULL_POINTER && i != offset {
                return Err(Error::UnknownFailure);
            }
            block_idx = next;
        }

        let data_block_start_index = self.fat_table.disk_block_index + self.fat_table.num_blocks;
        self.below.read(0, data_block_start_index + block_idx as u32, buf)?;
        Ok(0)
    }

    /// Write the block at the offset'th block of the ino'th inode.
    /// If offset < size, then overwrite the block at offset.
    /// If offset == size, then append the block at offset from freelist, updating the superblock, inode table, and fat table. Update size.
    /// If offset > size, then return an error.
    /// Finally, if the ino has never been written, initialize it.
    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        if ino >= self.num_inodes {
            return Err(Error::InodeOutOfBounds);
        }
        let (head, size) = self.get_inode_info(ino)?;
        if head < 0 && size < 0 {
            let new_block_idx = self.get_and_update_superblock()?;
            self.update_inode_info(ino, new_block_idx as u32, 1);
            self.update_fat_table_info(new_block_idx, NULL_POINTER);
            
            self.below.write(0, self.fat_table.disk_block_index + self.fat_table.num_blocks + new_block_idx, buf)?;
        } else if offset > size as u32 {
            unimplemented!();
        } else if offset < size as u32 {
            let mut block_idx = head;
            for i in 0..offset {
                let next = self.get_fat_table_info(block_idx);
                block_idx = next;
            }
            self.below.write(0, self.fat_table.disk_block_index + self.fat_table.num_blocks + block_idx as u32, buf)?;
        } else if offset == size as u32 {
            let mut last_block_idx = head;
            for _ in 0..offset-1 {
                let next = self.get_fat_table_info(last_block_idx);
                last_block_idx = next;
            }

            let new_block_idx = self.get_and_update_superblock()?;
            self.update_inode_info(ino, head as u32, size as u32 + 1);
            self.update_fat_table_info(last_block_idx as u32, new_block_idx as i32);
            self.update_fat_table_info(new_block_idx, NULL_POINTER);
            
            self.below.write(0, self.fat_table.disk_block_index + self.fat_table.num_blocks + new_block_idx, buf)?;
        } else {
            return Err(Error::UnknownCase);
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
        let state = (*myfs).state as *mut FS_C;
        let below = (*state).below;
    }

    return myfs;
}

pub fn fs_init_rs<T>(
    below: T,
    below_ino: u32,
    num_inodes: u32,
) -> FS<T> where T: Stackable + IsDisk {
    FS::new(below, below_ino, num_inodes)
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

pub fn fs_create_rs<T>(
    below: &mut T,
    below_ino: u32,
    ninodes: u32,
) -> i32 where T: Stackable + IsDisk {
    FS::setup_disk(below, below_ino, ninodes).unwrap_or(-1)
}


// list of operations to do read write block and inode 0, same and differnt blocks in inodes, every sequence of one operation, every sequence of two, every seq of three, and so on, compare with treedisk or in mem filesytem of array of bytes, small number of blocks?, 
// edge cases?
// code coverage?
/// Test in Rust, with Rust mock objects (instead of C mock objects), 
/// assume in the future all modules are written in Rust. Only a 
/// thin C wrapper to export, which is tested separately.
#[cfg(test)]
mod tests {
    // 32 blocks
    const DEBUG_SIZE: usize = 16384;
    const BLOCK_SIZE: usize = 512;
    const NUM_BLOCKS: u32 = 32;

    const NUM_INODES: u32 = 10;
    const ONE_BLOCK_STRING: &str = "With only 2000 lines of code, egos-2000 implements all the basics";
    const TWO_BLOCK_STRING: &str = "Two households, both alike in dignity
    (In fair Verona, where we lay our scene),
    From ancient grudge break to new mutiny,
    Where civil blood makes civil hands unclean.
    From forth the fatal loins of these two foes
    A pair of star-crossed lovers take their life;
    Whose misadventured piteous overthrows
    Doth with their death bury their parents’ strife.
    The fearful passage of their death-marked love
    And the continuance of their parents’ rage,
    Which, but their children’s end, naught could remove,
    Is now the two hours’ traffic of our stage;
    The which, if you with patient ears attend,
    What here shall miss, our toil shall strive to mend.";

    mod Default {
        use crate::common::Stackable;
        use crate::common::IsDisk;
        use super::*;

        #[derive(PartialEq)]
        #[derive(Debug)]
        pub(super) struct RamFS {
            mem: Box<[u8; DEBUG_SIZE as usize]>,
        }

        /// Return an inode_intf.
        pub(super) fn default() {
            unimplemented!()
        }

        impl RamFS {
            pub(super) fn new() -> Self {
                RamFS {
                    mem: Box::new([0; DEBUG_SIZE as usize]),
                }
            }
        }
        impl IsDisk for RamFS {}

        impl Stackable for RamFS {
            fn get_size(&mut self, ino: u32) -> Result<u32, crate::common::Error> {
                Ok((DEBUG_SIZE / BLOCK_SIZE) as u32)
            }

            fn set_size(&mut self, ino: u32, size: u32) -> Result<i32, crate::common::Error> {
                Ok(0)
            }

            fn read(
                &self,
                ino: u32,
                offset: u32,
                buf: &mut crate::common::Block,
            ) -> Result<i32, crate::common::Error> {
                let bytes = &self.mem[offset as usize * BLOCK_SIZE..(offset as usize + 1) * BLOCK_SIZE];
                buf.write_bytes(unix_fix_u8_to_i8_full(bytes), 0, BLOCK_SIZE);
                Ok(0)
            }

            fn write(
                &mut self,
                ino: u32,
                offset: u32,
                buf: &crate::common::Block,
            ) -> Result<i32, crate::common::Error> {
                let bytes = buf.read_bytes();
                self.mem[offset as usize * BLOCK_SIZE..(offset as usize + 1) * BLOCK_SIZE].copy_from_slice(unix_fix_i8_to_u8_full(bytes));
                Ok(0)
            }
        }
    }

    use super::*;  
    type DiskFS = Default::RamFS;

    // Test that the 'fs_init_rs' function initializes a new FS instance with valid arguments.
    // Coverage on table calculation methods.
    #[test]
    fn test_initialize_new_fs_instance() {
        // Arrange
        let below = DiskFS::new();
        let below_ino = 0;
        let num_inodes = NUM_INODES;

        // Act
        let fs = fs_init_rs(below, below_ino, num_inodes);

        // assert_eq!(fs.below, below);
        assert_eq!(fs.below_ino, below_ino);
        assert_eq!(fs.num_inodes, num_inodes);
        assert_eq!(fs.superblock.fat_head_value, 0);
        assert_eq!(fs.superblock.disk_block_index, 0);
        // 80 bytes needed for 10 inodes
        assert_eq!(fs.inode_table.num_blocks, 1);
        assert_eq!(fs.inode_table.disk_block_index, 1);
        // 32 blocks - 1 for superblock - 1 for inode table = 30 blocks left
        // 30 blocks * 4 bytes per block = 120 bytes needed to reference all blocks
        // 120 bytes / 512 bytes per block = 0.234375 blocks needed 
        // the fat table itself consumes a block, so its actually 29 blocks * 4 bytes
        // see system of equations in fs.rs
        assert_eq!(fs.fat_table.num_blocks, 1);
        assert_eq!(fs.fat_table.disk_block_index, 2);
    }

    #[test]
    fn setup_disk_test() {
        // Create a mock DiskFS instance
        let mut disk_fs = DiskFS::new();

        // Call the fs_create_rs function
        let result = fs_create_rs(&mut disk_fs, 0, NUM_INODES);
        let mut result_ = fs_init_rs(disk_fs, 0, NUM_INODES);
        // Assert that the setup_disk function was called successfully
        assert_eq!(result, 0);

        assert_eq!(result_.get_inode_info(1).unwrap(), (-1, -1));

        assert_eq!(result_.get_fat_table_info(0), 1);
    }

    // Test if the 'fs_create_rs' function successfully sets up the disk with the correct number of blocks and writes the superblock, inode table, and fat table data.
    // Coverage on disk table read and writes.
    #[test]
    fn update_metadata_disk_test() {
        // Create a mock DiskFS instance
        let mut disk_fs = DiskFS::new();

        // Call the fs_create_rs function
        let result = fs_create_rs(&mut disk_fs, 0, NUM_INODES);
        let mut result_ = fs_init_rs(disk_fs, 0, NUM_INODES);
        // Assert that the setup_disk function was called successfully
        assert_eq!(result, 0);

        // Assert that the superblock data is correct
        // Get the current free head for a write, update the free head.
        assert_eq!(result_.get_and_update_superblock().unwrap(), 0);
        assert_eq!(result_.get_and_update_superblock().unwrap(), 1);

        // Set ino 1 to have head and size at 1.
        result_.update_inode_info(1, 1, 1);
        assert_eq!(result_.get_inode_info(1).unwrap(), (1, 1));

        // Set the fat table next pointer at idx 0 to idx 2.
        result_.update_fat_table_info(0, 2);
        assert_eq!(result_.get_fat_table_info(0), 2);
    }

    #[test]
    fn fs_get_size_test() {
        // Create a mock DiskFS instance
        let mut disk_fs = DiskFS::new();

        // Call the fs_create_rs function
        let result = fs_create_rs(&mut disk_fs, 0, NUM_INODES);
        let mut result_ = fs_init_rs(disk_fs, 0, NUM_INODES);
        // Assert that the setup_disk function was called successfully
        assert_eq!(result, 0);

        result_.update_inode_info(1, 1, 1);
        assert_eq!(result_.get_size(1).unwrap(), 1);
    }

    #[test]
    fn fs_set_size_test() {
        // Create a mock DiskFS instance
        let mut disk_fs = DiskFS::new();

        // Call the fs_create_rs function
        let result = fs_create_rs(&mut disk_fs, 0, NUM_INODES);
        let mut result_ = fs_init_rs(disk_fs, 0, NUM_INODES);
        // Assert that the setup_disk function was called successfully
        assert_eq!(result, 0);

        result_.set_size(1, 5);
        assert_eq!(result_.get_size(1).unwrap(), 5);
    }

    #[test]
    fn fs_read_write_test() {
        // Create a mock DiskFS instance
        let mut disk_fs = DiskFS::new();

        // Call the fs_create_rs function
        let result = fs_create_rs(&mut disk_fs, 0, NUM_INODES);
        let mut result_ = fs_init_rs(disk_fs, 0, NUM_INODES);
        // Assert that the setup_disk function was called successfully
        assert_eq!(result, 0);

        let bytes = ONE_BLOCK_STRING.as_bytes();
        let mut block = Block::new();
        block.write_bytes(unix_fix_u8_to_i8_full(bytes), 0, bytes.len());
        let res = result_.write(1, 0, &block);
        assert_eq!(res.unwrap(), 0);

        let mut block_2 = Block::new();
        let res_ = result_.read(1, 0, &mut block_2);
        assert_eq!(res_.unwrap(), 0);
        // let bytes_ = unix_fix_i8_to_u8_full(block_2.read_bytes());
        // std::str::from_utf8(bytes_)
        assert_eq!(block_2.read_bytes(), block.read_bytes());
    }

    #[test]
    fn fs_read_write_test_2blocks() {
        // Create a mock DiskFS instance
        let mut disk_fs = DiskFS::new();

        // Call the fs_create_rs function
        let result = fs_create_rs(&mut disk_fs, 0, NUM_INODES);
        let mut result_ = fs_init_rs(disk_fs, 0, NUM_INODES);
        // Assert that the setup_disk function was called successfully
        assert_eq!(result, 0);

        let bytes = TWO_BLOCK_STRING.as_bytes();

        let mut block = Block::new();
        block.write_bytes(unix_fix_u8_to_i8_full(&bytes[0..512]), 0, 512);
        let res = result_.write(1, 0, &block);
        assert_eq!(res.unwrap(), 0);
        let mut block_ = Block::new();
        block_.write_bytes(unix_fix_u8_to_i8_full(&bytes[512..bytes.len()]), 0, 180);
        let res = result_.write(1, 1, &block_);
        assert_eq!(res.unwrap(), 0);

        let mut block_2 = Block::new();
        let mut res_ = result_.read(1, 0, &mut block_2);
        assert_eq!(res_.unwrap(), 0);
        assert_eq!(block_2.read_bytes(), block.read_bytes());

        let mut block_2_ = Block::new();
        res_ = result_.read(1, 1, &mut block_2_);
        assert_eq!(res_.unwrap(), 0);
        assert_eq!(block_2_.read_bytes(), block_.read_bytes());
    }

    #[test]
    fn gen_read_write() {
        // see python code
    }

}