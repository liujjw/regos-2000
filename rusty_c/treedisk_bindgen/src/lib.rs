#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// #![cfg_attr(not(test), no_std)]

extern crate panic_halt;

use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// #[repr(C)]
// #[derive(Copy, Clone)]
// struct treedisk_snapshot {
//     superblock: treedisk_block, 
//     inodeblock: treedisk_block,
//     inode_blockno: block_no,
//     inode: *mut treedisk_inode,
// }

// #[repr(C)]
// #[derive(Copy, Clone)]
// struct treedisk_state {
//     below: *mut inode_store_t,
//     below_ino: u32,
//     ninodes: u32,
// }

// call within unsafe 
// static mut log_rpb: u32 = 0;
// static mut null_block: block_t = block_t { bytes: [0 as cty::c_char; BLOCK_SIZE as usize] };		

// no panic fn yet

// fn log_shift_r(x: block_no, nbits: u32) -> block_no {
//     if nbits >= core::mem::size_of::<block_no>() as u32 * 8 {
//         return 0;
//     }
//     x >> nbits
// }

// fn treedisk_get_snapshot(snapshot: *mut treedisk_snapshot, 
//                         ts: *mut treedisk_state, inode_no: u32) -> i32 {
//     let snapshot_ref = snapshot.superblock;
//     if (*ts).(*below).read(ts.below, ts.below_ino, 0, snapshot_ref as *mut block_t) < 0 {
//         return -1;
//     }

//     if inode_no >= (*snapshot).superblock.superblock.n_inodeblocks * INODES_PER_BLOCK {
//         return -1;
//     }

//     (*snapshot).inode_blockno = 1 + inode_no / INODES_PER_BLOCK;
//     if (*ts).(*below).read(ts.below, ts.below_ino, (*snapshot).inode_blockno, snapshot_ref as *mut block_t) < 0 {
//         return -1;
//     }

//     snapshot.inode = *mut snapshot.inodeblock.inodeblock.inodes[(inode_no % INODES_PER_BLOCK) as usize];
//     return 0;
// }

// #[no_mangle]
// pub extern "C" 

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//     }
// }