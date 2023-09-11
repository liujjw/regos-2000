#define DEBUG_SIZE 4096
#define BLOCK_SIZE 512

int getsize() { return DEBUG_SIZE / BLOCK_SIZE; }
int setsize() { assert(0); }
int ramread(inode_intf bs, unsigned int ino, block_no offset, block_t *block) {
    memcpy(block, fs + offset * BLOCK_SIZE, BLOCK_SIZE);
    return 0;
}
int ramwrite(inode_intf bs, unsigned int ino, block_no offset, block_t *block) {
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