import random 
import string

num_inodes = 100
max_offset = 10
num_operations = 100
num_bytes = 16

class DictFS:
  def __init__(self, num_inodes):
    # mp of ino to a list of 512 byte blocks
    self.mp = {}
    self.num_inodes = num_inodes

  def read(self, ino, offset):
    if ino not in self.mp:
      return None
    if offset >= len(self.mp[ino]):
      return None
    return self.mp[ino][offset]
  
  def read_all(self, ino):
    if ino not in self.mp:
      return None
    return self.mp[ino]
  
  def write(self, ino, offset, data):
    if ino not in self.mp:
      arr = [b'\x00'] * offset
      arr.append(data)
      self.mp[ino] = arr
    else:
      arr = self.mp[ino]
      if offset >= len(arr):  
        arr.extend([b'\x00'] * (offset - len(arr)))
        arr.append(data)
      else:
        arr[offset] = data


def encode(input_string, desired_length):
  input_bytes = input_string.encode("utf-8")  

  if len(input_bytes) < desired_length:
      padding_length = desired_length - len(input_bytes)
      padded_bytes = input_bytes + b'\x00' * padding_length
  else:
      padded_bytes = input_bytes

  return padded_bytes


def gen_ops():
  arr = []
  for i in range(num_operations):
    ar = ["write", random.randint(0, num_inodes), random.randint(0, max_offset)]
    arr.append(ar)
  return arr


def generate_random_string(length):
    characters = string.ascii_letters + string.digits  
    return ''.join(random.choice(characters) for _ in range(length))


def driver():
  res = []
  ops = gen_ops()
  fs = DictFS(num_inodes)
  for op in ops:
    if op[0] == "write":
      inode, offset = op[1], op[2]
      data = encode(generate_random_string(random.randint(num_bytes // 2, num_bytes)), num_bytes)
      fs.write(inode, offset, data)
      res.append(fs.read_all(inode))      
  print(res)

driver()