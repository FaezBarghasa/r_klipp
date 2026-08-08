# MKS SKIPR: SD Card Flashing Layout

The MKS SKIPR board contains a bootloader that allows for easy firmware updates by placing a compiled binary onto an SD card. For detailed instructions on how to flash the firmware via SD card, please see the [flashing guide](./flash-via-sd.md).

## Naming Convention

To trigger the flashing process, the firmware binary must be named exactly:

`mks_skipr.bin`

## SD Card Structure

1.  Format the SD card to FAT32.
2.  Place the `mks_skipr.bin` file in the root directory of the SD card.
3.  Ensure no other `.bin` files are in the root directory to avoid confusion.

The SD card should look like this:

```
/
└── mks_skipr.bin
```

## Flashing Process

1.  Power off the MKS SKIPR board completely.
2.  Insert the prepared SD card into the board's SD card slot.
3.  Power on the board.

The bootloader will detect `mks_skipr.bin`, flash the new firmware, and then rename the file to `mks_skipr.cur` to prevent re-flashing on the next boot. The board will then boot into the newly flashed firmware.