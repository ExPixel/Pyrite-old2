mod common;

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_branch() {
    let (cpu, _mem) = common::execute_arm(
        "b",
        "
        mov     r0, #5
        b       _exit
        mov     r0, #8  @ should not be executed
        ",
    );
    assert_eq!(cpu.registers.read(0), 5);
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_branch_and_link() {
    let (cpu, _mem) = common::execute_arm(
        "bl",
        "
        ldr     r1, =skipped
        mov     r0, #5
        bl       _exit
    skipped:
        mov     r0, #8  @ should not be executed
        ",
    );
    assert_eq!(cpu.registers.read(0), 5);
    assert_eq!(cpu.registers.read(14), cpu.registers.read(1));
}

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_branch_and_exchange() {
    let (cpu, _mem) = common::execute_arm(
        "bx-to-arm",
        "
        ldr     r1, =location
        mov     r0, #5
        bx      r1
        mov     r0, #3
    location:
        nop
        ",
    );
    assert_eq!(cpu.registers.read(0), 5);

    let (cpu, _mem) = common::execute_arm(
        "bx-to-thumb",
        "
        ldr     r1, =location
        orr     r1, #1
        mov     r0, #5
        bx      r1
        mov     r0, #3

        .thumb
    location:
        nop
        ",
    );
    assert_eq!(cpu.registers.read(0), 5);
}
