ASFLAGS     := -mcpu=arm7tdmi -march=armv4t -mthumb-interwork
CFLAGS      := -mcpu=arm7tdmi -march=armv4t -mthumb-interwork -ffreestanding -static -nostdlib -c -O0
AS          := $(DEVKITARM)/bin/arm-none-eabi-as
CC          := $(DEVKITARM)/bin/arm-none-eabi-gcc
LD          := $(DEVKITARM)/bin/arm-none-eabi-ld
OBJCOPY     := $(DEVKITARM)/bin/arm-none-eabi-objcopy
OBJDUMP     := $(DEVKITARM)/bin/arm-none-eabi-objdump

AS_SOURCES  := $(wildcard *.s)
AS_OBJECTS  := $(patsubst %.s,%.o,$(AS_SOURCES))

C_SOURCES   := $(wildcard *.c)
C_OBJECTS   := $(patsubst %.c,%.o,$(C_SOURCES))

OBJECTS     := $(AS_OBJECTS) $(C_OBJECTS)

all: bios.bin
	make clear

%.o: %.s
	$(AS) $(ASFLAGS) -o $@ $<
%o: %.c
	$(CC) $(CFLAGS) -o $@ $<

bios.elf: $(OBJECTS)
	$(LD) --script bios.ld -o $@ $^

bios.bin: bios.elf
	$(OBJCOPY) -O binary $< $@

.PHONY: clear
clear:
	rm -f bios.elf
	rm -f $(OBJECTS)

.PHONY: clean
clean:	clear
	rm -f bios.bin