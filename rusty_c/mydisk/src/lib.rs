#![no_std]

mod common;
use common::*;
use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

struct DiskFS {
    read: fn(bs: *mut inode_store_t, 
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    write: fn(bs: *mut inode_store_t,
            ino: cty::c_uint, offset: block_no, block: *mut block_t) -> cty::c_int,
    get_size: fn() -> cty::c_uint,
    set_size: fn() -> cty::c_int,
}

impl DiskFS {
    fn from_inode_store(inode_store: *mut inode_store_t) -> Self {
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

impl Stackable for DiskFS {
    fn get_size(&self) -> Result<u32, Error> {
        match self.get_size() {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn set_size(&mut self, size: u32) -> Result<u32, Error> {
        match self.set_size() {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<u32, Error> {
        match self.read(ino, offset, &mut buf.bytes) {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
    }

    fn write(&self, ino: u32, offset: u32, buf: &mut Block) -> Result<u32, Error> {
        match self.write(ino, offset, &mut buf.bytes) {
            -1 => Err(Error::UnknownFailure),
            x => Ok(x)
        }
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
        let below = DiskFS::from_inode_store(cur_state.below);
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
        let blocks_per_node = self.get_size()?;
        if ino >= self.num_inodes || offset >= blocks_per_node {
            return Err(Error::UnknownFailure);
        }
        let full_offset = (ino * blocks_per_node) + offset;
        self.below.write(self.below_ino, full_offset, buf)
    }
}

// use of mut not thread safe, however mutation occurs during write
#[repr(C)]
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

static unsafe extern "C" fn simfs_read(
    inode_store: *mut inode_store_t,
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from_inode_store(inode_store).read(ino, offset, block).unwrap_or(-1)
}

static unsafe extern "C" fn simfs_write(
    inode_store: *mut inode_store_t, 
    ino: cty::c_uint,
    offset: block_no,
    block: *mut block_t 
) -> cty::c_int {
    SimpleFS::from_inode_store(inode_store).write(ino, offset, block).unwrap_or(-1)
}