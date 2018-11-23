TARGET=pre_filter
VERSION=v3

ifeq ($(type),t85)
    MCU=attiny85
    AVRDUDEMCU=t85
else
    MCU=attiny45
    AVRDUDEMCU=t45
endif

CC=/usr/bin/avr-gcc
CFLAGS= -mmcu=$(MCU) -DF_CPU=16000000UL -Wall -Os -fno-inline-small-functions -mrelax
OBJ2HEX=/usr/bin/avr-objcopy
AVRDUDE=/usr/bin/avrdude
PROGRAMMER=-P gpio -c gpio
#PROGRAMMER=-P /dev/spidev0.0 -c linuxspi

program : $(TARGET)_$(AVRDUDEMCU).hex
	sudo $(AVRDUDE) -p $(AVRDUDEMCU) $(PROGRAMMER) -b 14400 -U flash:w:$(TARGET)_$(AVRDUDEMCU).hex

compile :
	$(CC) $(CFLAGS) -c $(TARGET).c
	$(CC) $(CFLAGS) -o $(TARGET)_$(AVRDUDEMCU).elf $(TARGET).o
	$(OBJ2HEX) -j .text -j .data -O ihex $(TARGET)_$(AVRDUDEMCU).elf $(TARGET)_$(AVRDUDEMCU).hex

clean :
	rm -f *.hex *.obj *.o *.elf

all: compile erase program
#clean
erase:
	sudo $(AVRDUDE) -p $(AVRDUDEMCU) $(PROGRAMMER) -e -U lfuse:w:0xe1:m -U hfuse:w:0xdf:m
	
# ATtiny45 (internal 1 MHz clock) 	0x62 	0xdf 
# ATtiny45 (internal 8 MHz clock) 	0xe2	0xdf 
# ATtiny45 (external 20 MHz clock) 	0xfe	0xdf 
# ATtiny85 (internal 1 MHz clock) 	0x62 	0xdf 
# ATtiny85 (internal 8 MHz clock) 	0xe2 	0xdf 
# ATtiny85 (external 20 MHz clock) 	0xfe 	0xdf
