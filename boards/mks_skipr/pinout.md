# MKS SKIPR Pinout for Klipper Rust Firmware

**Note:** This pinout is specific to the Klipper Rust firmware implementation. While it is based on the official MKS SKIPR schematic, it may differ from other firmware. For the official schematic, please refer to the manufacturer's documentation.

This document provides a comprehensive mapping of the pins available on the MKS SKIPR board, which is based on the STM32F407VET6 microcontroller.

## Main Connectors

| Connector | Pin | Function | MCU Pin | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **MOTOR** | | | | |
| | X-E | Stepper Enable | PA15 | Active Low |
| | X-S | Stepper Step | PA14 | |
| | X-D | Stepper Direction | PA13 | |
| | Y-E | Stepper Enable | PE11 | Active Low |
| | Y-S | Stepper Step | PE10 | |
| | Y-D | Stepper Direction | PE9 | |
| | Z-E | Stepper Enable | PE15 | Active Low |
| | Z-S | Stepper Step | PE14 | |
| | Z-D | Stepper Direction | PE13 | |
| | E0-E | Stepper Enable | PB5 | Active Low |
| | E0-S | Stepper Step | PB4 | |
| | E0-D | Stepper Direction | PB3 | |

## Heaters and Thermistors

| Connector | Pin Name | Function | MCU Pin |
| :--- | :--- | :--- | :--- |
| **BED_OUT** | +/- | Heated Bed Out | PB10 |
| **TH_BED** | T1 | Bed Thermistor | PA1 |
| **HE0_OUT** | +/- | Extruder 0 Heat | PB11 |
| **TH_E0** | T0 | E0 Thermistor | PA2 |

## Fans

| Connector | Function | MCU Pin |
| :--- | :--- | :--- |
| **FAN0** | Part Cooling | PB15 |
| **FAN1** | Hotend Fan | PB14 |

**Note:** This is a simplified pinout. Always refer to the official schematic from the manufacturer for the most accurate and up-to-date information.