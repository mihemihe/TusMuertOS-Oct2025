[org 0x0600]
;section stage1 start=0x7C00 changed to support relocate
byte_zero_plus_org: ; to calculate later instead of the ugly $-$$


bits 16
xchg bx, bx
mov sp, 0x7C00 ;REMOVE THIS IS DUCPLIACTE DD @@@ @#!@#!@#@!#

mov ax, 0x0000
mov ah, 0x80
mov ax, 0x00
mov al, 0x80
mov ax, 0x00
mov ax, 0x80
mov ax, 0x0000
mov si, 0x0000

xor eax, eax
xor ebx, ebx
xor ecx, ecx
xor edx, edx


mov ax, 0x15FA    
mov si, ax ; the register to print in hex
mov cl, 12
decrease_shift_bits_loop:
    cmp cl, 0
    js .exit_sub
    mov dx, si    
    shr dx, cl
    and dx, 0x000F
    cmp dl, 10
    jge .l1
    add dl, 48
    mov al, dl
    jmp .l2

    .l1:
    add dl, 55
    mov al, dl

    .l2:
        mov ah, 0x0E ; Write text in teletype mode
        mov bh, 0x00 ; page number, text modes
        mov bl, 0x07 ; light grey on black
        int 0x10
        sub cl, 4
        jmp decrease_shift_bits_loop
.exit_sub:





;
; / 0x10 moves 4 bits 
;












;xor eax, eax
;xor ebx, ebx
;xor ecx, ecx
;xor edx, edx
; mov cs, eax WRONG
;mov ss, ax
;mov ds, ax
;mov es, ax
;mov fs, ax
;mov gs, ax
;mov edi, eax
;mov esi, eax

; Initialize segments and registers to a sane state

cli             ; Disable interrupts
xor ax, ax      ; set AX to 0
mov ds, ax      ; set Data Segment to 0 
mov es, ax      ; set Extra segment to 0. Destination segment
mov ss, ax      ; set Stack sgment to 0.
mov sp, 0x7C00  ; Safe stack location. A push will not overlap and won't overwrite the first byte of bootloader
sti



relocate_mbr:
    ; No calling conventions. Not assuming AX, DS and ES = 0
    ; we dont move the stack, it is at 0x7c00, so careful later if we load a large kernel on low memory TODO
    cli ; Changing segments
    mov ax, 0
    mov ds, ax ; source data segment
    mov es, ax ; target extra segment
    sti
    mov si, 0x7C00
    mov di, 0x0600
    mov cx, 0x200 ; 512 bytes to the counter
    rep movsb

    ;[org 0x0600] ; 4th 512 bytes sector. 0x500 I guess is kinda safe too, or there be dragons. WE CANT DO THIS! maybe with sections
    ; track the CS:IP here
    ;jmp gate_a20 ; This does not work because it is a short offset jump ORG + offset
    ;jmp 0x0000:0x0626 ; Old, hardcoded way
    jmp 0x0000:gate_a20 ; the offset is the label relative offset + ORG
;section .after_relocate follows=stage1 vstart=0x0600
;after_relocate_entry:
; Validate a20 enabled
; 
gate_a20:
    ; No calling convention. 
    ; We prepare to pointers to HIGH and LOW memory, exactly 1MiB, 0000:0x0500 - FFFF:0x510 (the 10 difference means nothing) 
    ; We preserve what is in those high and low pointers via stack
    ; We save 0x00 in LOW pointer > We try to inject 0xFF in HIGH pointer. 
    ; If A20 enabled, no wrapping, 0xFF goes to HIGH, of a20 disabled, wrapping 1MiB 0xFF goes to low. 
    ; We compare 0xFF with LOW, and ZeroFlag and CarryFlag as set accordingly 
    ; We pop and restore the old values of HIGH and LOW. Order is important, LOW always last.
    ; We jump based on ZeroFlag to a flow form and assign 0 or 1 to AX
    cli
    xor ax, ax       ; ax = 0x0000    
    mov es, ax       ; DS = 0x0000    
    mov ax, 0xFFFF   ; fill AX with all 11111111 11111111
    mov ds, ax       ; Extra segment: FFFF x 0x10 = FFFF0
    ; ES: Low value
    ; DS: High value 
    mov di, 0x0500   ; move 0x500 to Source index ES:DI 0000:0x500
    mov si, 0x0510   ; move 0x510 to Destin index DS:SI FFFF:0x510
    ;ES:DI 0000:0x500 low 
    ;DS:SI FFFF:0x510 high

    ; Save whatever is in HIGH and LOW (if we can reach it obviously)
    mov al, byte [es:di] ; LOW
    push ax     
    mov al, byte [ds:si] ; HIGH
    push ax

    mov byte [es:di], 0x00 ; Try inject 0x00 on low memory 0x500, for comparation in 2 instructions below
    mov byte [ds:si], 0xFF ; Try inject 0xFF on high memory 0x10510. This COULD wrap!

    cmp byte [es:di], 0xFF ; LOW pointer will have 0x00 if a20 enabled, and 0xff if wrapped and disabled
;                           ; (a20enabled) 0x00 substract 0xFF goes negative. ZF=0 CF=1
;                           ; (a20disabled) 0xFF substract 0xFF goes Zero. ZF=1 CF=0
    ; Restore the original values. 
    pop ax
    mov byte [ds:si], al ; First HIGH, could wrap to LOW
    pop ax
    mov byte [es:di], al ; Then LOW, does not matter if the above wrapped. 
    sti

    mov ax, 0
    je a20_is_disabled ; jumps if wraped, a20 disabled, and LOW was 0xFF when cmp with 0xFF happened. so AX = 0 (BAD)
    mov ax, 1 ; No jump above! a20enabld AX=1 (GOOD)    
    a20_is_disabled: ; a20disabled AX=0
    ; TODO, enable the gate, so far bochs has it enable by default. Alter the order of ax 0 or 1 to dont crossjump

; 1,048,576 1 MiB 0xFFFF x 0x10 (65,535 x 16)
; 1,049,856 ES:DI 0xFFFF:0x510 ---0x10_0500 1280 bytes difference (0x510)
; 1,280 0x500




;mov si, msg
cli             ; Disable interrupts
xor ax, ax      ; set AX to 0
mov ds, ax 
sti







mov si, msg ;msg ;lea si, [msg - 0x7600] old shenanigans hen relocating and before changing ORG to 0x600
print_char:
    lodsb ; loads SI to AL
    test al, al ; 0 terminated string, out !
    jz .halt
    mov ah, 0x0E ; Write text in teletype mode
    mov bh, 0x00 ; page number, text modes
    mov bl, 0x07 ; light grey on black
    int 0x10
    jmp print_char

.halt:
    xchg bx, bx
    hlt
    jmp .halt

msg db 'ABCD1234', 0


; GDT static data


emitted_bytes_so_far_plus_org:
;times 510 - ($ - $$) db 0
times 510-(emitted_bytes_so_far_plus_org - byte_zero_plus_org) db 0
dw 0xAA55
TIMES (512*5)-($-$$) db 0x05



; NOTES: 
; Moved the msg literal string to the bottom. If you place it at the top it counts as instructions if you load it as a plain binary
; This is what the linker and a proper executable format would handle for you.

