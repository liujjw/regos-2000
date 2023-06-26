!#[no_std]
!#[feature(alloc)]
!#[feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use core::include;
use core::alloc::{GlobalAlloc, Layout};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

struct EgosAllocator;

// use egos allocator or another crates.io impl, then Box, since other C code
// relies on pointers to heap data we cannot use heapless or just the stack
unsafe impl GlobalAlloc for EgosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size() as cty::size_t) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        free(ptr as *mut cty::c_void);
    }
}

#[global_allocator]
static A: EgosAllocator = EgosAllocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn main() {
    
}

struct state {
    below: inode_store_t,
    below_ino: cty::c_uint,
}

#[no_mangle]
pub unsafe extern "C" fn init(below: *mut inode_store_t, below_ino: cty::c_uint) -> inode_store_t {
    let below;
    if let Some(below) = below.as_mut() {
        below = below;
    }
    let mut state = state {
        below: below,
        below_ino: below_ino,
    };
}   
    

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
