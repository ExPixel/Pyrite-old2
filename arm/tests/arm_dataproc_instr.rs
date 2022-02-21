mod common;

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_mov() {
    let (cpu, _mem) = common::execute_arm("mov-imm", "mov r0, #5");
    assert_eq!(cpu.registers.read(0), 5);

    // If the shift amount is specified in the instruction, the PC will be 8 bytes ahead.
    let (cpu, _mem) = common::execute_arm("mov-r15", "mov r0, r15");
    assert_eq!(cpu.registers.read(0), 8);

    // If a register is used to specify the shift amount the PC will be 12 bytes ahead.
    let (cpu, _mem) = common::execute_arm("mov-r15-shift-reg", "mov r0, r15, lsl r3");
    assert_eq!(cpu.registers.read(0), 12);

    // Check that flags and Rd are correctly set on mov with lsr #32.
    let (cpu, _mem) = common::execute_arm(
        "mov-lsr-32",
        "
        ldr     r1, =0x80000001
        movs    r0, r1, lsr #32
        ",
    );
    assert_eq!(cpu.registers.getf_c(), true);
    assert_eq!(cpu.registers.getf_n(), false);
    assert_eq!(cpu.registers.getf_z(), true);
    assert_eq!(cpu.registers.read(0), 0);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_asr() {
    // ASR by register with a value of 0.
    let (cpu, _mem) = common::execute_arm(
        "mov-asr-reg-0",
        "
        mov     r3, #3
        movs    r4, r3, lsr #1  @ set carry
        mov     r2, #0
        movs    r3, r4, asr r2
        ",
    );
    assert_eq!(cpu.registers.getf_c(), true);
    assert_eq!(cpu.registers.read(3), 1);

    // ASR by register with a value of 33
    let (cpu, _mem) = common::execute_arm(
        "mov-asr-reg-33",
        "
        ldr     r2, =0x80000000
        mov     r3, #33
        movs    r2, r2, asr r3
        ",
    );
    assert_eq!(cpu.registers.getf_c(), true);
    assert_eq!(cpu.registers.read(2), 0xFFFFFFFF);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_mvn() {
    let (cpu, _mem) = common::execute_arm(
        "mvn",
        "
        mov     r2, #label
        mvn     r3, #0
        eor     r2, r2, r3
        mvn     r3, r15    
        nop
        label:
        ",
    );
    assert_eq!(cpu.registers.read(3), cpu.registers.read(2));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_orr() {
    let (cpu, _mem) = common::execute_arm(
        "orr",
        "
        mov     r2, #2
        mov     r3, #3
        movs    r4, r3, lsr #1      @ set carry 
        orrs    r3, r3, r2, rrx
        ",
    );
    assert_eq!(cpu.registers.getf_c(), false);
    assert_eq!(cpu.registers.getf_n(), true);
    assert_eq!(cpu.registers.getf_z(), false);
    assert_eq!(cpu.registers.read(3), 0x80000003);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_rsc() {
    let (cpu, _mem) = common::execute_arm(
        "rsc",
        "
        mov     r2, #2
        mov     r3, #3
        adds    r9, r9, r9  @ clear carry
        rscs    r3, r2, r3
        ",
    );
    assert_eq!(cpu.registers.getf_c(), true);
    assert_eq!(cpu.registers.getf_n(), false);
    assert_eq!(cpu.registers.getf_z(), true);
    assert_eq!(cpu.registers.read(2), 2);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_sbc() {
    let mut exec = common::Executor::new("sbc", arm::Isa::Arm);

    exec.push(
        "
        ldr     r2,=0xFFFFFFFF
        adds    r3, r2, r2      @ set carry
        sbcs    r2, r2, r2
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.getf_n(), false);
    assert_eq!(exec.cpu.registers.getf_z(), true);

    exec.push(
        "
        adds    r9, r9          @ clear carry
        sbcs    r2, r2, #0
        ",
    );
    assert_eq!(exec.cpu.registers.getf_z(), false);
    assert_eq!(exec.cpu.registers.getf_c(), false);
    assert_eq!(exec.cpu.registers.getf_n(), true);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_adc() {
    let mut exec = common::Executor::new("adc", arm::Isa::Arm);
    exec.push(
        "
        mov     r2, #0x80000000
        mov     r3, #0xF
        adds    r9, r9, r9          @ clear carry
        adcs    r2, r2, r3
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), false);
    assert_eq!(exec.cpu.registers.getf_n(), true);
    assert_eq!(exec.cpu.registers.getf_v(), false);
    assert_eq!(exec.cpu.registers.getf_z(), false);

    exec.push(
        "
        adcs    r2, r2, r2
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.getf_n(), false);

    exec.push(
        "
        adc     r3, r3, r3
        ",
    );
    assert_eq!(exec.cpu.registers.read(3), 0x1F);

    exec.push(
        "
        mov     r0, #0xFFFFFFFF
        adds    r0, r0, #1          @ set carry
        mov     r0, #0
        mov     r2, #1
        adc     r0, r0, r2, lsr #1
        ",
    );
    assert_eq!(exec.cpu.registers.read(0), 1);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_add() {
    let mut exec = common::Executor::new("add", arm::Isa::Arm);

    exec.push(
        "
        ldr     r2, =0xFFFFFFFE
        mov     r3, #1
        adds    r2, r2, r3
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), false);
    assert_eq!(exec.cpu.registers.getf_n(), true);
    assert_eq!(exec.cpu.registers.getf_v(), false);
    assert_eq!(exec.cpu.registers.getf_z(), false);

    exec.push(
        "
        adds    r2, r2, r3	
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.getf_n(), false);
    assert_eq!(exec.cpu.registers.getf_v(), false);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_and() {
    let mut exec = common::Executor::new("and", arm::Isa::Arm);

    exec.push(
        "
        mov     r2, #2
        mov     r3, #5
        ands    r2, r2, r3, lsr #1
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.getf_z(), false);
    assert_eq!(exec.cpu.registers.read(2), 2);

    exec.push(
        "
        mov     r2, #0xC00
        mov     r3, r2
        mov     r4, #0x80000000
        ands    r2, r2, r4, asr #32
        ",
    );
    assert_eq!(exec.cpu.registers.getf_c(), true);
    assert_eq!(exec.cpu.registers.getf_n(), false);
    assert_eq!(exec.cpu.registers.getf_z(), false);
    assert_eq!(exec.cpu.registers.read(2), exec.cpu.registers.read(3));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_bic() {
    let (cpu, _mem) = common::execute_arm(
        "bic",
        "
        adds    r9, r9, r9          @ clear carry
        ldr     r2, =0xFFFFFFFF
        ldr     r3, =0xC000000D
        bics    r2, r2, r3, asr #1
        ",
    );
    assert_eq!(cpu.registers.getf_c(), true);
    assert_eq!(cpu.registers.getf_n(), false);
    assert_eq!(cpu.registers.getf_z(), false);
    assert_eq!(cpu.registers.read(2), 0x1FFFFFF9);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_eor() {
    let (cpu, _mem) = common::execute_arm(
        "eor",
        "
        mov     r2, #1
        mov     r3, #3
        eors    r2, r2, r3, lsl #31
        eors    r2, r2, r3, lsl #0
        ",
    );
    assert_eq!(cpu.registers.getf_c(), true);
    assert_eq!(cpu.registers.getf_n(), true);
    assert_eq!(cpu.registers.getf_z(), false);
    assert_eq!(cpu.registers.read(2), 0x80000002);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_cmn() {
    let (cpu, _mem) = common::execute_arm(
        "cmn",
        "
        adds    r9, r9, r9      @ clear carry
        ldr     r2, =0x7FFFFFFF
        ldr     r3, =0x70000000
        cmn     r2, r3
        ",
    );
    assert_eq!(cpu.registers.getf_c(), false);
    assert_eq!(cpu.registers.getf_n(), true);
    assert_eq!(cpu.registers.getf_v(), true);
    assert_eq!(cpu.registers.getf_z(), false);
    assert_eq!(cpu.registers.read(2), 0x7FFFFFFF);
}
