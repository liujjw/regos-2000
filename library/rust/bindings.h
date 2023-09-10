/**
 * @file bindings.h
 * @author Jackie Liu (jl2627@cornell.edu)
 * @brief Auto-generated and then manually edited cbindgen bindings to the Rust interface.
 * @version 0.1
 * @date 2023-07-23
 * 
 * @copyright  (C) 2023, Cornell University
 * All rights reserved.
 */
// MARK where Rust bindings were added
#include "inode.h"

typedef unsigned int block_no;

inode_store_t *simplefs_init(inode_store_t *below, unsigned int below_ino, unsigned int num_inodes);

int simplefs_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes);

inode_store_t *fs_init(inode_store_t *below, unsigned int below_ino, unsigned int num_inodes);

int fs_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes);

inode_intf treedisk_init(inode_store_t *below, unsigned int below_ino);

int treedisk_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes);

block_no setup_freelist(inode_store_t *below,
                        unsigned int below_ino,
                        block_no next_free,
                        block_no nblocks);