
#include <stdlib.h>
#include <string.h>

#ifdef MKFS
#include <stdio.h>
#else
// #include "egos.h"
#endif

// egos.h
struct earth {
    /* CPU interface */
    int (*intr_enable)();
    int (*intr_register)(void (*handler)(int));
    int (*excp_register)(void (*handler)(int));

    int (*mmu_alloc)(int* frame_no, void** cached_addr);
    int (*mmu_free)(int pid);
    int (*mmu_map)(int pid, int page_no, int frame_no);
    int (*mmu_switch)(int pid);
    int (*mmu_translate)(int pid, int page_no);

    /* Devices interface */
    int (*disk_read)(int block_no, int nblocks, char* dst);
    int (*disk_write)(int block_no, int nblocks, char* src);

    int (*tty_intr)();
    int (*tty_read)(char* buf, int len);
    int (*tty_write)(char* buf, int len);

    int (*tty_printf)(const char *format, ...);
    int (*tty_info)(const char *format, ...);
    int (*tty_fatal)(const char *format, ...);
    int (*tty_success)(const char *format, ...);
    int (*tty_critical)(const char *format, ...);

    /* Some information about earth layer configuration */
    enum { QEMU, ARTY } platform;
    enum { PAGE_TABLE, SOFT_TLB } translation;
};

struct grass {
    /* Shell environment variables */
    int workdir_ino;
    char workdir[128];

    /* Process control interface */
    int  (*proc_alloc)();
    void (*proc_free)(int pid);
    void (*proc_set_ready)(int pid);

    /* System call interface */
    void (*sys_exit)(int status);
    int  (*sys_send)(int pid, char* msg, int size);
    int  (*sys_recv)(int* pid, char* buf, int size);
};

extern struct earth *earth;
extern struct grass *grass;

/* Memory layout */
#define PAGE_SIZE          4096
#define FRAME_CACHE_END    0x80020000
#define FRAME_CACHE_START  0x80004000  /* 112KB  frame cache           */
                                       /*        earth interface       */
#define GRASS_STACK_TOP    0x80003f80  /* 8KB    earth/grass stack     */
                                       /*        grass interface       */
#define APPS_STACK_TOP     0x80002000  /* 6KB    app stack             */
#define SYSCALL_ARG        0x80000400  /* 1KB    system call args      */
#define APPS_ARG           0x80000000  /* 1KB    app main() argc, argv */
#define APPS_SIZE          0x00003000  
#define APPS_ENTRY         0x08005000  /* 12KB   app code+data         */
#define GRASS_SIZE         0x00002800
#define GRASS_ENTRY        0x08002800  /* 8KB    grass code+data       */
                                       /* 12KB   earth data            */
                                       /* earth code is in QSPI flash  */


#ifndef LIBC_STDIO
/* Only earth/dev_tty.c uses LIBC_STDIO and does not need these macros */
#define printf             earth->tty_printf
#define INFO               earth->tty_info
#define FATAL              earth->tty_fatal
#define SUCCESS            earth->tty_success
#define CRITICAL           earth->tty_critical
#endif

/* Memory-mapped I/O register access macros */
#define ACCESS(x) (*(__typeof__(*x) volatile *)(x))
#define REGW(base, offset) (ACCESS((unsigned int*)(base + offset)))
#define REGB(base, offset) (ACCESS((unsigned char*)(base + offset)))

// disk.h
#define BLOCK_SIZE            512
#define PAGING_DEV_SIZE       1024 * 1024
#define GRASS_NEXEC           8
#define GRASS_EXEC_SIZE       1024 * 1024
#define FS_DISK_SIZE          1024 * 1024 * 2
#define GRASS_EXEC_SEGMENT    (GRASS_EXEC_SIZE / GRASS_NEXEC / BLOCK_SIZE)

#define GRASS_EXEC_START      PAGING_DEV_SIZE / BLOCK_SIZE
#define SYS_PROC_EXEC_START   GRASS_EXEC_START + GRASS_EXEC_SEGMENT
#define SYS_FILE_EXEC_START   GRASS_EXEC_START + GRASS_EXEC_SEGMENT * 2
#define SYS_DIR_EXEC_START    GRASS_EXEC_START + GRASS_EXEC_SEGMENT * 3
#define SYS_SHELL_EXEC_START  GRASS_EXEC_START + GRASS_EXEC_SEGMENT * 4

