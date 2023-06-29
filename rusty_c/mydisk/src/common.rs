!#[feature(alloc)]
!#[feature(alloc_error_handler)]

mod common {
  extern crate alloc;

  use core::panic::PanicInfo;
  use core::alloc::{GlobalAlloc, Layout};
  use core::include;

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

  pub struct Block {
    // [u8: BLOCK_SIZE]
    bytes: block_t
  }

  // TODO impl core::fmt::write::write_str to use write!() macro or use the core::io version

  pub enum Error {
    UnknownFailure
  }

  pub trait Stackable {
    fn get_size(&self) -> 
      Result<u32, Erro>;
    fn set_size(&mut self, size: u32) -> 
      Result<u32, Error>;
    fn read(&self, ino: u32, offset: u32, buf: &mut Block) -> 
      Result<u32, Error>;
    fn write(&self, ino: u32, offset: u32, buf: &mut Block) -> 
      Result<u32, Error>;
  }

  pub trait IsDisk {
    pub static info: &str = "Structs implementing this trait are the disk itself"; 
  }
}