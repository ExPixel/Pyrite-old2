.include "constants.inc"

.section ".text"

.global swi_soft_reset
.align 4
.arm

@@  SWI 00h (GBA/NDS7/NDS9) - SoftReset
@@  Clears 200h bytes of RAM (containing stacks, and BIOS IRQ vector/flags), initializes system, supervisor, and irq stack pointers, sets R0-R12, LR_svc, SPSR_svc, LR_irq, and SPSR_irq to zero, and enters system mode.
@@    sp_svc    sp_irq    sp_sys    zerofilled area       return address
@@    3007FE0h  3007FA0h  3007F00h  [3007E00h..3007FFFh]  Flag[3007FFAh]
@@  Return: Does not return to calling procedure, instead, loads the above return address into R14, and then jumps to that address by a "BX R14" opcode.
swi_soft_reset:

    mrs     r1, cpsr            @ save the mode bits from CPSR
    bic     r0, r1, #MODE_MASK
    orr     r0, r0, #MODE_SVC
    ldr     sp, =0x3007FE0      @ sp_svc = 0x3007FE0
    mov     lr, #0              @ lr_svc = 0
    msr     spsr, lr            @ spsr_svc = 0
    bic     r0, r1, #MODE_MASK
    orr     r0, r0, #MODE_IRQ
    ldr     sp, =0x3007FA0      @ sp_irq = 0x3007FA0
    mov     lr, #0              @ lr_irq = 0
    msr     spsr, lr            @ spsr_irq = 0
    bic     r0, r1, #MODE_MASK
    orr     r0, r0, #MODE_SYS
    ldr     sp, =0x3007F00      @ sp_sys = 0x3007F00

    ldr     r0, =0x3007E00
    mov     r1, #0
    mov     r2, #0x200
    bl      memset

    mov     r0, #0
    mov     r1, #0
    mov     r2, #0
    mov     r3, #0
    mov     r4, #0
    mov     r5, #0
    mov     r6, #0
    mov     r7, #0
    mov     r9, #0
    mov     r10, #0
    mov     r11, #0
    mov     r12, #0

    ldr     lr, =0x08000000
    bx      lr
