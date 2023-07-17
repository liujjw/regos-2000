#![no_std]

extern crate alloc;

mod common;
use common::*;
use core::include;
use core::convert::From;   
use core::convert::Into; 
use core::mem::size_of;
use alloc::boxed::Box;
use core::mem::transmute;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// TODO use mutex wrapper over from and a static belows for more memory safety
// nothing prevents another call and another owned instance referring to the same data:
// SimpleFS::setup_disk(&mut DiskFS::from(below), below_ino, ninodes).unwrap_or(-1)

// TODO Make SimpleFS own below, and change other references to owned values and vice versa

// standard c pointers to functions
struct DiskFS {
    ds_read:        unsafe extern "C" fn(bs: *mut inode_store_t, 
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    ds_write:       unsafe extern "C" fn(bs: *mut inode_store_t,
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    ds_get_size:    unsafe extern "C" fn() -> cty::c_uint,
    ds_set_size:    unsafe extern "C" fn() -> cty::c_int,
}

impl IsDisk for DiskFS {}

impl DiskFS {
    // don't use From or Into traits for simplicity
    fn into(self) -> inode_intf {
        // inconsistent function signatures, so we need to wrap them
        unsafe extern "C" fn getsizewrapper(
            this_bs: *mut inode_store_t,
            ino: cty::c_uint
        ) -> cty::c_int {
            self.ds_get_size()
        }
        
        unsafe extern "C" fn setsizewrapper(
            this_bs: *mut inode_store_t, 
            ino: cty::c_uint,
            newsize: block_no
        ) -> cty::c_int {
            self.ds_set_size()
        }
        
        // inode_store_t* is inode_intf
        let mut inode_store = Box::new(inode_store_t {
            state: core::ptr::null_mut(),
            getsize: Some(getsizewrapper),
            setsize: Some(setsizewrapper),
            read: Some(self.ds_read),
            write: Some(self.ds_write)
        });
        return Box::into_raw(inode_store);
    }

    fn from(inode_store: inode_intf) -> Self {
        if !(*inode_store).state.is_null() {
            panic!("DiskFS must be the lowest layer, and state is null");
        }

        unsafe extern "C" fn getsizewrapper() -> cty::c_uint {
            (*inode_store).getsize.unwrap()()
        }

        unsafe extern "C" fn setsizewrapper() -> cty::c_int {
            (*inode_store).setsize.unwrap()()
        }

        DiskFS {
            ds_read: unsafe {
                (*inode_store).read.unwrap()
            },
            ds_write: unsafe {
                (*inode_store).write.unwrap()
            },
            ds_get_size: getsizewrapper,
            ds_set_size: setsizewrapper
        }
    }
}

impl Stackable for DiskFS {
    fn get_size(&self) -> Result<u32, Error> {
        Ok((self.ds_get_size)()) 
    }

    fn set_size(&mut self, size: u32) -> Result<i32, Error> {
        match (self.ds_set_size)() {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        match (self.ds_read)(self.into(), ino, offset, buf.into()) {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        match (self.ds_write)(self.into(), ino, offset, buf.into()) {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }
}
// in bytes
const CONST_ROW_WIDTH = 4;
struct Metadata {
    row_width: u32,
    num_blocks_needed: u32,
}

struct SimpleFS<'a, T: Stackable> {
    below: &'a mut T,
    below_ino: u32,
    num_inodes: u32,
    metadata: Option<Metadata>
}

#[repr(C)]
struct SimpleFS_C {
    below: *mut inode_store_t,
    below_ino: cty::c_uint,
    num_inodes: cty::c_uint,
}

impl<T: Stackable + IsDisk> SimpleFS<'_, T> {
    pub fn new(below: &mut T, below_ino: u32, num_inodes: u32) -> Self {
        let tmp = SimpleFS {
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes,
            metadata: None
        };
        match tmp.compute_metadata() {
            Ok(data) => {
                tmp.metadata = Some(data);
                return tmp;
            },
            Err(e) => panic!("failed to compute metadata")
        }        
    }

    fn from(inode_store: *mut inode_store_t) -> Self {
        let cur_state: &mut SimpleFS_C = unsafe {
            &mut transmute(*((*inode_store).state))
        };
        let below = DiskFS::from(cur_state.below);
        let below_ino = cur_state.below_ino;
        let num_inodes = cur_state.num_inodes;
        SimpleFS::new(&mut below, below_ino, num_inodes)
    }

    // use of mut not thread safe, however mutation occurs during write
    fn into(self) -> *mut inode_store_t {
        let cur_state = Box::new(SimpleFS_C {
            below: self.below.into(),
            below_ino: self.below_ino,
            num_inodes: self.num_inodes
        });
        // pointers owned by box must NOT live past their lifetime
        let mut inode_store = Box::new(inode_store_t {
            state: Box::into_raw(cur_state),
            getsize: simfs_get_size,
            setsize: simfs_set_size,
            read: simfs_read,
            write: simfs_write
        });
        return Box::into_raw(inode_store);
    }

    /// Have a few blocks in the beginning reserved for metadata about the free blocks.
    /// Each of the N inodes has a 4 bytes entry in the metadata blocks, 
    /// and that entry tells us the number of blocks allocated. 
    fn compute_metadata(&self) -> Result<Metadata, Error> {
        // assume rows <= 512 bytes (BLOCK_SIZE)
        if size_of::<[u8; CONST_ROW_WIDTH]>() > BLOCK_SIZE {
            panic!("row size exceeds block size");
        }
        let num_blocks_needed = libm::ceil(
            (size_of::<[u8; CONST_ROW_WIDTH]>() * self.num_inodes) as f64 / 
            BLOCK_SIZE as f64
        ) as u32;
        Metadata {
            row_width: CONST_ROW_WIDTH,
            num_blocks_needed: num_blocks_needed
        }
    }

    pub fn get_metadata<'a>(&'a self) -> Result<&'a Metadata, Error> {
        self.metadata.as_ref().ok_or(Error::UnknownFailure)
    }

    pub fn setup_disk(below: &mut T, below_ino: u32, num_inodes: u32) -> Result<i32, Error> {
        let mut simple_fs = SimpleFS::new(below, below_ino, num_inodes);
        let Metadata {
            row_width,
            num_blocks_needed
        } = simple_fs.get_metadata()?;
        // zero out everything for now
        for i in 0..num_blocks_needed {
            let mut buf = Block::new();
            simple_fs.write(i, 0, &mut buf)?;
        }
        for i in num_blocks_needed..num_inodes {
            let mut buf = Block::new();
            simple_fs.write(i, 0, &mut buf)?;
        }
        
        Ok(0)
    }

    
    /// Determine which metadata block this inode row lives on and which 4-bytes 
    /// in that block to look at.
    fn compute_indices(&mut self, ino: u32) -> Result<(i32, i32), Error> {
        // floor since zero indexed ino
        let block_no = ((ino + 1) * self.metadata.unwrap().row_width) / BLOCK_SIZE;
        let ino_row_starting_byte_index_in_block = 
            (ino * self.metadata.unwrap().row_width) % BLOCK_SIZE;
        let byte_index = ino_row_starting_byte_index_in_block / 8; 
        Ok((block_no, byte_index))
    }

    // beware endianness and alignment, assume 4 bytes of size info
    // riscv is little endian, so prefer little endian
    fn compute_inode_metadata_at(buf: &mut Block, ibyte: u32) -> u32 {
        if CONST_ROW_WIDTH != 4 {
            panic!("row width assumed to be 32");
        }
        if ibyte + 4 > BLOCK_SIZE {
            panic!("byte index out of bounds");
        }
        let bytes = buf.get_bytes();
        let mut byte_slice = bytes[ibyte as usize..(ibyte + CONST_ROW_WIDTH) as usize];
        u32::from_le_bytes(byte_slice.try_into().unwrap())
    }

    fn compute_new_inode_metadata_at(buf: &mut Block, ibyte: u32, val: u32) {
        if CONST_ROW_WIDTH != 4 {
            panic!("row width assumed to be 32");
        }
        if ibyte + 4 > BLOCK_SIZE {
            panic!("byte index out of bounds");
        }
        let bytes = buf.get_bytes();
        let mut byte_slice = bytes[ibyte as usize..(ibyte + CONST_ROW_WIDTH) as usize];
        byte_slice.copy_from_slice(&val.to_le_bytes());
    }

    /// Number of used blocks per inode.
    pub fn blocks_used(&mut self, ino: u32) -> u32 {
        let (block_no, byte_index) = self.compute_indices(ino).unwrap();

        let mut buf = Block::new();
        self.below.read(self.below_ino, block_no, buf);
        
        compute_inode_metadata_at(&mut buf, byte_index)
    }

    pub fn set_blocks_used(&mut self, ino: u32, val: u32) {
        let (block_no, byte_index) = self.compute_indices(ino).unwrap();

        let mut buf = Block::new();
        self.below.read(self.below_ino, block_no, buf);
        
        compute_new_inode_metadata_at(&mut buf, byte_index, val);
        self.below.write(self.below_ino, block_no, buf);
    }
}

impl<T: Stackable> Stackable for SimpleFS<'_, T> {
    /// # of blocks per inode, constant for all inodes
    fn get_size(&self) -> Result<u32, Error> {
        let num = self.below.get_size();
        let denom = self.num_inodes;
        if denom == 0 || num == 0 || num < denom {
            return Err(Error::UnknownFailure);
        }
        // implicit floor division
        Ok(num / denom)
    }
    
