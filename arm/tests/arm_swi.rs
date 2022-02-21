mod common;

#[test]
#[cfg(feature = "devkit-arm-tests")]
pub fn test_swi() {
    let (cpu, _mem) = common::execute_arm(
        "swi",
        "
        b       main
        b       undefined_handler
        b       swi_handler
    main:
        mov     r0, #4
        ldr     r3, =return_point
        swi     #6
    return_point:
        mov     r2, #6
        b       _exit
    undefined_handler:
        b       _exit
    swi_handler:
        mov     r1, #5
        mov     r4, r14
        movs    r15, r14
        b       _exit
        ",
    );
    assert_eq!(cpu.registers.read(0), 4);
    assert_eq!(cpu.registers.read(1), 5);
    assert_eq!(cpu.registers.read(2), 6);
    assert_eq!(cpu.registers.read(3), cpu.registers.read(4));
}
