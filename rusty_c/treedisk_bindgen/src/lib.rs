#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// #![cfg_attr(not(test), no_std)]

extern crate panic_halt;

use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[no_mangle]
unsafe extern "C" fn log_shift_r(x: block_no, nbits: u32) -> block_no {
    if nbits >= core::mem::size_of::<block_no>() as u32 * 8 {
        return 0;
    }
    x >> nbits
}
