[org 0x7C00]
bits 16
start:
    mov ax, start      ; imm16 encodes the absolute offset of “start”
; this is an example to test the org directive, if you decompile you see the magic
;
; ndisasm -b 16 src/build/stagesandbox.bin
; 00000000  B8007C            mov ax,0x7c00
;
; ndisasm -b 16 -o 0x7c00 src/build/stagesandbox.bin
; 00007C00  B8007C            mov ax,0x7c00
;                                    ^^^^^^ <------ The label start is at that position because the org diretive
