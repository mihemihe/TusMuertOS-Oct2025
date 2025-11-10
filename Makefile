ASM = nasm
DASM = ndisasm
ASM_BOOTLOADER_STAGE1 = src/bootloader/stage1.asm
BIN_BOOTLOADER_STAGE1 = src/build/stage1.bin
ASM_DISASM_STAGE1 = src/build/stage1disas.asm
IMG_BOOTLOADER = src/build/floppy.img

.PHONY: all $(IMG_BOOTLOADER) $(BIN_BOOTLOADER_STAGE1) $(ASM_DISASM_STAGE1)

all: $(IMG_BOOTLOADER) $(ASM_DISASM_STAGE1) $(ASM_DISASM_STAGE1) # Target is the final img file. Main goal



# 1st: Build binary from assembly file
$(BIN_BOOTLOADER_STAGE1): $(ASM_BOOTLOADER_STAGE1) # Target file depends on source asm file
	$(ASM) -f bin $(ASM_BOOTLOADER_STAGE1) -o $(BIN_BOOTLOADER_STAGE1)

# probably I need to add sme steps to create an empty floppy and delete old artifacts.
# Maybe add it as a second task that I can invoke in the command line

# 2nd: Inject binary into floppy. At the beginning
$(IMG_BOOTLOADER): $(BIN_BOOTLOADER_STAGE1)
	dd if=$(BIN_BOOTLOADER_STAGE1) of=$(IMG_BOOTLOADER) bs=512 count=1 conv=notrunc	
	
$(ASM_DISASM_STAGE1): $(BIN_BOOTLOADER_STAGE1)
	$(DASM) -b 16 $(BIN_BOOTLOADER_STAGE1)  > $(ASM_DISASM_STAGE1)