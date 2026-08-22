# `r_klipp_parser`: Zero-Allocation Streaming G-Code Lexer & Parser

`r_klipp_parser` is a `no_std`, high-throughput, zero-allocation G-Code parser designed for embedded MCU firmware and host engines.

---

## ⚡ Key Features

- **Zero Allocations**: Lexes tokens and extracts commands directly from streaming byte slices without dynamic heap allocations.
- **Robust Comment & Checksum Handling**: Strips inline `(...)` and `;` comments and verifies N-number line checksums (`*NN`).
- **Comprehensive Command Coverage**:
  - Motion: `G0`, `G1`, `G2`, `G3`, `G4`, `G28`, `G29`, `G38.2`, `G43`, `G44`, `G90`, `G91`, `G96`, `G97`.
  - Machine Control: `M3`, `M5`, `M82`, `M83`, `M104`, `M106`, `M109`, `M140`, `M190`, `M204`, `M220`, `M221`.
  - Pick & Place: `M800`, `M801` (valves), `M810`, `M811` (feeders).
- **Proptest Verified**: Fuzzed against millions of arbitrary byte sequences to ensure zero panics.

---

## 🧪 Testing
```bash
cargo test -p parser
cargo test -p parser --test proptest_parser
```
