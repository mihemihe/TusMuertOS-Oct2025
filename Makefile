ASM = nasm
DASM = ndisasm

ASM_BOOTLOADER_STAGE1 = src/bootloader/stage1.asm
BIN_BOOTLOADER_STAGE1 = src/build/stage1.bin
ASM_DISASM_STAGE1     = src/build/stage1disas.asm
IMG_BOOTLOADER        = src/build/floppy.img

.PHONY: all force

all: $(BIN_BOOTLOADER_STAGE1) $(IMG_BOOTLOADER) $(ASM_DISASM_STAGE1)

# Always rebuild binary
$(BIN_BOOTLOADER_STAGE1): force
	$(ASM) -f bin $(ASM_BOOTLOADER_STAGE1) -o $(BIN_BOOTLOADER_STAGE1)

# Always update floppy image
$(IMG_BOOTLOADER): $(BIN_BOOTLOADER_STAGE1) force
	dd if=$(BIN_BOOTLOADER_STAGE1) of=$(IMG_BOOTLOADER) bs=1014 count=1 conv=notrunc

# Always regenerate disassembly
$(ASM_DISASM_STAGE1): $(BIN_BOOTLOADER_STAGE1) force
	$(DASM) -b 16 $(BIN_BOOTLOADER_STAGE1) > $(ASM_DISASM_STAGE1)

force: