mod common;

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_mrs() {
    let (cpu, _mem) = common::execute_arm(
        "mrs",
        "
        mov     r0, #0xC0000000
        adds    r0, r0, r0        @ Z=0, C=1, V=0, N=1
        mov     r2, #0x50000000
        mrs     r2, cpsr
        ",
    );

    assert_eq!(cpu.registers.read(2) & 0x10000000, 0); // N = 1
    assert_ne!(cpu.registers.read(2) & 0x20000000, 0); // Z = 0
    assert_eq!(cpu.registers.read(2) & 0x40000000, 0); // C = 1
    assert_ne!(cpu.registers.read(2) & 0x80000000, 0); // V = 0
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_msr() {
    let mut exec = common::Executor::new("msr", arm::Isa::Arm);

    exec.push(
        "
        movs    r2, #0
        msr     cpsr_flg, #0x90000000
        ",
    );
    assert!(!exec.cpu.registers.getf_c());
    assert!(exec.cpu.registers.getf_n());
    assert!(exec.cpu.registers.getf_v());
    assert!(!exec.cpu.registers.getf_z());

    exec.push(
        "
        mov     r11, #1
        mrs     r2, cpsr
        bic     r2, r2, #0x1f
        orr     r2, r2, #0x11 
        msr     cpsr, r2        @ Set FIQ mode
        mov     r11, #2
        orr     r2, r2, #0x1f
        msr     cpsr, r2        @ Set System mode
        ",
    );
    assert_eq!(exec.cpu.registers.read(11), 1);
}
