mod common;

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_imm() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-preinc-imm",
        "
        ldr     r0, =var
        sub     r2, r0, #3
        mov     r3, r2
        ldr     r0, [r0, #0]
        ldr     r1, [r2, #3]

        var: .word 0x00000080
        ",
    );

    assert_eq!(cpu.registers.read(0), 0x80);
    assert_eq!(cpu.registers.read(1), 0x80);
    assert_eq!(cpu.registers.read(2), cpu.registers.read(3));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_imm_unaligned() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-preinc-imm-unaligned",
        "
        ldr     r0, =var
        ldr     r1, [r0, #1]
        ldr     r2, [r0, #2]
        ldr     r3, [r0, #3]

        var: .word 0xff008f00
        ",
    );

    assert_eq!(cpu.registers.read(1), 0x00ff008f);
    assert_eq!(cpu.registers.read(2), 0x8f00ff00);
    assert_eq!(cpu.registers.read(3), 0x008f00ff);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_imm() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-predec-imm",
        "
        ldr     r0, =var
        mov     r2, r0
        mov     r3, r2
        add     r0, r0, #206
        ldr     r1, [r0, #-206]
        ldr     r2, [r2, #-0]

        var: .word 0x00000080
        ",
    );

    assert_eq!(cpu.registers.read(1), 0x80);
    assert_eq!(cpu.registers.read(2), 0x80);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_imm_unaligned() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-predec-imm-unaligned",
        "
        ldr     r0, =var+4
        ldr     r0, [r0, #-2]

        var: .word 0xff008f00
        ",
    );

    assert_eq!(cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_imm_writeback() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-preinc-imm-writeback",
        "
        ldr     r0, =var
        sub     r2, r0, #3
        mov     r3, r0
        ldr     r0, [r0, #0]!
        ldr     r1, [r2, #3]!

        var: .word 0x00000080
        ",
    );

    assert_eq!(cpu.registers.read(0), 0x80);
    assert_eq!(cpu.registers.read(1), 0x80);
    assert_eq!(cpu.registers.read(2), cpu.registers.read(3));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_imm_unaligned_writeback() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-preinc-imm-unaligned-writeback",
        "
        ldr     r0, =var
        ldr     r0, [r0, #2]!

        var: .word 0xff008f00
        ",
    );

    assert_eq!(cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_imm_writeback() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-predec-imm-writeback",
        "
        ldr     r0, =var
        add     r2, r0, #1
        mov     r3, r0
        add     r0, r0, #206
        ldr     r0, [r0, #-206]!
        ldr     r1, [r2, #-1]!

        var: .word 0x00000080
        ",
    );

    assert_eq!(cpu.registers.read(0), 0x80);
    assert_eq!(cpu.registers.read(1), 0x80);
    assert_eq!(cpu.registers.read(2), cpu.registers.read(3));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_imm_unaligned_writeback() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-predec-imm-unaligned-writeback",
        "
        ldr     r0, =var+4
        ldr     r0, [r0, #-2]!


        var: .word 0xff008f00
        ",
    );

    assert_eq!(cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_reg() {
    let mut exec = common::Executor::new("ldr-preinc-reg", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        sub     r2, r0, #8
        sub     r0, r0, #1
        mov     r3, r2
        mov     r4, #2
        ldr     r0, [r0, r4, lsr #1]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2, r4, lsl #2]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r2, r2, lsr #1
        mov     r3, #0xC0000000     @ this is in arm wrestler but it doesn't do anything???
        ldr     r0, [r2, r2]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r3, #0x8
        ldr     r0, [r2, r3, lsr #32]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        add     r2, r2, #1
        mov     r3, #0xC0000000
        ldr     r0, [r2, r3, asr #32]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        add     r2, r2, #2
        ldr     r3, =0xFFFFFFFC
        adds    r4, r3, r3          @ set carry
        ldr     r0, [r2, r3, rrx]
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.read(0), 0x80);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_reg_unaligned() {
    let (cpu, _mem) = common::execute_arm(
        "ldr-preinc-reg-unaligned",
        "
        ldr     r0, =var
        mov     r2, #2
        ldr     r0, [r0, r2]

        var:    .word   0xff008f00
        ",
    );
    assert_eq!(cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_reg() {
    let mut exec = common::Executor::new("ldr-predec-reg", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        add     r2, r0, #8
        add     r0, r0, #1
        mov     r3, r2
        mov     r4, #2
        ldr     r0, [r0, -r4, lsr #1]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2, -r4, lsl #2]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r3, #0x8
        ldr     r0, [r2, -r3, lsr #32]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        sub     r2, r2, #1
        mov     r3, #0x80000000
        ldr     r0, [r2, -r3, asr #32]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        sub     r2, r2, #4
        ldr     r3, =0xFFFFFFF8
        adds    r4, r3, r3          @ set carry
        ldr     r0, [r2, -r3, rrx]
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.read(0), 0x80);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_reg_unaligned() {
    let mut exec = common::Executor::new("ldr-predec-reg-unaligned", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0xff008f00
        ",
    );

    exec.push(
        "
        ldr     r0, =var+4
        mov     r2, #1
        ldr     r0, [r0, -r2, lsl #1]
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_reg_writeback() {
    let mut exec = common::Executor::new("ldr-preinc-reg-writeback", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        mov     r3, r0
        sub     r2, r0, #8
        sub     r0, r0, #1
        mov     r4, #2
        ldr     r0, [r0, r4, lsr #1]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2, r4, lsl #2]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r4, r2
        mov     r2, r2, lsr#1
        mov     r3, #0xC0000000
        ldr     r0, [r2, r2]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(4));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r4, r2
        add     r2, r2, #1
        mov     r3, #0xC0000000
        ldr     r0, [r2, r3, asr #32]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(4));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r5, r2
        add     r2, r2, #2
        ldr     r3, =0xfffffffc
        adds    r4, r3, r3          @ set carry
        ldr     r0, [r2, r3, rrx]!
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(5));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_preinc_reg_unaligned_writeback() {
    let mut exec = common::Executor::new("ldr-preinc-reg-unaligned-writeback", arm::Isa::Arm);
    exec.data(
        "
        var: .word 0xff008f00
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        mov     r2, #2
        ldr     r0, [r0, r2]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_reg_writeback() {
    let mut exec = common::Executor::new("ldr-predec-reg-writeback", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        mov     r3, r0
        add     r2, r0, #8
        add     r0, r0, #1
        mov     r4, #2
        ldr     r0, [r0, -r4, lsr #1]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2, -r4, lsl #2]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r4, r2
        sub     r2, r2, #1
        mov     r3, #0x80000000
        ldr     r0, [r2, -r3, asr #32]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(4));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r5, r2
        sub     r2, r2, #4
        ldr     r3, =0xfffffff8
        adds    r4, r3, r3          @ set carry
        ldr     r0, [r2, -r3, rrx]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(5));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_predec_reg_unaligned_writeback() {
    let mut exec = common::Executor::new("ldr-predec-reg-unaligned-writeback", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0xff008f00
        ",
    );

    exec.push(
        "
        ldr     r0, =var+4
        mov     r2, #2
        ldr     r0, [r0, -r2]!
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postinc_imm() {
    let mut exec = common::Executor::new("ldr-postinc-imm", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        add     r3, r0, #3
        mov     r2, r0
        ldr     r0, [r0], #3
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2], #3
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postinc_imm_unaligned() {
    let mut exec = common::Executor::new("ldr-postinc-imm-unaligned", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0xff008f00
        ",
    );

    exec.push(
        "
        ldr     r0, =var+2
        ldr     r0, [r0], #5
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postdec_imm() {
    let mut exec = common::Executor::new("ldr-postdec-imm", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        mov     r2, r0
        sub     r3, r0, #0xff
        ldr     r0, [r0], #-0xff
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2], #-0xff
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postdec_imm_unaligned() {
    let mut exec = common::Executor::new("ldr-postdec-imm-unaligned", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0xff008f00
        ",
    );

    exec.push(
        "
        ldr     r0, =var+2
        ldr     r0, [r0], #-5
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postinc_reg() {
    let mut exec = common::Executor::new("ldr-postinc-reg", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        mov     r2, r0
        add     r5, r0, #8
        mov     r3, r0
        mov     r4, #2
        ldr     r0, [r0], r4, lsr #1
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2], r4, lsl #2
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(5));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        mov     r0, #123
        add     r3, r2, r0
        ldr     r0, [r2], r0
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        sub     r4, r2, #1
        mov     r3, #0xC0000000
        ldr     r0, [r2], r3, asr #32
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(4));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        sub     r4, r2, #2
        ldr     r3, =0xFFFFFFFC
        adds    r5, r3, r3        @ set carry
        ldr     r0, [r2], r3, rrx
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(4));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postinc_reg_unaligned() {
    let mut exec = common::Executor::new("ldr-postinc-reg-unaligned", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0xff008f00
        ",
    );

    exec.push(
        "
        ldr     r0, =var+2
        mov     r2, #1
        ldr     r0, [r0], r2
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x8f00ff00);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postdec_reg() {
    let mut exec = common::Executor::new("ldr-postdec-reg", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0x00000080
        ",
    );

    exec.push(
        "
        ldr     r0, =var
        mov     r2, r0
        sub     r5, r0, #16
        mov     r3, r0
        mov     r4, #2
        ldr     r0, [r0], -r4, lsr #1
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);

    exec.push(
        "
        ldr     r0, [r2], -r4, lsl #3
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(5));

    exec.clear_source();
    exec.push(
        "
        ldr r2, =var
        mov     r0, #123
        sub     r3, r2, r0
        ldr     r0, [r2], -r0
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        add     r4, r2, #1
        mov     r3, #0xC0000000
        ldr     r0, [r2], -r3, asr #32
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(4));

    exec.clear_source();
    exec.push(
        "
        ldr     r2, =var
        add     r4, r2, #2
        ldr     r3, =0xfffffffc
        adds    r5, r3, r3        @ set carry
        ldr     r0, [r2], -r3, rrx
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.read(0), 0x80);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(4));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_ldr_postdec_reg_unaligned() {
    let mut exec = common::Executor::new("ldr-postdec-reg-unaligned", arm::Isa::Arm);
    exec.data(
        "
        var:    .word 0xff008f00
        ",
    );

    exec.push(
        "
        ldr     r0, =var+2
        mov     r2, #5
        ldr     r0, [r0], -r2
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 0x8f00ff00);
}
