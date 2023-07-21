#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

constexpr static const uintptr_t Block_BLOCK_SIZE = (uintptr_t)BLOCK_SIZE;

constexpr static const uintptr_t Metadata_CONSTANT_ROW_WIDTH = 4;

extern "C" {

inode_store_t *init(inode_store_t *below, unsigned int below_ino, unsigned int num_inodes);

int simplefs_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes);

} // extern "C"