    fn set_size(&mut self, size: u32) -> Result<i32, Error> {
        return Err(Error::UnknownFailure);
    }
    
    // We will need to shift reads and writes over by the size of the metadata blocks.
    // Assume we start writing at offset 0.
    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        let blocks_used = self.blocks_used(ino);
        if offset >= blocks_used {
            return Err(Error::UnknownFailure);
        }
        let metadata_offset = self.get_metadata()?.num_blocks_needed;
        let blocks_per_node = self.get_size()?;
        if ino >= self.num_inodes || offset >= blocks_per_node {
            return Err(Error::UnknownFailure);
        }
        let full_offset = (ino * blocks_per_node) + offset + metadata_offset;
        Ok(self.below.read(self.below_ino, full_offset, buf))
    }

    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        let metadata_offset = self.get_metadata()?.num_blocks_needed;
        let blocks_per_node = self.get_size()?;
        if ino >= self.num_inodes || offset >= blocks_per_node {
            return Err(Error::UnknownFailure);
        }
        let full_offset = (ino * blocks_per_node) + offset + metadata_offset;
        let res = self.below.write(self.below_ino, full_offset, buf);
        if res < 0 {
            panic!("failed to write");
        }

        // update metadata
        let mut blocks_used = self.blocks_used(ino);
        if offset == blocks_used {
            blocks_used += 1;
            self.set_blocks_used(ino, blocks_used);
        } else if offset > blocks_used {
            panic!("offset > blocks_used");
        }

        // success
        res
    }
}

