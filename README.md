# ezfd-bootloader-device

A CAN FD bootloader for STM32H5 microcontrollers written in Rust using the [Embassy](https://embassy.dev) async embedded framework. It enables remote firmware updates over a CAN FD bus without requiring a physical connection to the device.

## How It Works

On startup, the bootloader reads the MCU's unique ID to determine its node identity, configures FDCAN with extended 29-bit IDs, and waits for commands from a host (e.g. a Raspberry Pi). Once a full firmware image has been erased, written, and verified, the host sends a `Jump` command and the bootloader hands off execution to the user application at `0x08008000`.

### CAN ID Format

All frames use 29-bit extended IDs packed as:

```
[ priority 3b ][ target 5b ][ command 16b ][ source 5b ]
```

### Commands

| Command        | Value  | Direction       | Description                          |
|----------------|--------|-----------------|--------------------------------------|
| `Ping`         | `0x40` | Host → Device   | Keep-alive                           |
| `Erase`        | `0x45` | Host → Device   | Erase application flash region       |
| `EraseOk`      | `0x46` | Device → Host   | Erase complete ACK                   |
| `Write`        | `0x47` | Host → Device   | Write data chunk to flash            |
| `WriteOk`      | `0x48` | Device → Host   | Write complete ACK (echoes address)  |
| `AddressAndSize` | `0x4A` | Host → Device | Set target address and chunk size    |
| `Jump`         | `0xAAAA` | Host → Device | Jump to user application             |

### Flashing Sequence

```
Host                          Device
 |--- Erase ----------------->|
 |<-- EraseOk ----------------|
 |--- AddressAndSize -------->|
 |--- Write ----------------->|
 |<-- WriteOk ----------------|
 |--- AddressAndSize -------->|  (repeat for each chunk)
 |--- Write ----------------->|
 |<-- WriteOk ----------------|
 |--- Jump ------------------>|
                               → boots user app
```

## Hardware

- **MCU**: STM32H533RETx (Cortex-M33, 240 MHz)
- **CAN peripheral**: FDCAN2 on PB12 (TX) / PB13 (RX)
- **Bitrate**: 1 Mbps nominal / 5 Mbps data (CAN FD with BRS)
- **Node IDs**: Nuc1 (`0x06`), Nuc2 (`0x07`) — identified by MCU UID at startup

## Building & Flashing

Requires Rust with the `thumbv8m.main-none-eabihf` target and [probe-rs](https://probe.rs) with a compatible debug probe.

```bash
# Build
cargo build

# Flash and run (via probe-rs)
cargo run
```

RTT logs are printed over the debug probe connection and viewable in the probe-rs terminal or any RTT client.

## Configuring Your User Application

The bootloader occupies the first 32KB of flash (`0x08000000`–`0x08007FFF`). Your application must be configured to start at `0x08008000`.

See **[USER_APP_SETUP.md](USER_APP_SETUP.md)** for the required linker script and `VECT_TAB_OFFSET` changes.

## Memory Map

```
0x08000000  ┌─────────────────────┐
            │   Bootloader (32KB) │
0x08008000  ├─────────────────────┤
            │  User Application   │
            │      (480KB)        │
0x08080000  └─────────────────────┘
```

---

## Roadmap

- [ ] **Auto-jump timeout** — Automatically boot into the user application after a configurable timeout if no CAN activity is detected, so the device does not wait indefinitely for a host connection on normal power cycles.

- [ ] **Multi-interface support** — Abstract the transport layer to support interfaces beyond FDCAN (e.g. UART, USB, classic CAN), allowing the same bootloader protocol to run over different physical buses.

- [ ] **Broader STM32 compatibility** — Generalize clock configuration, flash region definitions, and peripheral selection to support other STM32 families beyond the H533 (e.g. STM32G4, STM32H7, STM32U5).

- [ ] **User integration abstraction** — Provide a cleaner interface for node identity, address mapping, and command handling so the bootloader can be integrated into a project with minimal changes to a single configuration file.

- [ ] **TUI** — Make a nice terminal-based user interface for flashing and interacting with the bootloader.

- [ ] **Size Reduction** — Reduce binary size without compromising functionality. I think it can get down to sub 16 kB.
