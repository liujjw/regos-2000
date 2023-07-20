#![no_std]

extern crate alloc;

mod common;
use common::*;
use core::include;
use core::mem::size_of;
use alloc::boxed::Box;
use core::mem::transmute_copy;
use core::mem::transmute;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// TODO document: For type inconsistencies from egos C, favor the inode_store_t struct when
/// interfacing with egos C code as canonical; favor Rust Stackable trait otherwise.
/// Type inconsistencies may result in undefined behavior, but we leave it to egos C
/// to handle that if it comes up.
// The inconsistencies exist in the function signatures of the implementations of the 
// inode_store_t struct for treedisk.c and disk.c.

/// TODO document: To convert to and from egos C types, we copy memory with the intent
/// to own it (freeing the memory originating from C). We return copies of heap memory 
/// to egos C if really necessary, otherwise all heap memory originating from Rust 
/// is owned and managed by Rust.

// TODO use mutex wrapper over from and a static belows for more memory safety
// nothing prevents another call and another owned instance referring to a copy of the data:
// SimpleFS::setup_disk(&mut DiskFS::from(below), below_ino, ninodes).unwrap_or(-1)

// TODO Make SimpleFS own below, and change other references to owned values and vice versa
  
impl Block {
    pub fn new() -> Block {
        Block {
            bytes: [0; BLOCK_SIZE as usize]
        }
    }

    pub fn get_bytes<'a>(&'a self) -> &'a [u8] {
        &self.bytes
    }

    pub fn from_(block: *mut block_t) -> Self {
        unsafe {
            Block {
                bytes: (*block).bytes
            }
        }
    }

    // TODO stack vs heap *mut block_t
    pub fn into_(&self) -> *mut block_t {
        let mut bytes_copy: [u8; BLOCK_SIZE as usize] = [0; BLOCK_SIZE as usize];
        (&mut bytes_copy).copy_from_slice(self.get_bytes());
        let new_block: *mut block_t = &mut block_t {
            bytes: bytes_copy
        };
        return new_block;
    }    

    pub fn heap_into_(self) -> *mut block_t {
        let mut block = Box::new(block_t {
            bytes: [0; BLOCK_SIZE as usize]
        });
        block.bytes = self.bytes;
        Box::into_raw(block)
    }
}

// standard c pointers to functions
struct DiskFS {
    ds_read:        unsafe extern "C" fn(bs: *mut inode_store_t, 
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    ds_write:       unsafe extern "C" fn(bs: *mut inode_store_t,
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    // in real diskfs.c, inconsistent function signatures with no arguments
    ds_get_size:    unsafe extern "C" fn(this_bs: *mut inode_store, ino: cty::c_uint) -> cty::c_int,
    ds_set_size:    unsafe extern "C" fn(this_bs: *mut inode_store, ino: cty::c_uint, newsize: block_no) -> cty::c_int,
}

impl IsDisk for DiskFS {}

impl DiskFS {
    fn take_into_(self) -> inode_intf {
        // inode_store_t* is inode_intf
        let mut inode_store = Box::new(inode_store_t {
            state: core::ptr::null_mut(),
            getsize: Some(self.ds_get_size),
            setsize: Some(self.ds_set_size),
            read: Some(self.ds_read),
            write: Some(self.ds_write)
        });
        return Box::into_raw(inode_store);
    }

    fn into_(&self) -> inode_intf {
        // inode_store_t* is inode_intf
        let mut inode_store = Box::new(inode_store_t {
            state: core::ptr::null_mut(),
            getsize: Some(self.ds_get_size),
            setsize: Some(self.ds_set_size),
            read: Some(self.ds_read),
            write: Some(self.ds_write)
        });
        return Box::into_raw(inode_store);
    }

    fn from_(inode_store: inode_intf) -> Self {
        unsafe {
            if !(*inode_store).state.is_null() {
                panic!("DiskFS must be the lowest layer, and state is null");
            }
        }

        // unsafe extern "C" fn getsizewrapper() -> cty::c_uint {
        //     (*inode_store).getsize.unwrap()()
        // }

        // unsafe extern "C" fn setsizewrapper() -> cty::c_int {
        //     (*inode_store).setsize.unwrap()()
        // }

        DiskFS {
            ds_read: unsafe {
                (*inode_store).read.unwrap()
            },
            ds_write: unsafe {
                (*inode_store).write.unwrap()
            },
            ds_get_size: unsafe {
                (*inode_store).getsize.unwrap()
            },
            ds_set_size: unsafe {
                (*inode_store).setsize.unwrap()
            }
        }
    }
}

impl Stackable for DiskFS {
    fn get_size(&self) -> Result<u32, Error> {
        // make up dummy arguments
        // https://stackoverflow.com/questions/36005527/why-can-functions-with-no-arguments-defined-be-called-with-any-number-of-argumen
       unsafe {
            Ok((self.ds_get_size)(core::ptr::null_mut(), 0) as u32) 
       }
    }

    fn set_size(&mut self, size: u32) -> Result<i32, Error> {
        // make up dummy arguments, the real diskfs.c has no arguments
        unsafe {
            match (self.ds_set_size)(core::ptr::null_mut(), 0, 0) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x)
            }
        }
    }

    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        unsafe {
            match (self.ds_read)(self.into_(), ino, offset, buf.into_()) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x)
            }
        }
    }

    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        unsafe {
            match (self.ds_write)(self.into_(), ino, offset, buf.into_()) {
                -1 => Err(Error::UnknownFailure),
                x => Ok(x)
            }
        }
    }
}
// in bytes
const CONST_ROW_WIDTH: u32 = 4;
struct Metadata {
    row_width: u32,
    num_blocks_needed: u32,
}