#[no_mangle]
pub unsafe extern "C" fn init(
    below: *mut inode_store_t, 
    below_ino: cty::c_uint,
    num_inodes: cty::c_uint) 
-> *mut inode_store_t {
    // assume below is aligned, initialized, and valid, but can check if non-null
    if below.is_null() {
        panic!("below is null");
    }
    let myfs: *mut inode_store_t = 
        (SimpleFS::new(&mut DiskFS::from(below), below_ino, num_inodes)).into();
    return myfs;
}

/// @precondition: assumes below is just the disk
/// @precondition: number of total blocks below >> num_inodes
/// Returns # of blocks in the given inode, which is constant for every inode 
/// (external fragmentation is possible). Semantics of the static keyword may differ
/// from C to Rust, we can use static here to keep these functions in the same memory 
/// location and not worry about Rust features. 
#[no_mangle]
unsafe extern "C" fn simfs_get_size(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint
) -> cty::c_uint {
    SimpleFS::from(inode_store).get_size().unwrap_or(-1)
} 

#[no_mangle]
unsafe extern "C" fn simfs_set_size(
    inode_store: *mut inode_store_t, 
    size: cty::c_int
) -> cty::c_int {
    if size < 0 {
        panic!("size must be non-negative");
    }
    SimpleFS::from(inode_store).set_size(size as u32).unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn simfs_read(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from(inode_store).read(ino, offset, &mut Block::from(block)).unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn simfs_write(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from(inode_store).write(ino, offset, &mut Block::from(block)).unwrap_or(-1)
}

#[no_mangle]
pub unsafe extern "C" fn simplefs_create(
    below: *mut inode_store_t, 
    below_ino: cty::c_uint,
    ninodes: cty::c_uint
) -> cty::c_int {
    SimpleFS::setup_disk(&mut DiskFS::from(below), below_ino, ninodes).unwrap_or(-1)
}