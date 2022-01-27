all: apps
	@echo "$(GREEN)-------- Compile the Grass Layer --------$(END)"
	$(RISCV_CC) $(CFLAGS) $(LDFLAGS) $(GRASS_LAYOUT) $(GRASS_SRCS) $(DEFAULT_LDLIBS) $(INCLUDE) -o $(RELEASE_DIR)/grass.elf
	$(OBJDUMP) --source --all-headers --demangle --line-numbers --wide $(RELEASE_DIR)/grass.elf > $(DEBUG_DIR)/grass.lst
	@echo "$(CYAN)-------- Compile the Earth Layer --------$(END)"
	$(RISCV_CC) $(CFLAGS) $(LDFLAGS) $(ARTY_FLAGS) $(EARTH_LAYOUT) $(EARTH_SRCS) $(EARTH_LDLIBS) $(INCLUDE) -o $(RELEASE_DIR)/earth.elf
	$(OBJDUMP) --source --all-headers --demangle --line-numbers --wide $(RELEASE_DIR)/earth.elf > $(DEBUG_DIR)/earth.lst

.PHONY: apps
apps: apps/*.c
	mkdir -p $(DEBUG_DIR) $(RELEASE_DIR)
	@echo "$(YELLOW)-------- Compile the Apps Layer --------$(END)"
	for FILE in $^ ; do \
	  export APP=$$(basename $${FILE} .c);\
	  echo "Compile" $${FILE} "=>" $(RELEASE_DIR)/$${APP}.elf;\
	  $(RISCV_CC) $(CFLAGS) $(LDFLAGS) $(APPS_LAYOUT) $(APPS_SRCS) $${FILE} $(DEFAULT_LDLIBS) $(INCLUDE) -o $(RELEASE_DIR)/$${APP}.elf;\
	  echo "Compile" $${FILE} "=>" $(DEBUG_DIR)/$${APP}.lst;\
	  $(OBJDUMP) --source --all-headers --demangle --line-numbers --wide $(RELEASE_DIR)/$${APP}.elf > $(DEBUG_DIR)/$${APP}.lst;\
	done

images:
	@echo "$(YELLOW)-------- Create the Disk Image --------$(END)"
	$(CC) $(BUILD_DIR)/mkfs.c -o $(BUILD_DIR)/mkfs
	cd $(BUILD_DIR); ./mkfs
	@echo "$(YELLOW)-------- Create the BootROM Image --------$(END)"
	@echo "[Note] Require vivado_lab in your \$$PATH. Otherwise, you can execute the tcl command in $(BUILD_DIR)/arty_board/write_cfgmem.tcl manually in Vivado (the input box at the bottom of hardware manager)."
	$(OBJCOPY) -O binary $(RELEASE_DIR)/earth.elf $(BUILD_DIR)/earth.bin
	$(CC) $(BUILD_DIR)/mkrom.c -o $(BUILD_DIR)/mkrom
	cd $(BUILD_DIR); ./mkrom
	#cd $(BUILD_DIR); vivado_lab -nojournal -mode batch -source arty_board/write_cfgmem.tcl; rm *.log *.prm
	#srec_info $(BUILD_DIR)/egos_bootROM.mcs -Intel
	#rm $(BUILD_DIR)/earth.bin

loc:
	cloc . --exclude-dir=$(BUILD_DIR)

clean:
	rm -rf $(DEBUG_DIR) $(RELEASE_DIR)
	rm -rf $(BUILD_DIR)/mkfs $(BUILD_DIR)/disk.img $(BUILD_DIR)/earth.bin $(BUILD_DIR)/egos_bootROM.* $(BUILD_DIR)/*.log

EARTH_SRCS = earth/*.c earth/sd/*.c shared/*.c
EARTH_LAYOUT = -Tearth/layout.lds

GRASS_SRCS = grass/enter.S grass/*.c shared/*.c
GRASS_LAYOUT = -Tgrass/layout.lds

APPS_SRCS = apps/enter.S shared/*.c
APPS_LAYOUT = -Tapps/layout.lds

RISCV_CC = riscv64-unknown-elf-gcc
OBJDUMP = riscv64-unknown-elf-objdump
OBJCOPY = riscv64-unknown-elf-objcopy

BUILD_DIR = install
DEBUG_DIR = $(BUILD_DIR)/debug
RELEASE_DIR = $(BUILD_DIR)/release

INCLUDE = -Ishared/include
CFLAGS = -march=rv32imac -mabi=ilp32 -mcmodel=medlow
CFLAGS += -ffunction-sections -fdata-sections

ARTY_FLAGS = -L$(BUILD_DIR)/arty_board -I$(BUILD_DIR)/arty_board
LDFLAGS = -Wl,--gc-sections -nostartfiles -nostdlib

DEFAULT_LDLIBS = -lc -lgcc
EARTH_LDLIBS = -Wl,--start-group -lc -lgcc -lm -lmetal -lmetal-gloss -Wl,--end-group

GREEN = \033[1;32m
YELLOW = \033[1;33m
CYAN = \033[1;36m
END = \033[0m
