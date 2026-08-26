---
slug: qemu-slint-full-test-matrix
status: awaiting-approval
intent: clear
review_required: false
pending-action: write .omo/plans/qemu-slint-full-test-matrix.md after explicit user okay
approach: Harden the two existing test-harness scripts to exact contract + build exhaustive per-crate/module/function test matrix for r_klipp with upstream-compatibility conformance (Klipper, KlipperScreen, Moonraker, Fluidd), executed via QEMU Cortex-M emulation and Dockerized Embedded-Linux + Slint software rendering.
---

# Draft: qemu-slint-full-test-matrix

## Components (topology ledger)
<!-- id | outcome (one line) | status | evidence path -->

| id | component | outcome | status | evidence |
|----|-----------|---------|--------|----------|
| C1 | QEMU harness `tools/run_qemu_test.sh` | builds thumbv7em-none-eabihf release, runs qemu lm3s6965evb/cortex-m3, prints serial log, exit 0 iff `TESTS PASSED` | active (exists, needs hardening to exact spec) | tools/run_qemu_test.sh:1-107 |
| C2 | Docker Slint harness `tools/test_slint_docker.sh` | compose up --build, wait healthy, grep logs for panics/render failures, exit code contract | active (exists, needs hardening) | tools/test_slint_docker.sh:1-101 |
| C3 | Compose stack `docker-compose.yml` + `tools/docker/Dockerfile.host` | xvfb-run + LIBGL_ALWAYS_SOFTWARE=1 + SLINT_BACKEND=software at 480x320 | active (healthcheck is weak) | docker-compose.yml:14-19, tools/docker/Dockerfile.host:28-45 |
| C4 | MCU firmware self-test binary `qemu_runner` | bare-metal no_std runner printing TESTS PASSED over UART0 DR + semihosting exit; only 4 smoke tests today | active (extend to subsystem coverage) | crates/klipper-mcu-firmware/src/bin/qemu_runner.rs:1-66 |
| C5 | r_klipp unit/integration test matrix | every module/function of 31 workspace members covered or explicitly waived | gap — 133 tests/60 files, many crates zero | grep #[test] count; .cargo/config.toml aliases |
| C6 | Upstream conformance fixtures | klipper-proto↔klippy protocol, klipper-host↔moonraker API/WS, host-ui↔KlipperScreen panels, host-server WS↔fluidd expectations | partial (klipper-host/tests/protocol_compatibility.rs exists, 1 test) | references/{moonraker,fluidd,KlipperScreen} vendored; Klipper3d/klipper NOT vendored |
| C7 | Embedded Linux (aarch64) job | host-server/host-ui cross-built for aarch64 + run under ARM64 container via binfmt/qemu-user | gap — none today | README target section |
| C8 | `.agent_rules.md` usage docs | both tools documented with invocation + exit contracts + CI wiring | active (exists; refresh after hardening) | .agent_rules.md:23-55 |

## Open assumptions (announced defaults)

| assumption | adopted default | rationale | reversible? |
|---|---|---|---|
| "test every module and function of …Klipper3d/klipper…" means r_klipp modules fully tested WITH upstream projects as compatibility oracles — not porting upstream Python test suites into this repo's CI | yes | upstream repos have their own CI; r_klipp is the Rust reimplementation being validated against their wire/API contracts | yes |
| Tools exist already → audit-and-harden surgically instead of rewrite from scratch | yes | existing files already match ~90% of stated contract; behavioral.md surgical-changes rule | yes |
| Klipper3d/klipper missing from references/ → vendor as read-only reference clone for protocol fixture extraction | yes | other three repos already vendored there; symmetric layout | yes |
| "Embedded Linux" = aarch64 cross-target job for host-server/host-ui (MKS SKIPR host is Cortex-A53) run in ARM64 docker via qemu binfmt | yes | matches documented hardware (.agent_rules.md:14); user said "QEMU and Embedded Linux" | yes |
| Coverage gate = every public function has ≥1 passing test or an explicit waiver row in matrix doc (not blanket 80% line coverage, which testing.md rules would demand but no_std crates make awkward) | hybrid: waiver-per-item matrix + cargo-llvm-cov report where measurable | deterministic, auditable, honest about no_std constraints | yes |

