pub trait Stackable {
  async fn get_size(&mut self, ino: u32) -> Result<u32, Error>;
  async fn set_size(&mut self, ino: u32, size: u32) -> Result<i32, Error>;
  // &mut is safer for compatiblity with C, since below will call read and needs a *mut
  // however, assume read does not mutate 
  async fn read(&mut self, ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error>;
  async fn write(&mut self, ino: u32, offset: u32, buf: &Block) -> Result<i32, Error>;
}