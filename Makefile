arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

crate_name := trashos
target ?= $(arch)-$(crate_name)
linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, build/arch/$(arch)/%.o, $(assembly_source_files))
kernel_lib := target/$(target)/debug/lib$(crate_name).a

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -serial stdio 

gdb: $(iso)
	@gdb "$(kernel)" \
		-ex "set arch $(arch)" \
		-ex "target remote | exec qemu-system-x86_64 -gdb stdio -cdrom $(iso) -smp 4 -S -no-shutdown -no-reboot"

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(assembly_object_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) $(assembly_object_files) $(kernel_lib)

kernel:
	@cargo build

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