## Findings (cited - path:lines)

1. **Workspace**: 31 members — Cargo.toml:4-35 (`crates/actors…crates/tools`, `crates/bin/r_klipp_firmware`, `crates/bin/r_klipp_host`, `host-server`, `host-ui`).
2. **Test density**: 133 `#[test]`/`#[tokio::test]` across 60 files; hotspots: klipper-proto (~13), mcu-firmware test dir (13+), legacy src/control/mpcc; many members (actors, automation, comms, hal, io, motors, compat-layer, driver-*…) show ≤2 or zero.
3. **run_qemu_test.sh current state**: checks qemu-system-arm/cargo in PATH (:18-26), special-cases pkg `klipper-mcu-firmware`/bin `qemu_runner` (:29-31), fuzzy fallback binary discovery (:50-56), builds release (:43-48), runs `-machine lm3s6965evb -cpu cortex-m3 -nographic -serial mon:stdio -semihosting-config enable=on,target=native -kernel` with timeout (:72-80), RETRIES on second machine type (:83-91) — deviates from strict spec (spec says lm3s6965evb only), prints serial log block (:95-97), exits 0 iff grep `TESTS PASSED` (:100-107).
4. **qemu_runner.rs**: `no_main`, UART0 DR @0x4000C000 volatile writes (:16-27), banner + 4 hardcoded PASS lines (codec CRC, thermal runaway, sensor OOB, stepper DMA timing) :39-58, prints `TESTS PASSED` :57, semihosting SYS_EXIT :60+.
5. **docker-compose.yml**: service `slint-ui-test`, env LIBGL_ALWAYS_SOFTWARE=1 / SLINT_BACKEND=software / DISPLAY_RESOLUTION=480x320 (:8-13); healthcheck `pgrep -f xvfb || … || exit 0` (:15) — always-green healthcheck, must be fixed to real signal.
6. **Dockerfile.host**: rust:1.80-slim-bookworm, installs xvfb + mesa GL/X11/font deps (:5-26), ENV LIBGL_ALWAYS_SOFTWARE=1 + SLINT_BACKEND=software (:29-33), CMD runs `cargo test -p host-server` then `xvfb-run -s '-screen 0 480x320x24' cargo test -p host-ui` (:45). Meets xvfb-run + LIBGL requirements.
7. **test_slint_docker.sh current state**: detects compose v1/v2 (:22 area), `compose up --build -d`, polls health/status with MAX_WAIT_SEC, captures logs, greps `(panicked at|fatal runtime error|Slint rendering error|UI test failure|SIGSEGV|core dumped|failed to initialize rendering backend)` (:73), checks container exit code + healthy-within-timeout, FAILURES aggregation → exit contract (:70-101). Needs: verify it actually fails when unhealthy (healthcheck bug above defeats it).
8. **`.agent_rules.md`** already documents both tools incl. exit-code contracts (:23-55) — needs refresh only where behavior changes.
9. **Cargo aliases** available for CI wiring: `cargo lint`, `cargo lint-mcu`, `cargo fmt-check`, `cargo deny-check`, `cargo test-core` (.cargo/config.toml:10-15).
10. **references/** contains moonraker, fluidd, KlipperScreen (full clones); Klipper3d/klipper absent (fdx-ls references/, glob references/klipper* → none).
11. **Existing integration seams**: crates/klipper-host/tests/{integration,protocol_compatibility}.rs; host-ui/tests/{ui_test,klipperscreen_panel_tests}.rs; host-ui/src/lib.rs has 2 slint tests; tests/integration/motion_integration_test.rs.

## Decisions (with rationale)

- **D1 Harden-not-rewrite** both scripts: keep working structure, remove deviations from stated contract (drop the second QEMU machine retry OR keep as opt-in flag — plan will make lm3s6965evb the single default path; strict-spec first, extras behind flags). Rationale: surgical-change rule + user gave exact contract.
- **D2 Fix healthcheck to a real probe**: replace always-exit-0 pgrep chain with a command that verifies the UI/server process is alive AND a marker file/log line from test completion; script then treats `healthy` as meaningful. Without this, C2's "wait for healthy" step is vacuous.
- **D3 Vendor Klipper3d/klipper** into references/klipper (shallow read-only clone) purely as fixture oracle for klipper-proto conformance (serial/CAN framing, CRC16-xmodem, message schema from klippy/serialhdl.py + msgproto.py). Never built/modified.
- **D4 Test-matrix-first authoring order**: inventory pass generates `.omo/plans/…` companion `docs/testing/test_matrix.md` (crate → module → public fn → test id(s) or waiver reason), then authoring waves fill gaps core-first: parser, motion, kinematics, thermal, safety, klipper-proto → drivers/hal/io/comms → bins/host-server/host-ui → sim/compat-layer/actors/automation.
- **D5 QEMU depth split**: firmware crate gets compile-gated unit tests (std-host `cfg(test)`) + extended `qemu_runner` subsystem smoke (parser round-trip, proto frame encode/decode, thermal PID step, kinematics transform, safety interlock trip) since full embassy executor can't run on lm3s6965evb (Cortex-M3 vs F4 target mismatch — runner is deliberately M3-compatible minimal). Document this constraint in matrix waivers.
- **D6 aarch64 job**: new `tools/test_embedded_linux.sh` (or extend C2 script w/ flag) using docker `--platform linux/arm64` + qemu binfmt to run host-server tests on ARM64, proving Embedded Linux SBC compatibility.
- **D7 Exit-contract discipline**: every tool exits 0 only on positive evidence (TESTS PASSED token / clean log scan + healthy + exit 0), 1 otherwise, captured logs printed to stdout and persisted under `target/test-logs/`.

## Scope IN

- Audit + harden `tools/run_qemu_test.sh`, `tools/test_slint_docker.sh`; chmod +x both.
- Fix `docker-compose.yml` healthcheck; verify/keep Dockerfile xvfb-run + LIBGL_ALWAYS_SOFTWARE=1.
- Update `.agent_rules.md`: usage, prerequisites (rustup target add thumbv7em-none-eabihf; qemu-system-arm; docker w/ compose v2; binfmt for arm64), exit contracts, CI wiring examples, troubleshooting.
- Extend `qemu_runner.rs` subsystem self-tests (D5 list).
- Generate exhaustive module/function inventory + test matrix doc with waivers.
- Author missing unit/integration/property tests across all 31 members until matrix complete (waiver rows allowed only with justification).
- Upstream conformance fixtures (D3): klippy serial protocol vectors, moonraker JSON-RPC/websocket request-response fixtures, fluidd websocket store shape expectations, KlipperScreen panel-state mapping tests.
- New aarch64 Embedded Linux test job (D6).
- Wire everything into one entry point: extend `tools/run_fullstack_hardware_sim.sh` phases to call hardened tools + new jobs.

## Scope OUT (Must NOT have)

- MUST NOT modify/build any repo under `references/` (read-only oracles).
- MUST NOT change product behavior of workspace crates except adding test code + `#[cfg(test)]` helpers + the qemu_runner binary extension.
- MUST NOT add new external dependencies beyond dev-deps needed by tests (justify each in matrix doc).
- MUST NOT weaken lint gates (`cargo lint`, `lint-mcu`, `deny-check`) — tests must pass them too.
- MUST NOT require physical hardware — everything runs on QEMU/Docker/xvfb.
- MUST NOT port upstream Python test suites wholesale into r_klipp.

## Open questions

None blocking. Assumptions D1-D7 announced above; veto any at the gate.

## Approval gate

status: awaiting-approval
approach: see frontmatter
next workflow action: on user okay → rerun scaffold without --draft-only → append task batches into ## Todos → hand off for execution via $start-work.
