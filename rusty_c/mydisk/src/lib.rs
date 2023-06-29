!#[no_std]

mod common;
use common::*;
use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

struct DiskFS;

impl DiskFS {
    fn from_inode_store() -> Self {
        unimplemented!();
    }
}

impl Stackable for DiskFS {
    fn get_size(&self) -> Result<u32, Error> {
        Ok(BLOCK_SIZE)
    }

    fn set_size(&mut self, size: u32) -> Result<u32, Error> {
        return Err(Error::UnknownFailure);
    }

    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<u32, Error> {
        if offset >= BLOCK_SIZE {
            return Err(Error::UnknownFailure);
        }
        let block = unsafe {
            &mut *buf.bytes
        };
        let disk = unsafe {
            &mut *DISK
        };
        let block_no = (ino * BLOCK_SIZE) + offset;
        unsafe {
            disk.read(disk, block_no, block);
        }
        Ok(BLOCK_SIZE)
    }

    fn write(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<u32, Error> {
        if offset >= BLOCK_SIZE {
            return Err(Error::UnknownFailure);
        }
        let block = unsafe {
            &mut *buf.bytes
        };
        let disk = unsafe {
            &mut *DISK
        };
        let block_no = (ino * BLOCK_SIZE) + offset;
        unsafe {
            disk.write(disk, block_no, block);
        }
        Ok(BLOCK_SIZE)
    }
}

struct SimpleFS<T: Stackable, 'a> {
    below: &'a mut T,
    below_ino: u8,
    num_inodes: u32,
}

impl<T: Stackable + IsDisk> SimpleFS<T> {
    fn new(below: &mut T, below_ino: u8, num_inodes: u32) -> Self {
        SimpleFS {
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes
        }
    }

    fn to_inode_store(
        simple_fs: Self,
        get_size: fn(inode_store: *mut inode_store_t, ino: cty::c_uint) -> cty::c_uint,
        set_size: fn(inode_store: *mut inode_store_t, size: cty::c_int) -> cty::c_int,
        read: fn(inode_store: *mut inode_store_t, ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
        write: fn(inode_store: *mut inode_store_t, ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int
    ) -> *mut inode_store_t {
        let cur_state = Box::new(SimpleFS_C {
            below: simple_fs.below as *mut inode_store_t,
            below_ino: simple_fs.below_ino,
            num_inodes: simple_fs.num_inodes
        });
        // pointers owned by box must NOT live past their lifetime
        let mut inode_store = Box::new(inode_store_t {
            state: Box::into_raw(cur_state),
            getsize: get_size,
            setsize: set_size,
            read: read,
            write: write
        });
        return Box::into_raw(inode_store);
    }

    fn from_inode_store(inode_store: *mut inode_store_t) -> Self {
        let cur_state = unsafe {
            &mut *inode_store.state
        };
        let below = &mut *cur_state.below;
        let below_ino = cur_state.below_ino;
        let num_inodes = cur_state.num_inodes;
        SimpleFS {
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes
        }
    }
}

impl<T: Stackable> Stackable for SimpleFS<T> {
    fn get_size(&self) -> Result<u32, Error> {
        let num = self.below.get_size();
        let denom = self.num_inodes;
        if denom == 0 || num == 0 || num < denom {
            return Err(Error::UnknownFailure);
        }
        // implicit floor division
        Ok(num / denom)
    }

    fn set_size(&mut self, size: u32) -> Result<u32, Error> {
        return Err(Error::UnknownFailure);
    }

    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<u32, Error> {
        let blocks_per_node = self.get_size()?;
        if ino >= self.num_inodes || offset >= blocks_per_node {
            return Err(Error::UnknownFailure);
        }
        let full_offset = (ino * blocks_per_node) + offset;
        self.below.read(self.below_ino, full_offset, buf)
    }
    fn write(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<u32, Error> {

    }
}

// use of mut not thread safe, however mutation occurs during write
#repr(C)]
struct SimpleFS_C {
    below: *mut inode_store_t,
    below_ino: cty::c_uint,
    num_inodes: cty::c_uint,
}

#[no_mangle]
pub unsafe extern "C" fn init(
    below: *mut inode_store_t, 
    below_ino: cty::c_uint,
    num_inodes: cty::c_uint) 
-> *mut inode_store_t {
    // assume below is aligned, initialized, and valid, but can check if non-null
    if (below.is_null()) {
        panic!("below is null");
    }
    let myfs = SimpleFS::new(DiskFS::from_inode_store(below), below_ino, num_inodes);
    myfs.to_inode_store(
        simfs_get_size,
        simfs_set_size,
        simfs_read,
        simfs_write
    )    

}

// @precondition: assumes below is just the disk
// @precondition: number of total blocks below >> num_inodes
// Returns # of blocks in the given inode, which is constant for every inode 
// (external fragmentation is possible). Semantics of the static keyword may differ
// from C to Rust, we use static here to keep these functions in the same memory 
// location and not worry about Rust features. 
static unsafe extern "C" fn simfs_get_size(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint
) -> cty::c_uint {
    SimpleFS::from_inode_store(inode_store).get_size().unwrap_or(-1)
} 

static unsafe extern "C" fn simfs_set_size(
    inode_store: *mut inode_store_t, 
    size: cty::c_int
) -> cty::c_int {
    SimpleFS::from_inode_store(inode_store).set_size(size).unwrap_or(-1)
}

// read an inode at block offset return in a block_t
static unsafe extern "C" fn simfs_read(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    let cur_state = unsafe {
        & *inode_store.state
    };
    let blocks_per_node = inode_store.getsize(inode_store, ino);
    if ino >= cur_state.num_inodes || offset >= blocks_per_node {
        return -1;
    }
    let below = & *cur_state.below;
    let full_offset = (ino * blocks_per_node) + offset;
    return below.read(below, ino, full_offset, block);
}

static unsafe extern "C" fn simfs_write(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    let cur_state = unsafe {
        &mut *inode_store.state
    };
    let blocks_per_node = inode_store.getsize(inode_store, ino);
    if ino >= cur_state.num_inodes || offset >= blocks_per_node {
        return -1;
    }
    let below = &mut *state.below;
    let full_offset = (ino * blocks_per_node) + offset;
    return below.write(below, ino, full_offset, block);
}