#define GRASS_FS_START        (PAGING_DEV_SIZE + GRASS_EXEC_SIZE) / BLOCK_SIZE

typedef unsigned int block_no;      

typedef struct block {
    char bytes[BLOCK_SIZE];
} block_t;

// inode.h
#define NINODES  128

typedef struct inode_store {
    int (*getsize)(struct inode_store *this_bs, unsigned int ino);
    int (*setsize)(struct inode_store *this_bs, unsigned int ino, block_no newsize);
    int (*read)(struct inode_store *this_bs, unsigned int ino, block_no offset, block_t *block);
    int (*write)(struct inode_store *this_bs, unsigned int ino, block_no offset, block_t *block);
    void *state;
} inode_store_t;

typedef inode_store_t *inode_intf;    /* inode store interface */

inode_intf fs_disk_init();
inode_intf treedisk_init(inode_intf below, unsigned int below_ino);
int treedisk_create(inode_intf below, unsigned int below_ino, unsigned int ninodes);

// file.h
#define REFS_PER_BLOCK    (BLOCK_SIZE / sizeof(block_no))
#define INODES_PER_BLOCK  (BLOCK_SIZE / sizeof(struct treedisk_inode))

struct treedisk_superblock {
    block_no n_inodeblocks;		
    block_no free_list;			
};

struct treedisk_inode {
    block_no root;			
    block_no nblocks;			
};

struct treedisk_inodeblock {
    struct treedisk_inode inodes[INODES_PER_BLOCK];
};

struct treedisk_freelistblock {
    block_no refs[REFS_PER_BLOCK];
};

struct treedisk_indirblock {
    block_no refs[REFS_PER_BLOCK];
};

union treedisk_block {
    block_t datablock;
    struct treedisk_superblock superblock;
    struct treedisk_inodeblock inodeblock;
    struct treedisk_freelistblock freelistblock;
    struct treedisk_indirblock indirblock;
};

// original file.c
struct treedisk_snapshot {
    union treedisk_block superblock; 
    union treedisk_block inodeblock; 
    block_no inode_blockno;
    struct treedisk_inode *inode;
};

struct treedisk_state {
    inode_store_t *below;			/* inode store below */
    unsigned int below_ino;			/* inode number to use for the inode store below */
    unsigned int ninodes;			/* number of inodes in the treedisk */
};

static unsigned int log_rpb;                    /* log2(REFS_PER_BLOCK) */
static block_t null_block;			/* a block filled with null bytes */

static void panic(const char *s){
#ifdef MKFS
    fprintf(stderr, "%s", s);
    exit(1);
#else 
    FATAL(s);
#endif
}

static block_no log_shift_r(block_no x, unsigned int nbits){
    if (nbits >= sizeof(block_no) * 8) {
        return 0;
    }
    return x >> nbits;
}

static int treedisk_get_snapshot(struct treedisk_snapshot *snapshot,
                                 struct treedisk_state *ts, unsigned int inode_no){
    if ((*ts->below->read)(ts->below, ts->below_ino, 0, (block_t *) &snapshot->superblock) < 0)
        return -1;

    if (inode_no >= snapshot->superblock.superblock.n_inodeblocks * INODES_PER_BLOCK) {
        printf("!!TDERR: inode number too large %u %u\n", inode_no, snapshot->superblock.superblock.n_inodeblocks);
        return -1;
    }

    snapshot->inode_blockno = 1 + inode_no / INODES_PER_BLOCK;
    if ((*ts->below->read)(ts->below, ts->below_ino, snapshot->inode_blockno, (block_t *) &snapshot->inodeblock) < 0)
        return -1;

    snapshot->inode = &snapshot->inodeblock.inodeblock.inodes[inode_no % INODES_PER_BLOCK];
    return 0;
}

