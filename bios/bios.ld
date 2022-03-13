ENTRY(_start)
MEMORY
{
    ROM (rx)    : ORIGIN = 0x0, LENGTH = 16K
}

SECTIONS
{
    . = ORIGIN(ROM);
    .init : { *(.init) } >ROM

    .text : {
        *(.text)
        *(.text.*)
    } >ROM

    .rodata : {
        *(.rodata)
        *(.rodata.*)
    } >ROM
}