[org 0x7C00]

bits 16


xchg bx, bx
cli
xor ax, ax
mov ds, ax
mov es, ax
mov ss, ax
mov sp, 0x7C00
sti
xchg bx, bx
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
    xchg bx, bx
    hlt
    jmp .halt

msg db 'ABCD', 0

times 510-($-$$) db 0
dw 0xAA55

; TODO:
; Validate that if I write on the stack, the first byte is not at  0x7C00  so does not overwrite the bootloader code.
; On that address the first bye of the string is written. check with debugger

; NOTES: 
; Moved the msg literal string to the bottom. If you place it at the top it counts as instructions if you load it as a plain binary
; This is what the linker and a proper executable format would handle for you.