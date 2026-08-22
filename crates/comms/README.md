# `r_klipp_comms`: Lock-Free Communication & Protocol Bridges

`r_klipp_comms` provides lock-free single-producer single-consumer (SPSC) message rings and asynchronous communication channels for low-latency command streaming.

---

## 📡 Key Features

- **Lock-Free SPSC Buffering**: Microsecond packet exchange between asynchronous networking tasks and high-priority interrupt service routines.
- **Embedded Channel Interfaces**: Supports non-blocking byte transports across UART, USB CDC, SPI, and CAN-FD.
