# BigTreeTech PI-TS35 Panel Configuration

This document describes how to wire and configure the BigTreeTech PI-TS35 touchscreen panel for use with a host system running a Debian-based Linux distribution (such as Raspberry Pi OS or Armbian).

## Wiring

The PI-TS35 uses a ribbon cable to connect to a DSI display port.

*   **For Raspberry Pi**: Connect the ribbon cable to the DSI port on the Pi. Ensure the cable is oriented correctly.
*   **For MKS SKIPR**: The SKIPR has an integrated compute module with a compatible display connector. Connect the ribbon cable directly to this port.

## Host Configuration

To enable the display and touchscreen, you need to add a device tree overlay to your system's boot configuration.

### 1. Edit `/boot/config.txt`

Open the `/boot/config.txt` file with a text editor:

```bash
sudo nano /boot/config.txt
```

### 2. Add Overlay

Add the following lines to the end of the file to enable the I2C bus for the touchscreen and configure the panel:

```
# Enable I2C for touchscreen
dtparam=i2c_arm=on

# Enable DSI display panel
dtoverlay=vc4-kms-v3d
dtoverlay=btt-pi-ts35
```

### 3. Save and Reboot

Save the file (Ctrl+X, then Y, then Enter) and reboot the system:

```bash
sudo reboot
```

Upon rebooting, the system should recognize the display and the touchscreen should be active.

## Udev Rules

The `udev-rules` directory contains a rules file that can be used to configure the touchscreen input device. This is optional, but it can be useful for setting properties like touch calibration. To install the rules, copy the file to the `/etc/udev/rules.d/` directory:

```bash
sudo cp udev-rules/99-btt-pi-ts35.rules /etc/udev/rules.d/
```

After copying the file, reboot your system for the changes to take effect.