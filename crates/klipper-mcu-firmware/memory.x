/* Linker script for the STM32F407VGT6 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* The bootloader on the MKS SKIPR expects the firmware to be at 0x08010000.
   You may need to adjust this depending on your board's bootloader. */
_stext = 0x08010000;
