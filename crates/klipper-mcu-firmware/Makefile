# Makefile for building and flashing the klipper-mcu-firmware

# Variables
TARGET=thumbv7em-none-eabihf
BINARY_NAME=klipper-mcu-firmware
ELF_FILE=target/$(TARGET)/release/$(BINARY_NAME)
BIN_FILE=target/$(TARGET)/release/$(BINARY_NAME).bin
SD_CARD_FILE=firmware.bin

.PHONY: all clean flash flash-example build-release build-example

all: build-release

build-release:
	@echo "Building release firmware..."
	@cargo build --release

build-example:
	@echo "Building led_ping example..."
	@cargo build --release --example led_ping

flash: build-release
	@echo "Flashing main firmware..."
	@probe-rs run --chip STM32F407VGTx $(ELF_FILE)

flash-example: build-example
	@echo "Flashing led_ping example..."
	@probe-rs run --chip STM32F407VGTx target/$(TARGET)/release/examples/led_ping

# Creates the firmware.bin file for SD card flashing on the MKS SKIPR
sd-card-image: build-release
	@echo "Creating SD card binary..."
	@arm-none-eabi-objcopy -O binary $(ELF_FILE) $(BIN_FILE)
	@cp $(BIN_FILE) $(SD_CARD_FILE)
	@echo "Created $(SD_CARD_FILE). Copy this to your SD card."


clean:
	@echo "Cleaning project..."
	@cargo clean
