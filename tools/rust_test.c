#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <assert.h>
#include <stdlib.h>
#include <sys/stat.h>

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
// add one for each (4 * NINODE) > 512 bytes block 
#define METADATA_BLOCK_OFFSET 1 
int numblocks = DEBUG_SIZE / BLOCK_SIZE;
char fs[DEBUG_SIZE];

inode_intf ramdisk_init();

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

        // TODO smth wrong with read, not memory related?
        // mydisk->read(mydisk, ino, 0, (void*)mybuf);
        ramdisk->read(ramdisk, ino, (ino * blocks_per_inode) + METADATA_BLOCK_OFFSET, (void*)mybuf);

        fprintf(stderr, "[INFO] Checking ino=%d, has contents: %s\n, should match: %s\n", ino, mybuf, buf);
        assert(strcmp(buf, mybuf) == 0);    
        fprintf(stderr, "[INFO] Success!\n");

    }
    free(ramdisk);
    return 0;
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

