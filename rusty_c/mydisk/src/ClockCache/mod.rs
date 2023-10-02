#![cfg_attr(not(unix), no_std)]

extern crate alloc;

// no_std hashmap, could use alloc::collections::HashMap instead as well
use hashbrown::hash_map::{HashMap, DefaultState, RandomState};
use crate::common::*;
use alloc::boxed::Box;
use crate::bindings::*;
use crate::TreediskFS_RS::*;

struct CacheBlock {
    data: Block,
    ref_bit: bool,
    inode: i32,
    offset: i32,
    is_dirty: bool,
}

// Configuration paramters for the cache, must be set statically
const MAX_SIZE: usize = 10000; 
struct ClockCache<T: Stackable> {
    capacity: usize,
    len: usize,
    // choose primitive array over vecdeque/linked list for performance
    arr: [Option<CacheBlock>; MAX_SIZE],
    clock_hand: usize,
    lookup: HashMap<(i32, i32), usize, DefaultState<RandomState>>,
    below: T,
    below_ino: u32,
    num_inodes: u32,
}

impl<T: Stackable> ClockCache<T> {
    fn new(below: T, below_ino: u32, num_inodes: u32) -> Self {
        ClockCache {
            capacity: MAX_SIZE,
            len: 0,
            arr: [None; MAX_SIZE],
            clock_hand: 0,
            lookup: HashMap::with_hasher(RandomState::default()),
            below: below,
            below_ino: below_ino,
            num_inodes: num_inodes,
        }
    }

    fn clock_find(&mut self) -> usize {
        while self.arr[self.clock_hand]
            .as_ref()
            .map(|block| block.ref_bit)
            .unwrap_or(false)
        {
            self.arr[self.clock_hand].as_mut().unwrap().ref_bit = false;
            self.clock_hand = (self.clock_hand + 1) % self.capacity;
        }
        self.clock_hand
    }

    fn synch(&mut self, inode: i32) {
        if inode == -1 {
            for block_idx in 0..self.len {
                if let Some(block) = self.arr[block_idx].take() {
                    if block.is_dirty {
                        self.below.write(block.inode, block.offset, block.data);
                    }
                }
            }
        } else {
            for block_idx in 0..self.len {
                if let Some(block) = self.arr[block_idx].take() {
                    if block.is_dirty && block.inode == inode {
                        self.below.write(block.inode, block.offset, block.data);
                    }
                }
            }
        }
        self.len = 0;
    }
}

impl<T: Stackable> Stackable for ClockCache<T> {
    fn get_size(&mut self, inode: u32) -> Result<i32, Error> {
        self.below.get_size(inode)?
    }

    fn set_size(&mut self, inode: u32, size: u32) -> Result<i32, Error> {
        self.below.set_size(inode, size)?
    }

    fn write(&mut self, inode: u32, offset: u32, data: &Block) {
        if let Some(&block_idx) = self.lookup.get(&(inode, offset)) {
            self.arr[block_idx] = Some(CacheBlock {
                data: data.clone(),
                ref_bit: true,
                inode,
                offset,
                is_dirty: true,
            });
            self.clock_hand = block_idx;
        } else {
            if self.len < self.capacity {
                let block = CacheBlock {
                    data: data.clone(),
                    ref_bit: true,
                    inode,
                    offset,
                    is_dirty: true,
                };
                let idx = self.len;
                self.arr[idx] = Some(block);
                self.lookup.insert(&(inode, offset), idx);
                self.clock_hand = idx;
                self.len += 1;
            } else {
                let idx = self.clock_find();
                if let Some(block_to_evict) = self.arr[idx].take() {
                    if block_to_evict.is_dirty {
                        self.write(block_to_evict.inode, block_to_evict.offset, &block_to_evict.data);
                    }
                }
                let block = CacheBlock {
                    data: data.clone(),
                    ref_bit: true,
                    inode,
                    offset,
                    is_dirty: true,
                };
                self.arr[idx] = Some(block);
                self.lookup.insert(&(inode, offset), idx);
                self.clock_hand = idx;
                self.lookup.remove(&(block_to_evict.inode, block_to_evict.offset));
            }
        }
    }

    fn read(&self, inode: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {
        if let Some(&block_idx) = self.lookup.get(&(inode, offset)) {
            self.arr[block_idx].as_mut().unwrap().ref_bit = true;
            self.clock_hand = block_idx;
            buf.write_bytes(self.arr[block_idx].as_ref().unwrap().data.read_bytes(), 0, BLOCK_SIZE);
            Ok(0)
        } else {
            if self.len < self.capacity {
                let block = Block::new();
                self.below.read(inode, offset, block)?;
                let idx = self.len;
                let cacheblock = CacheBlock {
                    data: block,
                    ref_bit: true,
                    inode,
                    offset,
                    is_dirty: false,
                };
                self.arr[idx] = Some(cacheblock);
                self.lookup.insert(&(inode, offset), idx);
                self.clock_hand = idx;
                self.len += 1;
                buf.write_bytes(self.arr[idx].as_ref().unwrap().data.read_bytes(), 0, BLOCK_SIZE);
                Ok(0)
            } else {
                let idx = self.clock_find();
                if let Some(block_to_evict) = self.arr[idx].take() {
                    if block_to_evict.is_dirty {
                        self.below.write(block_to_evict.inode, block_to_evict.offset, block_to_evict.data);
                    }
                }
                let block = Block::new();
                self.below.read(inode, offset, block)?;
                let cacheblock = CacheBlock {
                    data: block,
                    ref_bit: true,
                    inode,
                    offset,
                    is_dirty: false,
                };
                self.arr[idx] = Some(cacheblock);
                self.lookup.insert(&(inode, offset), idx);
                self.clock_hand = idx;
                self.lookup.remove(&(block_to_evict.inode, block_to_evict.offset));
                buf.write_bytes(self.arr[idx].as_ref().unwrap().data.read_bytes(), 0, BLOCK_SIZE);
                Ok(0)
            }
        }
    }
}