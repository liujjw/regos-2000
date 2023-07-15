#![no_std]

extern crate alloc;

mod common;
use common::*;
use core::include;
use core::convert::From;   
use core::convert::Into; 
use core::mem::size_of;
use alloc::boxed::Box;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

struct DiskFS {
    read: fn(bs: *mut inode_store_t, 
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    write: fn(bs: *mut inode_store_t,
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    get_size: fn() -> cty::c_uint,
    set_size: fn() -> cty::c_int,
}

impl IsDisk for DiskFS {}

// TODO use mutex wrapper over from and a static belows for more memory safety
// nothing prevents another call and another owned instance referring to the same data:
// SimpleFS::setup(&mut DiskFS::from(below), below_ino, ninodes).unwrap_or(-1)
impl From<inode_intf> for DiskFS {
    fn from(inode_store: inode_intf) -> Self {
        if !inode_store.state.is_null() {
            panic!("DiskFS must be the lowest layer, and state is null");
        }
        DiskFS {
            read: unsafe {
                (*inode_store).read
            },
            write: unsafe {
                (*inode_store).write
            },
            get_size: unsafe {
                (*inode_store).getsize
            },
            set_size: unsafe {
                (*inode_store).setsize
            }
        }
    }
}

impl Into<inode_intf> for DiskFS {
    fn into(self) -> inode_intf {
        let mut inode_store = Box::new(inode_store_t {
            state: core::ptr::null_mut(),
            getsize: self.get_size,
            setsize: self.set_size,
            read: self.read,
            write: self.write
        });
        return Box::into_raw(inode_store);
    }
}

impl Stackable for DiskFS {
    fn get_size(&self) -> Result<i32, Error> {
        match self.get_size() {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn set_size(&mut self, size: u32) -> Result<i32, Error> {
        match self.set_size() {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        match self.read(ino, offset, &mut buf.bytes) {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn write(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        match self.write(ino, offset, &mut buf.bytes) {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }
}

struct Metadata {
    row_width: u32,
    num_blocks_needed: u32,
}

struct SimpleFS<'a, T: Stackable> {
    below: &'a mut T,
    below_ino: u8,
    num_inodes: u32,
    metadata: Option<Metadata>
}

impl From<*mut inode_store_t> for SimpleFS<DiskFS> {
    fn from(inode_store: *mut inode_store_t) -> Self {
        let cur_state = unsafe {
            &mut *inode_store.state
        };
        let below = DiskFS::from(cur_state.below);
        let below_ino = cur_state.below_ino;
        let num_inodes = cur_state.num_inodes;
        SimpleFS::new(below, below_ino, num_inodes)
    }
}

#[repr(C)]
struct SimpleFS_C {
    below: *mut inode_store_t,
    below_ino: cty::c_uint,
    num_inodes: cty::c_uint,
}

// explicit Into Impl overriding compiler default
impl Into<SimpleFS<DiskFS>> for *mut inode_store_t {
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
}

impl<T: Stackable + IsDisk> SimpleFS<T> {
    pub fn new(below: &mut T, below_ino: u8, num_inodes: u32) -> Self {
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
            Err(e) => panic!("failed to compute metadata: {:?}", e)
        }        
    }

    fn compute_metadata(&self) -> Result<Metadata, Error> {
        let blocks_per_inode = self.get_size()?;
        // assume no overflow
        let row_width_in_bytes = libm::ceil(blocks_per_inode as f64 / 8 as f64) as u32;
        // assume rows <= 512 bytes (BLOCK_SIZE)
        if size_of::<[u8; row_width_in_bytes]>() > BLOCK_SIZE {
            panic!("row size exceeds block size");
        }
        let num_blocks_needed = libm::ceil(
            (size_of::<[u8; row_width_in_bytes]>() * self.num_inodes) as f64 / 
            BLOCK_SIZE as f64
        ) as u32;
        Metadata {
            row_width: row_width_in_bytes,
            num_blocks_needed: num_blocks_needed
        }
    }

    pub fn get_metadata<'a>(&'a self) -> Result<&'a Metadata, Error> {
        self.metadata.as_ref().ok_or(Error::UnknownFailure)
    }

    // Have a few blocks in the beginning reserved for metadata about the free blocks.
    // Each of the N inodes has a dynamic n bytes entry in the metadata blocks, 
    // and that entry tells us which blocks inside the inode have been allocated. 
    // Unallocated blocks before the allocated block in the inode are zeroed out.
    // TODO add layers of indirection to minimize metadata table size
    // (currently 1 bit of metadata per 512 bytes of inode data)
    pub fn setup(below: &mut T, below_ino: u32, num_inodes: u32) -> Result<i32, Error> {
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

    fn compute_indices(&mut self, ino: u32, offset: u32) -> Result<(i32, i32, i32), Error> {
        // determine which metadata block this inode row lives on 
        // floor since zero indexed ino
        let block_no = ((ino + 1) * self.metadata.unwrap().row_width) / BLOCK_SIZE;
        let ino_row_starting_byte_index_in_block = 
            (ino * self.metadata.unwrap().row_width) % BLOCK_SIZE;
        let byte_index = (ino_row_starting_byte_index_in_block + offset) / 8; 
        let bit_index = (ino_row_starting_byte_index_in_block + offset) % 8;
        Ok((block_no, byte_index, bit_index))
    }

    // But why have a freelist anyway? The offset is physically mapped to blocks within inodes,
    // whereas in treedisk its only logically mapped.
    // If we want to append to a file, it could be useful to know where we can start, 
    // i.e. we provide a utility function if the API user forgets which blocks are free.
    pub fn is_free(&mut self, ino: u32, offset: u32) -> bool {
        let (block_no, byte_index, bit_index) = self.compute_indices(ino, offset).unwrap();

        let mut buf = Block::new();
        self.below.read(self.below_ino, block_no, buf);
        
        let bytes = buf.get_bytes();
        let byte = bytes[byte_index as usize];
        (byte >> bit_index) & 1 != 1
    }
}

impl<T: Stackable> Stackable for SimpleFS<'_, T> {
    // # of blocks per inode, constant for all inodes
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
    fn read(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        let metadata_offset = self.get_metadata()?.num_blocks_needed;
        let blocks_per_node = self.get_size()?;
        if ino >= self.num_inodes || offset >= blocks_per_node {
            return Err(Error::UnknownFailure);
        }
        let full_offset = (ino * blocks_per_node) + offset + metadata_offset;
        Ok(self.below.read(self.below_ino, full_offset, buf))
    }

    fn write(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
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
        let (block_no, byte_index, bit_index) = self.compute_indices(ino, offset).unwrap();
        let mut buf = Block::new();
        self.below.read(self.below_ino, block_no, buf);
        let bytes = buf.get_bytes();
        let mut byte = bytes[byte_index as usize];
        byte = byte | (1 << bit_index);
        bytes[byte_index as usize] = byte;
        self.below.write(self.below_ino, block_no, buf);

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
        (SimpleFS::new(DiskFS::from(below), below_ino, num_inodes)).into();
    return myfs;
}

// @precondition: assumes below is just the disk
// @precondition: number of total blocks below >> num_inodes
// Returns # of blocks in the given inode, which is constant for every inode 
// (external fragmentation is possible). Semantics of the static keyword may differ
// from C to Rust, we can use static here to keep these functions in the same memory 
// location and not worry about Rust features. 
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
    SimpleFS::from(inode_store).set_size(size).unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn simfs_read(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from(inode_store).read(ino, offset, block).unwrap_or(-1)
}

#[no_mangle]
unsafe extern "C" fn simfs_write(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from(inode_store).write(ino, offset, block).unwrap_or(-1)
}

#[no_mangle]
pub unsafe extern "C" fn simplefs_create(
    below: *mut inode_store_t, 
    below_ino: cty::c_uint,
    ninodes: cty::c_uint
) -> cty::c_int {
    SimpleFS::setup(&mut DiskFS::from(below), below_ino, ninodes).unwrap_or(-1)
}