
from collections import deque

hits = 0
faults = 0

class Block:
  def __init__(self, byte_arr: List[bytes]):
    self.byte_arr = byte_arr

class INode:
  def __init__(self, idx, blocks: List[Block]):
    self.idx = idx
    self.blocks = blocks
    self.is_dirty = False

class ClockCache:
  def __init__(self, capacity):
      self.q = deque()
      self.bitref = [False] * capacity
      self.count = 0
      self.capactiy = capacity

  def write(i):
    

  def read(i):
    pass

# Function to implement LRU Approximation,
# i.e. the clock algorithm or second chance algorithm
def LRU_Approximation(t, capacity):
  # t is the array of integers to be processed, which would represent pages in a real world scenario
  n = len(t)
  ca = Clock_Algorithm(capacity)

  for i in t:
    ca.write(i)

	print("Hits:", hits)
	print("Faults:", faults)


# Driver code, these are all writes, reads are analagous
t = [2, 3, 2, 1, 5, 2, 4, 5, 3, 2, 5, 2]
capacity = 4
LRU_Approximation(t, capacity)