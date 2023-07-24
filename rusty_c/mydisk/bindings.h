#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define Block_BLOCK_SIZE (uintptr_t)BLOCK_SIZE

#define Metadata_SUGGESTED_ROW_WIDTH 4

inode_store_t *simplefs_init(inode_store_t *below, unsigned int below_ino, unsigned int num_inodes);

int simplefs_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes);
