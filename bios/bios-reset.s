.section ".text"

.global swi_soft_reset
.align 4
.arm

swi_soft_reset:
    ldr     r0, =0x08000000
    bx      r0
