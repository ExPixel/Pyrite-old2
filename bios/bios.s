.section ".init"

.global _start
.align 4
.arm

_start:
    b v_reset
    b v_undefined_instruction
    b v_software_interrupt
    b v_prefetch_abort
    b v_data_abort
    b v_adress_exceeeds_26bit
    b v_irq_interrupt
    b v_fiq_interrupt

.section ".text"

v_reset:
    ldr     r0, =swi_soft_reset
    bx      r0

v_undefined_instruction:
    movs    pc, lr

v_software_interrupt:
    movs    pc, lr

v_prefetch_abort:
    movs    pc, lr

v_data_abort:
    movs    pc, lr

v_adress_exceeeds_26bit:
    movs    pc, lr

v_irq_interrupt:
    stmfd   r13!, {r0-r3, r12, r14}  @ save registers to SP_irq
    movs    r0, #0x04000000          @ ptr+4 to 03FFFFFC (mirror of 03007FFC)
    add     r14, r15, #0x0           @ retadr for USER handler $+8=138h
    ldr     r15, [r0, #-0x4]         @ jump to [03FFFFFC] USER handler
    ldmfd   r13!, {r0-r3, r12, r14}  @ restore registers from SP_irq
    subs    r15, r14, #0x4           @ return from IRQ (PC=LR-4, CPSR=SPSR)
    movs    pc, lr

v_fiq_interrupt:
    movs    pc, lr
