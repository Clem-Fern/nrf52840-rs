# Personnal nRF52840 Rust projects

### Hardware details
- Custom nRF52840 board
- 1 Blue LED `P1_11`
- 1 RGB LED (red `P0_06`, green `P0_07`, blue `P0_08`)
- 1 Button `P1_01`

### Getting started
- Rust required (`thumbv7em-none-eabihf` target)

- `rustup target add thumbv7em-none-eabihf`

- [probe-rs](https://probe.rs/docs/getting-started/installation/)

- []

- Build and flash:
  ```
  cargo build --target thumbv7em-none-eabihf
  probe-rs run target/thumbv7em-none-eabihf/debug/nrf52840-blink
  ```