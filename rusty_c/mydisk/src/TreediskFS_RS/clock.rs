use std::collections::HashMap;

struct Below {
    mp: HashMap<(i32, i32), Vec<u8>>,
}

impl Below {
    fn new() -> Self {
        Below {
            mp: HashMap::new(),
        }
    }

    fn read(&mut self, inode: i32, offset: i32) -> Option<&Vec<u8>> {
        self.mp.get(&(inode, offset))
    }

    fn write(&mut self, inode: i32, offset: i32, data: Vec<u8>) {
        self.mp.insert((inode, offset), data);
    }
}

struct Block {
    data: Vec<u8>,
    ref_bit: bool,
    inode: i32,
    offset: i32,
    is_dirty: bool,
}

struct ClockCache {
    capacity: usize,
    len: usize,
    arr: Vec<Option<Block>>,
    clock_hand: usize,
    lookup: HashMap<(i32, i32), usize>,
}

impl ClockCache {
    fn new(capacity: usize) -> Self {
        ClockCache {
            capacity,
            len: 0,
            arr: vec![None; capacity],
            clock_hand: 0,
            lookup: HashMap::new(),
        }
    }

    fn clock_find(&mut self) -> usize {
        while self.arr[self.clock_hand].as_ref().unwrap().ref_bit {
            self.arr[self.clock_hand].as_mut().unwrap().ref_bit = false;
            self.clock_hand = (self.clock_hand + 1) % self.capacity;
        }
        self.clock_hand
    }

    fn read_below(&mut self, below: &mut Below, inode: i32, offset: i32) -> Block {
        let data = below.read(inode, offset).cloned().unwrap_or_else(Vec::new);
        Block {
            data,
            ref_bit: true,
            inode,
            offset,
            is_dirty: false,
        }
    }

    fn write_below(&mut self, below: &mut Below, inode: i32, offset: i32, data: Vec<u8>) {
        below.write(inode, offset, data);
    }

    fn read(&mut self, below: &mut Below, inode: i32, offset: i32) -> Vec<u8> {
        if let Some(&idx) = self.lookup.get(&(inode, offset)) {
            self.arr[idx].as_mut().unwrap().ref_bit = true;
            self.clock_hand = idx;
            self.arr[idx].as_ref().unwrap().data.clone()
        } else {
            if self.len < self.capacity {
                let block = self.read_below(below, inode, offset);
                let idx = self.len;
                self.arr[idx] = Some(block);
                self.lookup.insert((inode, offset), idx);
                self.arr[idx].as_mut().unwrap().ref_bit = true;
                self.arr[idx].as_mut().unwrap().is_dirty = false;
                self.clock_hand = idx;
                self.len += 1;
                self.arr[idx].as_ref().unwrap().data.clone()
            } else {
                let idx = self.clock_find();
                if let Some(block_to_evict) = self.arr[idx].take() {
                    if block_to_evict.is_dirty {
                        self.write_below(below, block_to_evict.inode, block_to_evict.offset, block_to_evict.data);
                    }
                }
                let block = self.read_below(below, inode, offset);
                self.arr[idx] = Some(block);
                self.lookup.insert((inode, offset), idx);
                self.arr[idx].as_mut().unwrap().ref_bit = true;
                self.clock_hand = idx;
                self.arr[idx].as_mut().unwrap().is_dirty = false;
                self.lookup.remove(&(block_to_evict.inode, block_to_evict.offset));
                self.arr[idx].as_ref().unwrap().data.clone()
            }
        }
    }

    fn write(&mut self, below: &mut Below, inode: i32, offset: i32, data: Vec<u8>) {
        if let Some(&idx) = self.lookup.get(&(inode, offset)) {
            self.arr[idx] = Some(Block {
                data,
                ref_bit: true,
                inode,
                offset,
                is_dirty: true,
            });
            self.clock_hand = idx;
        } else {
            if self.len < self.capacity {
                let block = Block {
                    data,
                    ref_bit: true,
                    inode,
                    offset,
                    is_dirty: true,
                };
                let idx = self.len;
                self.arr[idx] = Some(block);
                self.lookup.insert((inode, offset), idx);
                self.clock_hand = idx;
                self.len += 1;
            } else {
                let idx = self.clock_find();
                if let Some(block_to_evict) = self.arr[idx].take() {
                    if block_to_evict.is_dirty {
                        self.write_below(below, block_to_evict.inode, block_to_evict.offset, block_to_evict.data);
                    }
                }
                let block = Block {
                    data,
                    ref_bit: true,
                    inode,
                    offset,
                    is_dirty: true,
                };
                self.arr[idx] = Some(block);
                self.lookup.insert((inode, offset), idx);
                self.clock_hand = idx;
                self.lookup.remove(&(block_to_evict.inode, block_to_evict.offset));
            }
        }
    }

    fn below_get_size(&mut self, inode: i32) -> usize {
        0 // Replace with your implementation
    }

    fn get_size(&mut self, inode: i32) -> usize {
        self.below_get_size(inode)
    }

    fn below_set_size(&mut self, inode: i32, size: usize) {
        // Replace with your implementation
    }

    fn set_size(&mut self, inode: i32, size: usize) {
        self.below_set_size(inode, size);
    }

    fn synch(&mut self, below: &mut Below, inode: i32) {
        if inode == -1 {
            for block_idx in 0..self.len {
                if let Some(block) = self.arr[block_idx].take() {
                    if block.is_dirty {
                        self.write_below(below, block.inode, block.offset, block.data);
                    }
                }
            }
        } else {
            for block_idx in 0..self.len {
                if let Some(block) = self.arr[block_idx].take() {
                    if block.is_dirty && block.inode == inode {
                        self.write_below(below, block.inode, block.offset, block.data);
                    }
                }
            }
        }
    }
}
