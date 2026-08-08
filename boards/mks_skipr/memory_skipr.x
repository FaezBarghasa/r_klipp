/*
    Linker script for the MKS SKIPR board.
    This is based on the STM32F407VET6 microcontroller.
    - 512 KB Flash
    - 192 KB SRAM
*/

MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM   : ORIGIN = 0x20000000, LENGTH = 192K
}

/* Define the stack size */
_stack_size = 16K;

/* The rest of the linker script would define sections like .text, .data, .bss, etc. */
/* This file only specifies the memory layout. A full script is provided by the HAL/PAC. */

SECTIONS
{
    /* The startup code goes first into FLASH */
    .isr_vector :
    {
        . = ALIGN(4);
        KEEP(*(.isr_vector)) /* Startup code */
        . = ALIGN(4);
    } >FLASH

    /* The program code and other data goes into FLASH */
    .text :
    {
        . = ALIGN(4);
        *(.text)           /* .text sections (code) */
        *(.text*)          /* .text* sections (code) */
        *(.rodata)         /* .rodata sections (constants, strings, etc.) */
        *(.rodata*)        /* .rodata* sections (constants, strings, etc.) */
        . = ALIGN(4);
        _etext = .;        /* define a global symbols at end of code */
    } >FLASH
}
