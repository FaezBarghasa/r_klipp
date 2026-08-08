<!-- File path: docs/profiling.md -->

<!--
AI-generated comment:
This file was created by an AI assistant to document the performance profiling workflow.
Source files for context: N/A (new file)
-->

Performance Profiling Guide

This guide explains how to profile the CPU usage of both the host-side and firmware-side code of the r_klipp project to identify performance bottlenecks ("hot spots"). We will use cargo-flamegraph for this purpose.

1. Installation

First, you need to install cargo-flamegraph and its dependencies. On Linux, this typically requires perf.

# Install flamegraph subcommand
cargo install flamegraph

# On Debian/Ubuntu, install perf
sudo apt-get install linux-perf


You may need to adjust system settings to allow perf to run for non-root users:

sudo sh -c 'echo -1 > /proc/sys/kernel/perf_event_paranoid'


2. Profiling the Host (klipper-host)

The host-side code, which handles G-code parsing, motion planning, and communication, can be profiled directly on your development machine.

Command

Run the klipper-host binary with your desired arguments (e.g., a printer config file) under cargo flamegraph.

# From the workspace root
cargo flamegraph --release -p klipper-host -- run crates/compat-layer/tests/ender3.cfg


Process

cargo flamegraph will compile and run the klipper-host application in release mode.

It will use perf to sample the application's stack traces for a period of time.

Let the application run for a while, or perform actions that you want to profile (e.g., load a G-code file through the API).

Once you stop the application (e.g., with Ctrl+C), flamegraph will process the collected samples and generate an interactive SVG file named flamegraph.svg.

3. Profiling the Firmware (via Simulator)

Since we cannot run perf directly on the MCU, we profile the firmware logic by running it within the simulator (sim crate). This allows us to find hot spots in algorithms like motion planning and PID control.

Command

Run the simulator example under cargo flamegraph.

# From the workspace root
cargo flamegraph --release -p sim --example run_sim


Process

This command will compile and run the run_sim example in release mode.

The run_sim example simulates both the MCU and a host, executing a pre-defined sequence of commands.

flamegraph will sample the execution and, upon completion, generate a flamegraph.svg file.

4. Interpreting the Flame Graph

Open the generated flamegraph.svg file in a web browser.

X-axis (Width): Represents the percentage of total CPU time a function was on the stack. Wider bars are functions that consume more CPU time and are the primary candidates for optimization.

Y-axis (Stack Depth): Shows the call stack. The function at the bottom called the function above it, and so on.

Color: The color is usually not significant for performance analysis (it can be randomized to differentiate adjacent blocks).

To find bottlenecks, look for the widest bars, especially those near the top of the graph. Clicking on a bar will zoom in on its call stack, allowing you to explore the performance of its children functions in more detail.

By analyzing the flame graph, you can pinpoint exactly which functions are consuming the most CPU cycles and focus your optimization efforts where they will have the most impact.