#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![register_tool(c2rust)]
#![feature(register_tool)]
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    static mut earth: *mut earth;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct earth {
    pub intr_enable: Option::<unsafe extern "C" fn() -> libc::c_int>,
    pub intr_register: Option::<
        unsafe extern "C" fn(
            Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
        ) -> libc::c_int,
    >,
    pub excp_register: Option::<
        unsafe extern "C" fn(
            Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
        ) -> libc::c_int,
    >,
    pub mmu_alloc: Option::<
        unsafe extern "C" fn(*mut libc::c_int, *mut *mut libc::c_void) -> libc::c_int,
    >,
    pub mmu_free: Option::<unsafe extern "C" fn(libc::c_int) -> libc::c_int>,
    pub mmu_map: Option::<
        unsafe extern "C" fn(libc::c_int, libc::c_int, libc::c_int) -> libc::c_int,
    >,
    pub mmu_switch: Option::<unsafe extern "C" fn(libc::c_int) -> libc::c_int>,
    pub mmu_translate: Option::<
        unsafe extern "C" fn(libc::c_int, libc::c_int) -> libc::c_int,
    >,
    pub disk_read: Option::<
        unsafe extern "C" fn(libc::c_int, libc::c_int, *mut libc::c_char) -> libc::c_int,
    >,
    pub disk_write: Option::<
        unsafe extern "C" fn(libc::c_int, libc::c_int, *mut libc::c_char) -> libc::c_int,
    >,
    pub tty_intr: Option::<unsafe extern "C" fn() -> libc::c_int>,
    pub tty_read: Option::<
        unsafe extern "C" fn(*mut libc::c_char, libc::c_int) -> libc::c_int,
    >,
    pub tty_write: Option::<
        unsafe extern "C" fn(*mut libc::c_char, libc::c_int) -> libc::c_int,
    >,
    pub tty_printf: Option::<
        unsafe extern "C" fn(*const libc::c_char, ...) -> libc::c_int,
    >,
    pub tty_info: Option::<
        unsafe extern "C" fn(*const libc::c_char, ...) -> libc::c_int,
    >,
    pub tty_fatal: Option::<
        unsafe extern "C" fn(*const libc::c_char, ...) -> libc::c_int,
    >,
    pub tty_success: Option::<
        unsafe extern "C" fn(*const libc::c_char, ...) -> libc::c_int,
    >,
    pub tty_critical: Option::<
        unsafe extern "C" fn(*const libc::c_char, ...) -> libc::c_int,
    >,
    pub platform: C2RustUnnamed_0,
    pub translation: C2RustUnnamed,
}
pub type C2RustUnnamed = libc::c_uint;
pub const SOFT_TLB: C2RustUnnamed = 1;
pub const PAGE_TABLE: C2RustUnnamed = 0;
pub type C2RustUnnamed_0 = libc::c_uint;
pub const ARTY: C2RustUnnamed_0 = 1;
pub const QEMU: C2RustUnnamed_0 = 0;
pub type block_no = libc::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct block {
    pub bytes: [libc::c_char; 512],
}
pub type block_t = block;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct inode_store {
    pub getsize: Option::<
        unsafe extern "C" fn(*mut inode_store, libc::c_uint) -> libc::c_int,
    >,
    pub setsize: Option::<
        unsafe extern "C" fn(*mut inode_store, libc::c_uint, block_no) -> libc::c_int,
    >,
    pub read: Option::<
        unsafe extern "C" fn(
            *mut inode_store,
            libc::c_uint,
            block_no,
            *mut block_t,
        ) -> libc::c_int,
    >,
    pub write: Option::<
        unsafe extern "C" fn(
            *mut inode_store,
            libc::c_uint,
            block_no,
            *mut block_t,
        ) -> libc::c_int,
    >,
    pub state: *mut libc::c_void,
}
pub type inode_store_t = inode_store;
pub type inode_intf = *mut inode_store_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct treedisk_state {
    pub below: *mut inode_store_t,
    pub below_ino: libc::c_uint,
    pub ninodes: libc::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct treedisk_snapshot {
    pub superblock: treedisk_block,
    pub inodeblock: treedisk_block,
    pub inode_blockno: block_no,
    pub inode: *mut treedisk_inode,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct treedisk_inode {
    pub root: block_no,
    pub nblocks: block_no,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union treedisk_block {
    pub datablock: block_t,
    pub superblock: treedisk_superblock,
    pub inodeblock: treedisk_inodeblock,
    pub freelistblock: treedisk_freelistblock,
    pub indirblock: treedisk_indirblock,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct treedisk_indirblock {
    pub refs: [block_no; 128],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct treedisk_freelistblock {
    pub refs: [block_no; 128],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct treedisk_inodeblock {
    pub inodes: [treedisk_inode; 64],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct treedisk_superblock {
    pub n_inodeblocks: block_no,
    pub free_list: block_no,
}
static mut log_rpb: libc::c_uint = 0;
static mut null_block: block_t = block_t { bytes: [0; 512] };
unsafe extern "C" fn panic(mut s: *const libc::c_char) {
    ((*earth).tty_fatal).expect("non-null function pointer")(s);
}
unsafe extern "C" fn log_shift_r(mut x: block_no, mut nbits: libc::c_uint) -> block_no {
    if nbits as libc::c_ulong
        >= (::std::mem::size_of::<block_no>() as libc::c_ulong)
            .wrapping_mul(8 as libc::c_int as libc::c_ulong)
    {
        return 0 as libc::c_int as block_no;
    }
    return x >> nbits;
}
unsafe extern "C" fn treedisk_get_snapshot(
    mut snapshot: *mut treedisk_snapshot,
    mut ts: *mut treedisk_state,
    mut inode_no: libc::c_uint,
) -> libc::c_int {
    if (Some(((*(*ts).below).read).expect("non-null function pointer")))
        .expect(
            "non-null function pointer",
        )(
        (*ts).below,
        (*ts).below_ino,
        0 as libc::c_int as block_no,
        &mut (*snapshot).superblock as *mut treedisk_block as *mut block_t,
    ) < 0 as libc::c_int
    {
        return -(1 as libc::c_int);
    }
    if inode_no as libc::c_ulong
        >= ((*snapshot).superblock.superblock.n_inodeblocks as libc::c_ulong)
            .wrapping_mul(
                (512 as libc::c_int as libc::c_ulong)
                    .wrapping_div(
                        ::std::mem::size_of::<treedisk_inode>() as libc::c_ulong,
                    ),
            )
    {
        ((*earth).tty_printf)
            .expect(
                "non-null function pointer",
            )(
            b"!!TDERR: inode number too large %u %u\n\0" as *const u8
                as *const libc::c_char,
            inode_no,
            (*snapshot).superblock.superblock.n_inodeblocks,
        );
        return -(1 as libc::c_int);
    }
    (*snapshot)
        .inode_blockno = (1 as libc::c_int as libc::c_ulong)
        .wrapping_add(
            (inode_no as libc::c_ulong)
                .wrapping_div(
                    (512 as libc::c_int as libc::c_ulong)
                        .wrapping_div(
                            ::std::mem::size_of::<treedisk_inode>() as libc::c_ulong,
                        ),
                ),
        ) as block_no;
    if (Some(((*(*ts).below).read).expect("non-null function pointer")))
        .expect(
            "non-null function pointer",
        )(
        (*ts).below,
        (*ts).below_ino,
        (*snapshot).inode_blockno,
        &mut (*snapshot).inodeblock as *mut treedisk_block as *mut block_t,
    ) < 0 as libc::c_int
    {
        return -(1 as libc::c_int);
    }
    let ref mut fresh0 = (*snapshot).inode;
    *fresh0 = &mut *((*snapshot).inodeblock.inodeblock.inodes)
        .as_mut_ptr()
        .offset(
            (inode_no as libc::c_ulong)
                .wrapping_rem(
                    (512 as libc::c_int as libc::c_ulong)
                        .wrapping_div(
                            ::std::mem::size_of::<treedisk_inode>() as libc::c_ulong,
                        ),
                ) as isize,
        ) as *mut treedisk_inode;
    return 0 as libc::c_int;
}
unsafe extern "C" fn treedisk_alloc_block(
    mut ts: *mut treedisk_state,
    mut snapshot: *mut treedisk_snapshot,
) -> block_no {
    let mut b: block_no = 0;
    static mut count: libc::c_int = 0;
    count += 1;
    b = (*snapshot).superblock.superblock.free_list;
    if b == 0 as libc::c_int as libc::c_uint {
        panic(
            b"treedisk_alloc_block: inode store is full\n\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut freelistblock: treedisk_block = treedisk_block {
        datablock: block_t { bytes: [0; 512] },
    };
    (Some(((*(*ts).below).read).expect("non-null function pointer")))
        .expect(
            "non-null function pointer",
        )(
        (*ts).below,
        (*ts).below_ino,
        b,
        &mut freelistblock as *mut treedisk_block as *mut block_t,
    );
    let mut i: libc::c_uint = 0;
    i = (512 as libc::c_int as libc::c_ulong)
        .wrapping_div(::std::mem::size_of::<block_no>() as libc::c_ulong)
        as libc::c_uint;
    loop {
        i = i.wrapping_sub(1);
        if !(i > 0 as libc::c_int as libc::c_uint) {
            break;
        }
        if freelistblock.freelistblock.refs[i as usize]
            != 0 as libc::c_int as libc::c_uint
        {
            break;
        }
    }
    let mut free_blockno: block_no = 0;
    if i == 0 as libc::c_int as libc::c_uint {
        free_blockno = b;
        (*snapshot)
            .superblock
            .superblock
            .free_list = freelistblock.freelistblock.refs[0 as libc::c_int as usize];
        if (Some(((*(*ts).below).write).expect("non-null function pointer")))
            .expect(
                "non-null function pointer",
            )(
            (*ts).below,
            (*ts).below_ino,
            0 as libc::c_int as block_no,
            &mut (*snapshot).superblock as *mut treedisk_block as *mut block_t,
        ) < 0 as libc::c_int
        {
            panic(
                b"treedisk_alloc_block: superblock\0" as *const u8 as *const libc::c_char,
            );
        }
    } else {
        free_blockno = freelistblock.freelistblock.refs[i as usize];
        freelistblock.freelistblock.refs[i as usize] = 0 as libc::c_int as block_no;
        if (Some(((*(*ts).below).write).expect("non-null function pointer")))
            .expect(
                "non-null function pointer",
            )(
            (*ts).below,
            (*ts).below_ino,
            b,
            &mut freelistblock as *mut treedisk_block as *mut block_t,
        ) < 0 as libc::c_int
        {
            panic(
                b"treedisk_alloc_block: freelistblock\0" as *const u8
                    as *const libc::c_char,
            );
        }
    }
    return free_blockno;
}
unsafe extern "C" fn treedisk_getsize(
    mut this_bs: *mut inode_store_t,
    mut ino: libc::c_uint,
) -> libc::c_int {
    let mut ts: *mut treedisk_state = (*this_bs).state as *mut treedisk_state;
    let mut snapshot: treedisk_snapshot = treedisk_snapshot {
        superblock: treedisk_block {
            datablock: block_t { bytes: [0; 512] },
        },
        inodeblock: treedisk_block {
            datablock: block_t { bytes: [0; 512] },
        },
        inode_blockno: 0,
        inode: 0 as *mut treedisk_inode,
    };
    if treedisk_get_snapshot(&mut snapshot, ts, ino) < 0 as libc::c_int {
        return -(1 as libc::c_int);
    }
    return (*snapshot.inode).nblocks as libc::c_int;
}
unsafe extern "C" fn treedisk_setsize(
    mut this_bs: *mut inode_store_t,
    mut ino: libc::c_uint,
    mut nblocks: block_no,
) -> libc::c_int {
    return -(1 as libc::c_int);
}
unsafe extern "C" fn treedisk_read(
    mut this_bs: *mut inode_store_t,
    mut ino: libc::c_uint,
    mut offset: block_no,
    mut block: *mut block_t,
) -> libc::c_int {
    let mut ts: *mut treedisk_state = (*this_bs).state as *mut treedisk_state;
    let mut snapshot: treedisk_snapshot = treedisk_snapshot {
        superblock: treedisk_block {
            datablock: block_t { bytes: [0; 512] },
        },
        inodeblock: treedisk_block {
            datablock: block_t { bytes: [0; 512] },
        },
        inode_blockno: 0,
        inode: 0 as *mut treedisk_inode,
    };
    if treedisk_get_snapshot(&mut snapshot, ts, ino) < 0 as libc::c_int {
        return -(1 as libc::c_int);
    }
    if offset >= (*snapshot.inode).nblocks {
        return -(1 as libc::c_int);
    }
    let mut nlevels: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    if (*snapshot.inode).nblocks > 0 as libc::c_int as libc::c_uint {
        while log_shift_r(
            ((*snapshot.inode).nblocks).wrapping_sub(1 as libc::c_int as libc::c_uint),
            nlevels.wrapping_mul(log_rpb),
        ) != 0 as libc::c_int as libc::c_uint
        {
            nlevels = nlevels.wrapping_add(1);
        }
    }
    let mut b: block_no = (*snapshot.inode).root;
    loop {
        if b == 0 as libc::c_int as libc::c_uint {
            memset(
                block as *mut libc::c_void,
                0 as libc::c_int,
                512 as libc::c_int as libc::c_ulong,
            );
            return 0 as libc::c_int;
        }
        let mut result: libc::c_int = (Some(
            ((*(*ts).below).read).expect("non-null function pointer"),
        ))
            .expect("non-null function pointer")((*ts).below, (*ts).below_ino, b, block);
        if result < 0 as libc::c_int {
            return result;
        }
        if nlevels == 0 as libc::c_int as libc::c_uint {
            return 0 as libc::c_int;
        }
        nlevels = nlevels.wrapping_sub(1);
        let mut tib: *mut treedisk_indirblock = block as *mut treedisk_indirblock;
        let mut index: libc::c_uint = (log_shift_r(offset, nlevels.wrapping_mul(log_rpb))
            as libc::c_ulong)
            .wrapping_rem(
                (512 as libc::c_int as libc::c_ulong)
                    .wrapping_div(::std::mem::size_of::<block_no>() as libc::c_ulong),
            ) as libc::c_uint;
        b = (*tib).refs[index as usize];
    };
}
unsafe extern "C" fn treedisk_write(
    mut this_bs: *mut inode_store_t,
    mut ino: libc::c_uint,
    mut offset: block_no,
    mut block: *mut block_t,
) -> libc::c_int {
    let mut ts: *mut treedisk_state = (*this_bs).state as *mut treedisk_state;
    let mut dirty_inode: libc::c_int = 0 as libc::c_int;
    let mut snapshot_buffer: treedisk_snapshot = treedisk_snapshot {
        superblock: treedisk_block {
            datablock: block_t { bytes: [0; 512] },
        },
        inodeblock: treedisk_block {
            datablock: block_t { bytes: [0; 512] },
        },
        inode_blockno: 0,
        inode: 0 as *mut treedisk_inode,
    };
    let mut snapshot: *mut treedisk_snapshot = &mut snapshot_buffer;
    if treedisk_get_snapshot(snapshot, ts, ino) < 0 as libc::c_int {
        return -(1 as libc::c_int);
    }
    let mut nlevels: libc::c_uint = 0 as libc::c_int as libc::c_uint;
    if (*(*snapshot).inode).nblocks > 0 as libc::c_int as libc::c_uint {
        while log_shift_r(
            ((*(*snapshot).inode).nblocks)
                .wrapping_sub(1 as libc::c_int as libc::c_uint),
            nlevels.wrapping_mul(log_rpb),
        ) != 0 as libc::c_int as libc::c_uint
        {
            nlevels = nlevels.wrapping_add(1);
        }
    }
    let mut nlevels_after: libc::c_uint = 0;
    if offset >= (*(*snapshot).inode).nblocks {
        (*(*snapshot).inode)
            .nblocks = offset.wrapping_add(1 as libc::c_int as libc::c_uint);
        dirty_inode = 1 as libc::c_int;
        nlevels_after = 0 as libc::c_int as libc::c_uint;
        while log_shift_r(offset, nlevels_after.wrapping_mul(log_rpb))
            != 0 as libc::c_int as libc::c_uint
        {
            nlevels_after = nlevels_after.wrapping_add(1);
        }
    } else {
        nlevels_after = nlevels;
    }
    if (*(*snapshot).inode).nblocks == 0 as libc::c_int as libc::c_uint {
        nlevels = nlevels_after;
    } else if nlevels_after > nlevels {
        while nlevels_after > nlevels {
            let mut indir: block_no = treedisk_alloc_block(ts, snapshot);
            let mut tib: treedisk_indirblock = treedisk_indirblock {
                refs: [0; 128],
            };
            memset(
                &mut tib as *mut treedisk_indirblock as *mut libc::c_void,
                0 as libc::c_int,
                512 as libc::c_int as libc::c_ulong,
            );
            tib.refs[0 as libc::c_int as usize] = (*(*snapshot).inode).root;
            (*(*snapshot).inode).root = indir;
            dirty_inode = 1 as libc::c_int;
            if (Some(((*(*ts).below).write).expect("non-null function pointer")))
                .expect(
                    "non-null function pointer",
                )(
                (*ts).below,
                (*ts).below_ino,
                indir,
                &mut tib as *mut treedisk_indirblock as *mut block_t,
            ) < 0 as libc::c_int
            {
                panic(
                    b"treedisk_write: indirect block\0" as *const u8
                        as *const libc::c_char,
                );
            }
            nlevels = nlevels.wrapping_add(1);
        }
    }
    if dirty_inode != 0 {
        if (Some(((*(*ts).below).write).expect("non-null function pointer")))
            .expect(
                "non-null function pointer",
            )(
            (*ts).below,
            (*ts).below_ino,
            (*snapshot).inode_blockno,
            &mut (*snapshot).inodeblock as *mut treedisk_block as *mut block_t,
        ) < 0 as libc::c_int
        {
            panic(b"treedisk_write: inode block\0" as *const u8 as *const libc::c_char);
        }
    }
    let mut b: block_no = 0;
    let mut parent_no: *mut block_no = &mut (*(*snapshot).inode).root;
    let mut parent_off: block_no = (*snapshot).inode_blockno;
    let mut parent_block: *mut block_t = &mut (*snapshot).inodeblock
        as *mut treedisk_block as *mut block_t;
    loop {
        let mut tib_0: treedisk_indirblock = treedisk_indirblock {
            refs: [0; 128],
        };
        b = *parent_no;
        if b == 0 as libc::c_int as libc::c_uint {
            *parent_no = treedisk_alloc_block(ts, snapshot);
            b = *parent_no;
            if (Some(((*(*ts).below).write).expect("non-null function pointer")))
                .expect(
                    "non-null function pointer",
                )((*ts).below, (*ts).below_ino, parent_off, parent_block)
                < 0 as libc::c_int
            {
                panic(b"treedisk_write: parent\0" as *const u8 as *const libc::c_char);
            }
            if nlevels == 0 as libc::c_int as libc::c_uint {
                break;
            }
            memset(
                &mut tib_0 as *mut treedisk_indirblock as *mut libc::c_void,
                0 as libc::c_int,
                512 as libc::c_int as libc::c_ulong,
            );
        } else {
            if nlevels == 0 as libc::c_int as libc::c_uint {
                break;
            }
            if (Some(((*(*ts).below).read).expect("non-null function pointer")))
                .expect(
                    "non-null function pointer",
                )(
                (*ts).below,
                (*ts).below_ino,
                b,
                &mut tib_0 as *mut treedisk_indirblock as *mut block_t,
            ) < 0 as libc::c_int
            {
                panic(b"treedisk_write\0" as *const u8 as *const libc::c_char);
            }
        }
        nlevels = nlevels.wrapping_sub(1);
        let mut index: libc::c_uint = (log_shift_r(offset, nlevels.wrapping_mul(log_rpb))
            as libc::c_ulong)
            .wrapping_rem(
                (512 as libc::c_int as libc::c_ulong)
                    .wrapping_div(::std::mem::size_of::<block_no>() as libc::c_ulong),
            ) as libc::c_uint;
        parent_no = &mut *(tib_0.refs).as_mut_ptr().offset(index as isize)
            as *mut block_no;
        parent_block = &mut tib_0 as *mut treedisk_indirblock as *mut block_t;
        parent_off = b;
    }
    if (Some(((*(*ts).below).write).expect("non-null function pointer")))
        .expect("non-null function pointer")((*ts).below, (*ts).below_ino, b, block)
        < 0 as libc::c_int
    {
        panic(b"treedisk_write: data block\0" as *const u8 as *const libc::c_char);
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn treedisk_init(
    mut below: *mut inode_store_t,
    mut below_ino: libc::c_uint,
) -> inode_intf {
    if log_rpb == 0 as libc::c_int as libc::c_uint {
        loop {
            log_rpb = log_rpb.wrapping_add(1);
            if !((512 as libc::c_int as libc::c_ulong)
                .wrapping_div(::std::mem::size_of::<block_no>() as libc::c_ulong)
                .wrapping_sub(1 as libc::c_int as libc::c_ulong) >> log_rpb
                != 0 as libc::c_int as libc::c_ulong)
            {
                break;
            }
        }
    }
    let mut ts: *mut treedisk_state = malloc(
        ::std::mem::size_of::<treedisk_state>() as libc::c_ulong,
    ) as *mut treedisk_state;
    memset(
        ts as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<treedisk_state>() as libc::c_ulong,
    );
    let ref mut fresh1 = (*ts).below;
    *fresh1 = below;
    (*ts).below_ino = below_ino;
    let mut this_bs: *mut inode_store_t = malloc(
        ::std::mem::size_of::<inode_store_t>() as libc::c_ulong,
    ) as *mut inode_store_t;
    memset(
        this_bs as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<inode_store_t>() as libc::c_ulong,
    );
    let ref mut fresh2 = (*this_bs).state;
    *fresh2 = ts as *mut libc::c_void;
    let ref mut fresh3 = (*this_bs).getsize;
    *fresh3 = Some(
        treedisk_getsize
            as unsafe extern "C" fn(*mut inode_store_t, libc::c_uint) -> libc::c_int,
    );
    let ref mut fresh4 = (*this_bs).setsize;
    *fresh4 = Some(
        treedisk_setsize
            as unsafe extern "C" fn(
                *mut inode_store_t,
                libc::c_uint,
                block_no,
            ) -> libc::c_int,
    );
    let ref mut fresh5 = (*this_bs).read;
    *fresh5 = Some(
        treedisk_read
            as unsafe extern "C" fn(
                *mut inode_store_t,
                libc::c_uint,
                block_no,
                *mut block_t,
            ) -> libc::c_int,
    );
    let ref mut fresh6 = (*this_bs).write;
    *fresh6 = Some(
        treedisk_write
            as unsafe extern "C" fn(
                *mut inode_store_t,
                libc::c_uint,
                block_no,
                *mut block_t,
            ) -> libc::c_int,
    );
    return this_bs;
}
#[no_mangle]
pub unsafe extern "C" fn setup_freelist(
    mut below: *mut inode_store_t,
    mut below_ino: libc::c_uint,
    mut next_free: block_no,
    mut nblocks: block_no,
) -> block_no {
    let mut freelist_data: [block_no; 128] = [0; 128];
    let mut freelist_block: block_no = 0 as libc::c_int as block_no;
    let mut i: libc::c_uint = 0;
    while next_free < nblocks {
        freelist_data[0 as libc::c_int as usize] = freelist_block;
        let fresh7 = next_free;
        next_free = next_free.wrapping_add(1);
        freelist_block = fresh7;
        i = 1 as libc::c_int as libc::c_uint;
        while (i as libc::c_ulong)
            < (512 as libc::c_int as libc::c_ulong)
                .wrapping_div(::std::mem::size_of::<block_no>() as libc::c_ulong)
            && next_free < nblocks
        {
            let fresh8 = next_free;
            next_free = next_free.wrapping_add(1);
            freelist_data[i as usize] = fresh8;
            i = i.wrapping_add(1);
        }
        while (i as libc::c_ulong)
            < (512 as libc::c_int as libc::c_ulong)
                .wrapping_div(::std::mem::size_of::<block_no>() as libc::c_ulong)
        {
            freelist_data[i as usize] = 0 as libc::c_int as block_no;
            i = i.wrapping_add(1);
        }
        if (Some(((*below).write).expect("non-null function pointer")))
            .expect(
                "non-null function pointer",
            )(
            below,
            below_ino,
            freelist_block,
            freelist_data.as_mut_ptr() as *mut block_t,
        ) < 0 as libc::c_int
        {
            panic(b"treedisk_setup_freelist\0" as *const u8 as *const libc::c_char);
        }
    }
    return freelist_block;
}
#[no_mangle]
pub unsafe extern "C" fn treedisk_create(
    mut below: *mut inode_store_t,
    mut below_ino: libc::c_uint,
    mut ninodes: libc::c_uint,
) -> libc::c_int {
    if ::std::mem::size_of::<treedisk_block>() as libc::c_ulong
        != 512 as libc::c_int as libc::c_ulong
    {
        panic(
            b"treedisk_create: block has wrong size\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut n_inodeblocks: libc::c_uint = (ninodes as libc::c_ulong)
        .wrapping_add(
            (512 as libc::c_int as libc::c_ulong)
                .wrapping_div(::std::mem::size_of::<treedisk_inode>() as libc::c_ulong),
        )
        .wrapping_sub(1 as libc::c_int as libc::c_ulong)
        .wrapping_div(
            (512 as libc::c_int as libc::c_ulong)
                .wrapping_div(::std::mem::size_of::<treedisk_inode>() as libc::c_ulong),
        ) as libc::c_uint;
    let mut nblocks: libc::c_uint = (Some(
        ((*below).getsize).expect("non-null function pointer"),
    ))
        .expect("non-null function pointer")(below, below_ino) as libc::c_uint;
    if nblocks < n_inodeblocks.wrapping_add(2 as libc::c_int as libc::c_uint) {
        ((*earth).tty_printf)
            .expect(
                "non-null function pointer",
            )(
            b"treedisk_create: too few blocks\n\0" as *const u8 as *const libc::c_char,
        );
        return -(1 as libc::c_int);
    }
    let mut superblock: treedisk_block = treedisk_block {
        datablock: block_t { bytes: [0; 512] },
    };
    if (Some(((*below).read).expect("non-null function pointer")))
        .expect(
            "non-null function pointer",
        )(
        below,
        below_ino,
        0 as libc::c_int as block_no,
        &mut superblock as *mut treedisk_block as *mut block_t,
    ) < 0 as libc::c_int
    {
        return -(1 as libc::c_int);
    }
    if superblock.superblock.n_inodeblocks == 0 as libc::c_int as libc::c_uint {
        let mut superblock_0: treedisk_block = treedisk_block {
            datablock: block_t { bytes: [0; 512] },
        };
        memset(
            &mut superblock_0 as *mut treedisk_block as *mut libc::c_void,
            0 as libc::c_int,
            512 as libc::c_int as libc::c_ulong,
        );
        superblock_0.superblock.n_inodeblocks = n_inodeblocks;
        superblock_0
            .superblock
            .free_list = setup_freelist(
            below,
            below_ino,
            n_inodeblocks.wrapping_add(1 as libc::c_int as libc::c_uint),
            nblocks,
        );
        if (Some(((*below).write).expect("non-null function pointer")))
            .expect(
                "non-null function pointer",
            )(
            below,
            below_ino,
            0 as libc::c_int as block_no,
            &mut superblock_0 as *mut treedisk_block as *mut block_t,
        ) < 0 as libc::c_int
        {
            return -(1 as libc::c_int);
        }
        let mut i: libc::c_int = 1 as libc::c_int;
        while i as libc::c_uint <= n_inodeblocks {
            if (Some(((*below).write).expect("non-null function pointer")))
                .expect(
                    "non-null function pointer",
                )(below, below_ino, i as block_no, &mut null_block) < 0 as libc::c_int
            {
                return -(1 as libc::c_int);
            }
            i += 1;
        }
    }
    return 0 as libc::c_int;
}
