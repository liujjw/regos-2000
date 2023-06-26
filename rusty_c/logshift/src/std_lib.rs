// --target=x86_64-unknown-linux-gnu
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// #![cfg_attr(not(test), no_std)]

#[no_mangle]
unsafe extern "C" fn log_shift_r(x: cty::c_uint, nbits: cty::c_uint) -> cty::c_uint {
    if nbits >= core::mem::size_of::<cty::c_uint>() as cty::c_uint * 8 {
        return 0;
    } 
    x >> nbits
}

