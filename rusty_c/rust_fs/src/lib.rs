#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// #![cfg_attr(not(test), no_std)]

extern crate panic_halt;

use core::include;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[no_mangle]
pub extern "C" fn add_c(left: usize, right: usize) -> usize {
    add(left, right)
}