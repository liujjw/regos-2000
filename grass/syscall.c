/*
 * (C) 2022, Cornell University
 * All rights reserved.
 */

/* Author: Yunhao Zhang
 * Description: the system call interface to user applications
 */

#include "egos.h"
#include "syscall.h"
#include <string.h>

static struct syscall *sc = (struct syscall*)SYSCALL_ARG;

static void sys_invoke() {
    /* The standard way of system call is using the `ecall` instruction; 
     * Switching to ecall is given to students as an exercise */
    *((int*)0x2000000) = 1;
    while (sc->type != SYS_UNUSED);
}

int sys_yield() {
    // invoke scheduler through a software interrupt/syscall
    // use machine software interrupt over supervisor software interrupt
    // use ssip (supervisor software interrupt pending) over usip bit
    // section 3.1.4 in riscv manual

    // int mip_var;
    // asm("csrr %0, mip" : "=r"(mip_var));
    // asm("csrw mip, %0" :: "r"(mip_var | (0x1 << 1)));
    sc->type = SYS_YIELD;
    sys_invoke();
    return sc->retval;
}

int sys_send(int receiver, char* msg, int size) {
    if (size > SYSCALL_MSG_LEN) return -1;

    sc->type = SYS_SEND;
    sc->msg.receiver = receiver;
    memcpy(sc->msg.content, msg, size);
    sys_invoke();
    return sc->retval;    
}

int sys_recv(int* sender, char* buf, int size) {
    if (size > SYSCALL_MSG_LEN) return -1;

    sc->type = SYS_RECV;
    sys_invoke();
    memcpy(buf, sc->msg.content, size);
    if (sender) *sender = sc->msg.sender;
    return sc->retval;
}

void sys_exit(int status) {
    struct proc_request req;
    req.type = PROC_EXIT;
    sys_send(GPID_PROCESS, (void*)&req, sizeof(req));
}
