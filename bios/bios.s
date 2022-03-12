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
    ldr r0, =0x08000000
    bx r0
v_undefined_instruction:
    movs pc, lr
v_software_interrupt:
    movs pc, lr
v_prefetch_abort:
    movs pc, lr
v_data_abort:
    movs pc, lr
v_adress_exceeeds_26bit:
    movs pc, lr
v_irq_interrupt:
    movs pc, lr
v_fiq_interrupt:
    movs pc, lr
