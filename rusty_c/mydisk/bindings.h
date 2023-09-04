#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define Metadata_SUGGESTED_ROW_WIDTH 4

#define Block_BLOCK_SIZE (uintptr_t)BLOCK_SIZE

typedef unsigned int block_no;

typedef struct block {
  char bytes[512];
} block;

typedef struct block block_t;

typedef struct inode_store {
  int (*getsize)(struct inode_store*, unsigned int);
  int (*setsize)(struct inode_store*, unsigned int, block_no);
  int (*read)(struct inode_store*, unsigned int, block_no, block_t*);
  int (*write)(struct inode_store*, unsigned int, block_no, block_t*);
  void *state;
} inode_store;

typedef struct inode_store inode_store_t;

typedef inode_store_t *inode_intf;

typedef unsigned int C2RustUnnamed;

typedef unsigned int C2RustUnnamed_0;

#define SOFT_TLB 1

#define PAGE_TABLE 0

#define ARTY 1

#define QEMU 0

inode_store_t *simplefs_init(inode_store_t *below, unsigned int below_ino, unsigned int num_inodes);

int simplefs_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes);

extern void *malloc(unsigned long);

extern void *memset(void*, int, unsigned long);

inode_intf treedisk_init(inode_store_t *below, unsigned int below_ino);

block_no setup_freelist(inode_store_t *below,
                        unsigned int below_ino,
                        block_no next_free,
                        block_no nblocks);

int treedisk_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes);

extern void *malloc(size_t size);

extern void free(void *ptr);
