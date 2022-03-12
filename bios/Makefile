ASFLAGS := -mcpu=arm7tdmi -march=armv4t -mthumb-interwork
AS		:= $(DEVKITARM)/bin/arm-none-eabi-as $(AS_ARGS)
LD		:= $(DEVKITARM)/bin/arm-none-eabi-ld
OBJCOPY	:= $(DEVKITARM)/bin/arm-none-eabi-objcopy
OBJDUMP	:= $(DEVKITARM)/bin/arm-none-eabi-objdump

SOURCES := $(wildcard *.s)
OBJECTS := $(patsubst %.s,%.o,$(wildcard *.s))

all: bios.bin
	make clear

%.o: %.cpp
	$(AS) $(ASFLAGS) -o $@ $<

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