AS		:= $(DEVKITARM)/bin/arm-none-eabi-as
LD		:= $(DEVKITARM)/bin/arm-none-eabi-ld
OBJCOPY	:= $(DEVKITARM)/bin/arm-none-eabi-objcopy
OBJDUMP	:= $(DEVKITARM)/bin/arm-none-eabi-objdump

all: bios.bin

bios.o: bios.s
	$(AS) -mcpu=arm7tdmi -march=armv4t -mthumb-interwork -o $@ $<

bios.elf: bios.o
	$(LD) --script bios.ld -o $@ $<

bios.bin: bios.elf
	$(OBJCOPY) -O binary $< $@

.PHONY: clean
clean:
	rm -f bios.bin
	rm -f bios.elf
	rm -f bios.o