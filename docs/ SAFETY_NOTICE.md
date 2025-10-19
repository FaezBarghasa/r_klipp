<!-- File path: SAFETY_NOTICE.md -->

🚨 CRITICAL SAFETY NOTICE: r_klipp EXPERIMENTAL FIRMWARE 🚨

WARNING: This is experimental, non-production firmware. Flashing it to your 3D printer's mainboard carries significant risks, including but not limited to:

FIRE HAZARD: Incorrect thermal regulation can lead to uncontrolled heating, potentially causing components to melt or ignite.

MACHINE DAMAGE: Bugs in the motion control system could cause unexpected, rapid, or forceful movements, leading to collisions that damage your printer's mechanics.

COMPONENT DAMAGE: Firmware faults could lead to over-voltage or short-circuits, permanently damaging the mainboard or connected components (motors, heaters, etc.).

You assume all risks associated with using this firmware. The developers are not liable for any damage to property or persons.

First-Time Safe Testing Procedure

Before you connect mains power to your printer's heaters or motors, you MUST follow this procedure meticulously to verify the basic safety and functionality of the firmware.

Required Tools:

Multimeter (with continuity and temperature probe if possible)

Heat gun or hair dryer

A current-limited DC power supply (highly recommended)

Step 1: Bench Test (No Mains Power)

Disconnect All High-Power Outputs:

Unplug the power connectors for your hotend heater cartridge.

Unplug the power connectors for your heated bed.

Unplug all stepper motors.

Ensure the main PSU that powers these components is OFF and UNPLUGGED.

Flash the Firmware:

Flash the r_klipp firmware to your mainboard using a debug probe or SD card.

Power the MCU Logic Only:

Power the board's logic circuits only (usually via USB or a 5V/12V input). Do NOT connect the main 24V PSU yet.

Connect to Klipper Host:

Connect the board to your Klipper host (e.g., Raspberry Pi) and verify that it is detected (ls /dev/serial/by-id/*).

Start the Klipper service. The host should connect to the MCU without critical errors.

Step 2: Verify Thermistor Readings

Check Ambient Temperature:

In your Klipper interface (Mainsail/Fluidd), check the reported temperatures for your hotend and heated bed. They should read a plausible ambient temperature (e.g., 15-30°C).

If they read extremely high (e.g., 300°C) or low (e.g., -50°C), you have a short or an open circuit. DO NOT PROCEED. Check your wiring and firmware configuration.

Simulate Heating:

Gently warm your hotend's thermistor with a heat gun or hair dryer.

Watch the temperature reading in your Klipper interface. It should rise smoothly.

Remove the heat source. The temperature should fall smoothly back to ambient.

Repeat this process for the heated bed thermistor.

Step 3: Verify Endstop Functionality

Check Initial State:

From the Klipper console, send an QUERY_ENDSTOPS command. All endstops should report as open.

Manually Trigger Endstops:

While repeatedly sending QUERY_ENDSTOPS, manually press and hold the X-axis endstop switch. The status for that endstop should change to TRIGGERED.

Release the switch. The status should return to open.

Repeat for the Y and Z axes.

Step 4: Initial Power-On with Current Limiting (Highly Recommended)

Connect a Current-Limited PSU:

If you have a lab-style DC power supply, connect it instead of your printer's main PSU.

Set the voltage to your printer's standard (e.g., 24V) and set the current limit to a low value (e.g., 1A or 2A).

Test Heater Output:

From your Klipper interface, set the hotend temperature to a low value (e.g., 50°C).

The power supply should show a current draw.

Using a multimeter in DC voltage mode, carefully check the voltage across the hotend heater terminals on the mainboard. It should be receiving power (likely PWM-controlled, so the voltage may fluctuate).

Turn the heater off by setting its temperature to 0°C. The current draw should drop, and the voltage at the terminals should go to zero.

If at any point the current draw spikes to your limit unexpectedly, SHUT DOWN IMMEDIATELY.

Step 5: Test Failsafes

Heater Fault Test (Thermal Runaway):

With the current-limited PSU still attached, set the hotend to 50°C.

Wait for Klipper to report a "Heater not heating at expected rate" error. This verifies that the thermal runaway protection is active.

Thermistor Disconnect Test:

While the heater is OFF (target temp 0°C), carefully unplug the hotend thermistor from the mainboard.

Klipper should immediately report a "Thermistor ADC out of range" or similar critical error and enter a shutdown state.

Reconnect the thermistor and restart the firmware to clear the fault.

Repeat for the bed thermistor.

Only after successfully completing ALL of these steps should you consider connecting your printer's main power supply and attempting to control motors and heaters. Even then, proceed with extreme caution and do not leave the printer unattended during its first prints.