static block_no treedisk_alloc_block(struct treedisk_state *ts, struct treedisk_snapshot *snapshot){
    block_no b;
    static int count;
    count++;

    if ((b = snapshot->superblock.superblock.free_list) == 0)
        panic("treedisk_alloc_block: inode store is full\n");

    union treedisk_block freelistblock;
    (*ts->below->read)(ts->below, ts->below_ino, b, (block_t *) &freelistblock);

    unsigned int i;
    for (i = REFS_PER_BLOCK; --i > 0;)
        if (freelistblock.freelistblock.refs[i] != 0) {
            break;
        }

    block_no free_blockno;
    if (i == 0) {
        free_blockno = b;
        snapshot->superblock.superblock.free_list = freelistblock.freelistblock.refs[0];
        if ((*ts->below->write)(ts->below, ts->below_ino, 0, (block_t *) &snapshot->superblock) < 0) {
            panic("treedisk_alloc_block: superblock");
        }
    }
    else {
        free_blockno = freelistblock.freelistblock.refs[i];
        freelistblock.freelistblock.refs[i] = 0;
        if ((*ts->below->write)(ts->below, ts->below_ino, b, (block_t *) &freelistblock) < 0) {
            panic("treedisk_alloc_block: freelistblock");
        }
    }

    return free_blockno;
}

static int treedisk_getsize(inode_store_t *this_bs, unsigned int ino){
    struct treedisk_state *ts = this_bs->state;
    struct treedisk_snapshot snapshot;
    if (treedisk_get_snapshot(&snapshot, ts, ino) < 0)
        return -1;

    return snapshot.inode->nblocks; 
}

static int treedisk_setsize(inode_store_t *this_bs, unsigned int ino, block_no nblocks){
    return -1;
}

static int treedisk_read(inode_store_t *this_bs, unsigned int ino, block_no offset, block_t *block){
    struct treedisk_state *ts = this_bs->state;

    struct treedisk_snapshot snapshot;
    if (treedisk_get_snapshot(&snapshot, ts, ino) < 0)
        return -1;

    if (offset >= snapshot.inode->nblocks) {
        return -1;
    }

    unsigned int nlevels = 0;
    if (snapshot.inode->nblocks > 0)
        while (log_shift_r(snapshot.inode->nblocks - 1, nlevels * log_rpb) != 0) {
            nlevels++;
        }

    block_no b = snapshot.inode->root;
    for (;;) {
        if (b == 0) {
            memset(block, 0, BLOCK_SIZE);
            return 0;
        }

        int result = (*ts->below->read)(ts->below, ts->below_ino, b, block);
        if (result < 0)
            return result;
        if (nlevels == 0)
            return 0;

        nlevels--;
        struct treedisk_indirblock *tib = (struct treedisk_indirblock *) block;
        unsigned int index = log_shift_r(offset, nlevels * log_rpb) % REFS_PER_BLOCK;
        b = tib->refs[index];
    }
    return 0;
}

static int treedisk_write(inode_store_t *this_bs, unsigned int ino, block_no offset, block_t *block){
    struct treedisk_state *ts = this_bs->state;
    int dirty_inode = 0;

    struct treedisk_snapshot snapshot_buffer;
    struct treedisk_snapshot *snapshot = &snapshot_buffer;
    if (treedisk_get_snapshot(snapshot, ts, ino) < 0)
        return -1;

    unsigned int nlevels = 0;
    if (snapshot->inode->nblocks > 0)
        while (log_shift_r(snapshot->inode->nblocks - 1, nlevels * log_rpb) != 0) {
            nlevels++;
        }

    unsigned int nlevels_after;
    if (offset >= snapshot->inode->nblocks) {
        snapshot->inode->nblocks = offset + 1;
        dirty_inode = 1;
        nlevels_after = 0;
        while (log_shift_r(offset, nlevels_after * log_rpb) != 0) {
            nlevels_after++;
        }
    }
    else {
        nlevels_after = nlevels;
    }

    if (snapshot->inode->nblocks == 0) {
        nlevels = nlevels_after;
    } else if (nlevels_after > nlevels) {
        while (nlevels_after > nlevels) {
            block_no indir = treedisk_alloc_block(ts, snapshot);

            struct treedisk_indirblock tib;
            memset(&tib, 0, BLOCK_SIZE);
            tib.refs[0] = snapshot->inode->root;
            snapshot->inode->root = indir;
            dirty_inode = 1;
            if ((*ts->below->write)(ts->below, ts->below_ino, indir, (block_t *) &tib) < 0) {
                panic("treedisk_write: indirect block");
            }

            nlevels++;
        }
    }

    if (dirty_inode)
        if ((*ts->below->write)(ts->below, ts->below_ino, snapshot->inode_blockno, (block_t *) &snapshot->inodeblock) < 0) {
            panic("treedisk_write: inode block");
        }

    block_no b;
    block_no *parent_no = &snapshot->inode->root;
    block_no parent_off = snapshot->inode_blockno;
    block_t *parent_block = (block_t *) &snapshot->inodeblock;
    for (;;) {
        struct treedisk_indirblock tib;
        if ((b = *parent_no) == 0) {
            b = *parent_no = treedisk_alloc_block(ts, snapshot);
            if ((*ts->below->write)(ts->below, ts->below_ino, parent_off, parent_block) < 0)
                panic("treedisk_write: parent");
            if (nlevels == 0)
                break;
            memset(&tib, 0, BLOCK_SIZE);
        }
        else {
            if (nlevels == 0)
                break;
            if ((*ts->below->read)(ts->below, ts->below_ino, b, (block_t *) &tib) < 0)
                panic("treedisk_write");
        }

        nlevels--;
        unsigned int index = log_shift_r(offset, nlevels * log_rpb) % REFS_PER_BLOCK;
        parent_no = &tib.refs[index];
        parent_block = (block_t *) &tib;
        parent_off = b;
    }

    if ((*ts->below->write)(ts->below, ts->below_ino, b, block) < 0)
        panic("treedisk_write: data block");
    return 0;
}


