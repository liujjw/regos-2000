class Block:
  def __init__(self, data: list[bytes], ref_bit: bool, inode, offset, is_dirty):
    self.data = data 
    self.ref_bit = ref_bit
    self.inode = inode
    self.offset = offset
    self.is_dirty = is_dirty

class ClockCache:
  def __init__(self, capacity):
    self.capacity = capacity
    self.len = 0
    self.arr = [None] * capacity
    self.clock_hand = 0
    # lookup[(node, offset)] = idx
    self.lookup = {}

  def clock_find(self) -> int:
    """Find the next block to evict using the clock algorithm
    Assumes that the cache is full."""
    if self.arr[self.clock_hand].ref_bit:
      self.arr[self.clock_hand].ref_bit = False
      self.clock_hand = (self.clock_hand + 1) % self.capacity
      return self.clock_find()
    else:
      return self.clock_hand

  def read_below(self, inode, offset) -> list[bytes]:
    return Block([b'0x0'] * 512, True, inode, offset, False)
  
  def write_below(self, inode, offset, data: list[bytes]):
    pass

  def read(self, inode, offset) -> list[bytes]:
    if (inode, offset) in self.lookup:
      idx = self.lookup[(inode, offset)]
      self.arr[idx].ref_bit = True
      self.clock_hand = idx
      return self.arr[idx].data
    else:
      if self.len < self.capacity:
        self.arr[self.len] = self.read_below(inode, offset)
        self.lookup[(inode, offset)] = self.len
        self.arr[self.len].ref_bit = True
        self.arr[self.len].is_dirty = False
        self.clock_hand = self.len
        self.len += 1
        return self.arr[self.len - 1].data
      else:
        idx = self.clock_find()
        block_to_evict = self.arr[idx]
        if block_to_evict.is_dirty:
          self.write_below(block_to_evict.inode, block_to_evict.offset, block_to_evict.data)
        self.arr[idx] = self.read_below(inode, offset)
        self.lookup[(inode, offset)] = idx
        self.arr[idx].ref_bit = True
        self.clock_hand = idx
        self.arr[idx].is_dirty = False
        del self.lookup[(block_to_evict.inode, block_to_evict.offset)]
        return self.arr[idx].data
      
  def write(self, inode, offset, data):
    if (inode, offset) in self.lookup:
      idx = self.lookup[(inode, offset)]
      self.arr[idx] = Block(data, True, inode, offset, True)
      self.clock_hand = idx
    else:
      if self.len < self.capacity:
        self.arr[self.len] = Block(data, True, inode, offset, True)
        self.lookup[(inode, offset)] = self.len
        self.clock_hand = self.len
        self.len += 1
      else:
        idx = self.clock_find()
        block_to_evict = self.arr[idx]
        if block_to_evict.is_dirty:
          self.write_below(block_to_evict.inode, block_to_evict.offset, block_to_evict.data)
        self.arr[idx] = Block(data, True, inode, offset, True)
        self.lookup[(inode, offset)] = idx
        self.clock_hand = idx
        del self.lookup[(block_to_evict.inode, block_to_evict.offset)]