// #include "log_shift_rs.h" in file.c
// in makefile, LDFLAGS=-L$(LIBRARY_PATH), so
// $(CC) $(LDFLAGS) -o $@ $^ -lrusty_fs

#include "file.h"

block_no log_shift_r(block_no x, unsigned int nbits);