struct SimpleFS<T: Stackable> {
    below: T,
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

impl SimpleFS<DiskFS> {
    fn from_(inode_store: *mut inode_store_t) -> Self {
        // TODO dangerous transmutation to different sized type
        let cur_state: &mut SimpleFS_C = unsafe {
            &mut transmute_copy(&*((*inode_store).state))
        };
        let below = DiskFS::from_(cur_state.below);
        let below_ino = cur_state.below_ino;
        let num_inodes = cur_state.num_inodes;
        SimpleFS::new(below, below_ino, num_inodes)
    }

    // use of mut not thread safe, however mutation occurs during write
    fn take_into_(self) -> *mut inode_store_t {
        let cur_state = Box::new(SimpleFS_C {
            below: self.below.take_into_(),
            below_ino: self.below_ino,
            num_inodes: self.num_inodes
        });
        // pointers owned by box must NOT live past their lifetime

        let mut tmp: *mut cty::c_void = unsafe {
            transmute_copy(&*Box::into_raw(cur_state))
        };
        // tmp: *mut cty::c_void = &mut tmp;
        let mut inode_store = Box::new(inode_store_t {
            state: tmp,
            getsize: Some(
                simfs_get_size as unsafe extern "C" fn(*mut inode_store_t, cty::c_uint) -> cty::c_int
            ),
            setsize: Some(
                simfs_set_size as unsafe extern "C" fn(*mut inode_store_t, cty::c_uint, block_no) -> cty::c_int
            ),
            read: Some(
                simfs_read as unsafe extern "C" fn(*mut inode_store_t, cty::c_uint, block_no, *mut block_t) -> cty::c_int
            ),
            write: Some(
                simfs_write as unsafe extern "C" fn(*mut inode_store_t, cty::c_uint, block_no, *mut block_t) -> cty::c_int
            )
        });
        return Box::into_raw(inode_store);
    }
}

impl<T: Stackable + IsDisk> SimpleFS<T> {
    pub fn new(below: T, below_ino: u32, num_inodes: u32) -> Self {
        let tmp = SimpleFS {
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes,
            metadata: None
        };
        match Self::compute_metadata(num_inodes) {
            Ok(data) => {
                tmp.metadata = Some(data);
                return tmp;
            },
            Err(e) => panic!("failed to compute metadata")
        }        
    }

    /// Have a few blocks in the beginning reserved for metadata about the free blocks.
    /// Each of the N inodes has a 4 bytes entry in the metadata blocks, 
    /// and that entry tells us the number of blocks allocated. 
    pub fn compute_metadata(num_inodes: u32) -> Result<Metadata, Error> {
        // assume rows <= 512 bytes (BLOCK_SIZE)
        if size_of::<[u8; CONST_ROW_WIDTH as usize]>() > BLOCK_SIZE as usize {
            panic!("row size exceeds block size");
        }
        let num_blocks_needed = libm::ceil(
            (size_of::<[u8; CONST_ROW_WIDTH as usize]>() * num_inodes as usize) as f64 / 
            BLOCK_SIZE as f64
        ) as u32;
        Ok(Metadata {
            row_width: CONST_ROW_WIDTH,
            num_blocks_needed: num_blocks_needed
        })
    }

