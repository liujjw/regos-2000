!#[no_std]

mod common;
use common::*;
use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

struct SimpleFS<T: Stackable> {
    below: T,
    below_ino: u8,
    num_inodes: u32,
}

impl<T: Stackable> SimpleFS<T> {
    fn new(below: T, below_ino: u8, num_inodes: u32) -> Self {
        // init(below, below_ino, num_inodes)
        SimpleFS {
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes
        }
    }
}

impl<T: Stackable + IsDisk> Stackable for SimpleFS<T> {
    fn get_size(&self) -> 
        Result<SuccessInfo, ErroCode> {

    }
    fn set_size(&mut self, size: u32) -> 
        Result<SuccessInfo, ErrorCode> {

    }
    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> 
        Result<SuccessInfo, ErrorCode> {

    }
    fn write(&self, ino: u32, offset: u32, buf: &mut Block) -> 
        Result<SuccessInfo, ErrorCode> {

    }
}

// use of mut not thread safe, however mutation occurs during write
#repr(C)]
struct SimpleDiskState {
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
    let mut cur_state = Box::new(SimpleDiskState {
        below: below as *mut inode_store_t,
        below_ino: below_ino,
        num_inodes: num_inodes
    });
    // pointers owned by box must NOT live past their lifetime
    let mut inode_store = Box::new(inode_store_t {
        state: Box::into_raw(cur_state),
        getsize: simfs_get_size, // type is fn(...) -> ... (C-like ptr to function, not a closure)
        setsize: simfs_set_size,
        read: simfs_read,
        write: simfs_write
    });
    return Box::into_raw(inode_store);
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
    let cur_state = unsafe {
        & *inode_store.state
    };
    let below = & *cur_state.below;
    let num = below.getsize();
    let denom = cur_state.num_inodes;
    if denom == 0 || num == 0 || num < denom {
        return -1;
    }
    // implicit floor division
    num / denom
} 

static unsafe extern "C" fn simfs_set_size(
    inode_store: *mut inode_store_t, 
    size: cty::c_int
) -> cty::c_int {
    return -1;
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