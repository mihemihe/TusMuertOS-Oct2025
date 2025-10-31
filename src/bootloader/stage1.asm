[org 0x7C00]

bits 16

msg db 'Printing bios!!!!', 0

cli
xor ax, ax
mov ds, ax
mov es, ax
mov ss, ax
mov sp, 0x7C00
sti

mov si, msg

.print_char:
    lodsb
    test al, al
    jz .halt
    mov ah, 0x0E
    mov bh, 0x00
    mov bl, 0x07
    int 0x10
    jmp .print_char

.halt:
    hlt
    jmp .halt

times 510-($-$$) db 0
dw 0xAA55