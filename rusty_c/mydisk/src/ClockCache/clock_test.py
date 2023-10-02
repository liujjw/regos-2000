
from clock import ClockCache

import pytest

class TestClockCache:

    # read and write data to cache
    def test_read_write_data(self):
      cache = ClockCache(10)
      inode = 1
      offset = 0
      data = [b'0x1'] * 512
  
      # Write data to cache
      cache.write(inode, offset, data)
  
      # Read data from cache
      result = cache.read(inode, offset)
  
      assert result == data

    # read and write data to cache with multiple blocks
    def test_read_write_data_multiple_blocks(self):
      cache = ClockCache(10)
      inode = 1
      offset1 = 0
      offset2 = 1
      data1 = [b'0x1'] * 512
      data2 = [b'0x2'] * 512
  
      # Write data to cache
      cache.write(inode, offset1, data1)
      cache.write(inode, offset2, data2)
  
      # Read data from cache
      result1 = cache.read(inode, offset1)
      result2 = cache.read(inode, offset2)
  
      assert result1 == data1
      assert result2 == data2

    # read and write data to cache with full capacity
    def test_read_write_data_full_capacity(self):
      cache = ClockCache(2)
      inode1 = 1
      inode2 = 2
      offset = 0
      data1 = [b'0x1'] * 512
      data2 = [b'0x2'] * 512
  
      # Write data to cache
      cache.write(inode1, offset, data1)
      cache.write(inode2, offset, data2)
  
      # Read data from cache
      result1 = cache.read(inode1, offset)
      result2 = cache.read(inode2, offset)
  
      assert result1 == data1
      assert result2 == data2

    # read and write data to cache with empty cache
    def test_read_write_data_empty_cache(self):
      cache = ClockCache(10)
      inode = 1
      offset = 0
      data = [b'0x1'] * 512
  
      # Read data from empty cache
      result = cache.read(inode, offset)
  
      assert result == data

    # read and write data to cache with non-existent data
    def test_read_write_data_non_existent_data(self):
      cache = ClockCache(10)
      inode1 = 1
      inode2 = 2
      offset = 0
      data1 = [b'0x1'] * 512
  
      # Write data to cache
      cache.write(inode1, offset, data1)
  
      # Read non-existent data from cache
      result = cache.read(inode2, offset)
  
      assert result == data1

    # read and write data to cache with invalid inode and offset
    def test_read_write_data_invalid_inode_offset(self):
      cache = ClockCache(10)
      inode = 1
      offset = 0
      data = [b'0x1'] * 512
  
      # Write data to cache
      cache.write(inode, offset, data)
  
      # Read data with invalid inode and offset from cache
      result = cache.read(inode + 1, offset + 1)
      assert result == data