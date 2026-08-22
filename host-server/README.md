# `host-server`: Moonraker & OpenPnP REST/WebSocket Server

`host-server` is a multi-threaded host runtime powered by **Actix-Web**, **Tokio**, and embedded **SurrealDB**. It serves as the master coordinator connecting client UIs, OpenPnP, Moonraker web clients, and real-time microcontroller serial/CAN bridges.

---

## 🌐 Endpoints & Capabilities

### 1. Moonraker-Compatible REST API
- `GET /printer/info`: Printer identification and state.
- `GET /printer/objects/query`: Live telemetry query (temperatures, position, homing state).
- `POST /printer/gcode/script`: G-Code execution entrypoint.
- `GET /server/info`: Server version and system health.
- `GET /server/files/list`: Uploaded G-Code file catalog.

### 2. High-Speed WebSocket Telemetry
- `ws://0.0.0.0:7125/websocket`: Real-time bidirectional streaming of temperature readings, toolhead coordinates, and motion queue saturation.

### 3. OpenPnP Bridge (`openpnp.rs`)
- High-speed TCP socket bridge translating OpenPnP G-Code syntax (`M204`, `M800`, `M801`, `M810`, `M811`, `G4 P...`) to native RPC packets.

### 4. Embedded SurrealDB Persistence
- Stores G-Code metadata, time-series telemetry summaries, and machine configurations in `./data/r_klipp.db`.

---

## 🚀 Running the Server
```bash
cargo run -p host-server
```