    pub fn get_metadata<'a>(&'a self) -> Result<&'a Metadata, Error> {
        self.metadata.as_ref().ok_or(Error::UnknownFailure)
    }

    pub fn setup_disk(below: &mut T, below_ino: u32, num_inodes: u32) -> Result<i32, Error> {
        let Metadata {
            row_width,
            num_blocks_needed
        } = Self::compute_metadata(num_inodes)?;
        // zero out everything for now
        // TODO zero out another way
        // for i in 0..num_blocks_needed {
        //     let mut buf = Block::new();
        //     simple_fs.write(i, 0, &mut buf)?;
        // }
        // for i in num_blocks_needed..num_inodes {
        //     let mut buf = Block::new();
        //     simple_fs.write(i, 0, &mut buf)?;
        // }
        
        Ok(0)
    }

    
    /// Determine which metadata block this inode row lives on and which 4-bytes 
    /// in that block to look at.
    fn compute_indices(&mut self, ino: u32) -> Result<(u32, u32), Error> {
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
        let mut byte_slice = &bytes[ibyte as usize..(ibyte + CONST_ROW_WIDTH) as usize];
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
        let mut byte_slice = &bytes[ibyte as usize..(ibyte + CONST_ROW_WIDTH) as usize];
        byte_slice.copy_from_slice(&val.to_le_bytes());
    }

    /// Number of used blocks per inode.
    pub fn blocks_used(&mut self, ino: u32) -> u32 {
        let (block_no, byte_index) = self.compute_indices(ino).unwrap();

        let mut buf = Block::new();
        self.below.read(self.below_ino, block_no, &mut buf);
        
        Self::compute_inode_metadata_at(&mut buf, byte_index)
    }

    pub fn set_blocks_used(&mut self, ino: u32, val: u32) {
        let (block_no, byte_index) = self.compute_indices(ino).unwrap();

        let mut buf = Block::new();
        self.below.read(self.below_ino, block_no, &mut buf);
        
        Self::compute_new_inode_metadata_at(&mut buf, byte_index, val);
        self.below.write(self.below_ino, block_no, &mut buf);
    }
}

impl<T: Stackable + IsDisk> Stackable for SimpleFS<T> {
    /// # of blocks per inode, constant for all inodes
    // TODO update to the appropriate blocks_used impl
    fn get_size(&self) -> Result<u32, Error> {
        let num = self.below.get_size()?;
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
        self.below.read(self.below_ino, full_offset, buf)
    }

    fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error> {
        let metadata_offset = self.get_metadata()?.num_blocks_needed;
        let blocks_per_node = self.get_size()?;
        if ino >= self.num_inodes || offset >= blocks_per_node {
            return Err(Error::UnknownFailure);
        }
        let full_offset = (ino * blocks_per_node) + offset + metadata_offset;
        let res = self.below.write(self.below_ino, full_offset, buf)?;

        // update metadata
        let mut blocks_used = self.blocks_used(ino);
        if offset == blocks_used {
            blocks_used += 1;
            self.set_blocks_used(ino, blocks_used);
        } else if offset > blocks_used {
            panic!("offset > blocks_used");
        }

        // success
        Ok(res)
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
        (SimpleFS::new(DiskFS::from_(below), below_ino, num_inodes)).take_into_();
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
) -> cty::c_int {
    match SimpleFS::from_(inode_store).get_size() {
        Ok(val) => val as i32,
        Err(_) => -1
    }
} 

#[no_mangle]
unsafe extern "C" fn simfs_set_size(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint,
    newsize: block_no
) -> cty::c_int {
    if newsize < 0 {
        panic!("size must be non-negative");
    }
    SimpleFS::from_(inode_store).set_size(newsize as u32).unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn simfs_read(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from_(inode_store).read(ino, offset, &mut Block::from_(block)).unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn simfs_write(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from_(inode_store).write(ino, offset, &mut Block::from_(block)).unwrap_or(-1)
}

#[no_mangle]
pub unsafe extern "C" fn simplefs_create(
    below: *mut inode_store_t, 
    below_ino: cty::c_uint,
    ninodes: cty::c_uint
) -> cty::c_int {
    SimpleFS::setup_disk(&mut DiskFS::from_(below), below_ino, ninodes).unwrap_or(-1)
}