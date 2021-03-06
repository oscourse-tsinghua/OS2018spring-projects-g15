arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

os := rucore
target ?= $(arch)-$(os)
rust_os := target/$(target)/debug/lib$(os).a

boot_src := src/arch/$(arch)/boot
linker_script := $(boot_src)/linker.ld
grub_cfg := $(boot_src)/grub.cfg
assembly_source_files := $(wildcard $(boot_src)/*.asm)
assembly_object_files := $(patsubst $(boot_src)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))
user_object_files := $(wildcard user/*.o)

qemu_opts := -serial mon:stdio
features := use_apic

link_user = 1

ifdef link_user
features := $(features) link_user_program
assembly_object_files := $(assembly_object_files) $(user_object_files)
endif

ifdef travis
test := 1
features := $(features) qemu_auto_exit
endif

ifdef test
features := $(features) test
qemu_opts := $(qemu_opts) -device -isa-debug-exit
endif

# try to infer the correct QEMU
ifndef QEMU
QEMU := $(shell if which qemu-system-x86_64 > /dev/null; \
	then echo 'qemu-system-x86_64 -hda linuxppc.qcow2'; exit; \
	elif which x86_64-elf-qemu > /dev/null; \
	then echo 'x86_64-elf-qemu'; exit; \
	elif which qemu > /dev/null; \
	then echo 'qemu'; exit; \
	else \
	echo "***" 1>&2; \
	echo "*** Error: Couldn't find a working QEMU executable." 1>&2; \
	echo "*** Is the directory containing the qemu binary in your PATH" 1>&2; \
	echo "***" 1>&2; exit 1; fi)
endif

ifeq ($(OS),Windows_NT)
uname := Win32
else
uname := $(shell uname)
endif

ifeq ($(uname), Linux)
prefix :=
else
prefix := x86_64-elf-
endif

ld := $(prefix)ld
objdump := $(prefix)objdump
cc := $(prefix)gcc

.PHONY: all clean run iso kernel build debug_asm

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	$(QEMU) -cdrom $< $(qemu_opts) || [ $$? -eq 11 ]

iso: $(iso)

build: iso

debug_asm:
	@$(objdump) -dS $(kernel) | less

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@$(ld) -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

kernel:
	@RUST_TARGET_PATH=$(shell pwd) CC=$(cc) xargo build --target $(target) --features "$(features)"

# compile assembly files
build/arch/$(arch)/%.o: $(boot_src)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
