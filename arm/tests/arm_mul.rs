mod common;

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_mla() {
    let (cpu, _mem) = common::execute_arm(
        "mla",
        "
        ldr     r2, =0xFFFFFFF6
        mov     r3, #0x14
        ldr     r4, =0xD0
        mlas    r2, r3, r2, r4
        ",
    );
    assert!(!cpu.registers.getf_n());
    assert!(!cpu.registers.getf_z());
    assert_eq!(cpu.registers.read(2), 8);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_mul() {
    let (cpu, _mem) = common::execute_arm(
        "mul",
        "
        ldr     r2, =0xFFFFFFF6
        mov     r3, #0x14
        ldr     r4, =0xFFFFFF38
        muls    r2, r3, r2
        ",
    );
    assert!(cpu.registers.getf_n());
    assert!(!cpu.registers.getf_z());
    assert_eq!(cpu.registers.read(2), cpu.registers.read(4));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_umull() {
    let (cpu, _mem) = common::execute_arm(
        "umull",
        "
        ldr     r2, =0x80000000
        mov     r3, #8
        umulls  r4, r5, r2, r3
        ",
    );
    assert!(!cpu.registers.getf_n());
    assert!(!cpu.registers.getf_z());
    assert_eq!(cpu.registers.read(5), 4);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_smull() {
    let (cpu, _mem) = common::execute_arm(
        "smull",
        "
        ldr     r2, =0x80000000
        mov     r3, #8
        smulls  r4, r5, r2, r3
        ",
    );
    assert!(cpu.registers.getf_n());
    assert!(!cpu.registers.getf_z());
    assert_eq!(cpu.registers.read(5), 0xFFFFFFFC);
}