inode_store_t *treedisk_init(inode_store_t *below, unsigned int below_ino){
    if (log_rpb == 0) {		/* first time only */
        do {
            log_rpb++;
        } while (((REFS_PER_BLOCK - 1) >> log_rpb) != 0);
    }

    struct treedisk_state *ts = malloc(sizeof(struct treedisk_state));
    memset(ts, 0, sizeof(struct treedisk_state));
    ts->below = below;
    ts->below_ino = below_ino;

    inode_store_t *this_bs = malloc(sizeof(inode_store_t));
    memset(this_bs, 0, sizeof(inode_store_t));
    this_bs->state = ts;
    this_bs->getsize = treedisk_getsize;
    this_bs->setsize = treedisk_setsize;
    this_bs->read = treedisk_read;
    this_bs->write = treedisk_write;
    return this_bs;
}


block_no setup_freelist(inode_store_t *below, unsigned int below_ino, block_no next_free, block_no nblocks){
    block_no freelist_data[REFS_PER_BLOCK];
    block_no freelist_block = 0;
    unsigned int i;

    while (next_free < nblocks) {
        freelist_data[0] = freelist_block;
        freelist_block = next_free++;
        for (i = 1; i < REFS_PER_BLOCK && next_free < nblocks; i++)
            freelist_data[i] = next_free++;

        for (; i < REFS_PER_BLOCK; i++)
            freelist_data[i] = 0;

        if ((*below->write)(below, below_ino, freelist_block, (block_t *) freelist_data) < 0)
            panic("treedisk_setup_freelist");
    }
    return freelist_block;
}

int treedisk_create(inode_store_t *below, unsigned int below_ino, unsigned int ninodes){
    if (sizeof(union treedisk_block) != BLOCK_SIZE)
        panic("treedisk_create: block has wrong size");

    unsigned int n_inodeblocks = (ninodes + INODES_PER_BLOCK - 1) / INODES_PER_BLOCK;

    unsigned int nblocks = (*below->getsize)(below, below_ino);
    if (nblocks < n_inodeblocks + 2) {
        printf("treedisk_create: too few blocks\n");
        return -1;
    }

    union treedisk_block superblock;
    if ((*below->read)(below, below_ino, 0, (block_t *) &superblock) < 0)
        return -1;

    if (superblock.superblock.n_inodeblocks == 0) {
        union treedisk_block superblock;
        memset(&superblock, 0, BLOCK_SIZE);
        superblock.superblock.n_inodeblocks = n_inodeblocks;
        superblock.superblock.free_list =
            setup_freelist(below, below_ino, n_inodeblocks + 1, nblocks);
        if ((*below->write)(below, below_ino, 0, (block_t *) &superblock) < 0)
            return -1;

        for (int i = 1; i <= n_inodeblocks; i++)
            if ((*below->write)(below, below_ino, i, &null_block) < 0) {
                return -1;
            }

    }
    else {
    }

    return 0;
}
