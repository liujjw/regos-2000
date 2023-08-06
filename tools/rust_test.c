#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <assert.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <stdbool.h>

#include "disk.h"
#include "file.h"
// MARK where Rust code was imported
#include "bindings.h"


#define NINODE 3
char* contents[] = {
    "With only 2000 lines of code, egos-2000 implements all the basics.",
    "If debugging is the process of removing bugs, then programming must be the process of putting them in.",
    "The world is coming to an end... SAVE YOUR BUFFERS!"
};

char* multi_block_contents[] = {

};

#define DEBUG_SIZE 2048
// add one to the metadata offset for each (4 * NINODE) > 512 bytes block 
// each inode takes 4 bytes of metadata
#define METADATA_BLOCK_OFFSET 1 
int numblocks = DEBUG_SIZE / BLOCK_SIZE;

// 1 block for metadata, 1 block for each inode, 4 blocks total
char fs[DEBUG_SIZE];
char expected_metadata[] = {
    '\x01', '\x00', '\x00', '\x00',
    '\x01', '\x00', '\x00', '\x00',
    '\x01', '\x00', '\x00', '\x00'
}; 
#define NUM_METADATA_BYTES NINODE * 4

inode_intf ramdisk_init();
bool areArraysEqual(const char array1[], const char array2[], int size);

// modified from mkfs.c to ony write and check ram, not disk image
int main() {
    inode_intf ramdisk = ramdisk_init();
    assert(simplefs_create(ramdisk, 0, NINODE) >= 0);
    inode_intf mydisk = simplefs_init(ramdisk, 0, NINODE);
    fprintf(stderr, "[INFO] ramdisk address in c: %p\n", (void*) ramdisk);
    fprintf(stderr, "[INFO] ramdisk write address in c: %p\n", (void*) ramdisk->write);
    // exit(0);

    char buf[BLOCK_SIZE] = {0};
    for (int ino = 0; ino < NINODE; ino++) {
        fprintf(stderr, "[INFO] Loading ino=%d, %ld bytes\n", ino, strlen(contents[ino]));
        strncpy(buf, contents[ino], BLOCK_SIZE);
        char mybuf[BLOCK_SIZE] = {0};

        int blocks_per_inode = numblocks / NINODE;
        // fprintf(stderr, "[INFO] GOAT ino: %d, offset: %d, %s\n", ino, ino * blocks_per_inode, buf);
        
        mydisk->write(mydisk, ino, 0, (void*)buf);
        // ramdisk->write(ramdisk, ino, (ino * blocks_per_inode) + 0, (void*)buf);

        mydisk->read(mydisk, ino, 0, (void*)mybuf);
        // ramdisk->read(ramdisk, ino, (ino * blocks_per_inode) + METADATA_BLOCK_OFFSET, (void*)mybuf);

        fprintf(stderr, "[INFO] Checking ino=%d, has contents: %s\n, should match: %s\n", ino, mybuf, buf);
        // TEST write and read
        assert(strcmp(buf, mybuf) == 0);    
        fprintf(stderr, "[INFO] Success!\n");

    }
    // TEST metadata
    assert(areArraysEqual(expected_metadata, fs, NUM_METADATA_BYTES));
    free(ramdisk);
    free(mydisk);
    return 0;
}

bool areArraysEqual(const char array1[], const char array2[], int size) {
    for (int i = 0; i < size; i++) {
        if (array1[i] != array2[i]) {
            return false;
        }
    }
    return true;
}

int getsize() { return DEBUG_SIZE / BLOCK_SIZE; }

int setsize() { assert(0); }

int ramread(inode_intf bs, unsigned int ino, block_no offset, block_t *block) {
    memcpy(block, fs + offset * BLOCK_SIZE, BLOCK_SIZE);
    return 0;
}

int ramwrite(inode_intf bs, unsigned int ino, block_no offset, block_t *block) {
    // fprintf(stderr, "[INFO] ramwrite: fs + offset * BLOCK_SIZE: %p\n", (void*) (fs + offset * BLOCK_SIZE));

    memcpy(fs + offset * BLOCK_SIZE, block, BLOCK_SIZE);
    return 0;
}

inode_intf ramdisk_init() {
    inode_store_t *ramdisk = malloc(sizeof(*ramdisk));

    ramdisk->read = (void*)ramread;
    ramdisk->write = (void*)ramwrite;
    ramdisk->getsize = (void*)getsize;
    ramdisk->setsize = (void*)setsize;

    return ramdisk;
}

