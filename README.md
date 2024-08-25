# Inkarta

This repository contains two components for a wirelessly configurable e-ink picture frame:

1. Arduino sketch for the Inkplate ESP32-based e-paper display.
2. Server for managing and hosting images via a web interface.

## Features

- Automatically updates the picture at midnight.
- Enters low power mode until the next refresh or a manual refresh is triggered by pressing the wake button, extending battery life for several months.
- Server handles image processing and storage to a SQLite database.
- Web dashboard for image management.

## Getting Started

### Server

The server requires a [Rust](https://www.rust-lang.org/) installation in order to build.

1. Navigate to the `server/` directory.
2. If you're cross-compiling for a different target architecture, you may prefer to use [`cross`](https://github.com/cross-rs/cross). For example, for Raspberry Pi 64-bit running Debian: `cross build --release --target aarch64-unknown-linux-gnu`. Otherwise run `cargo build --release`.
3. Deploy the binary created in `target` to your server.

### Inkplate

#### Prerequisites

1. Install the `arduino-cli`

    ```sh
    brew install arduino-cli
    ```

2. Create `inkplate/secrets.h` with your Wi-Fi credentials and server info:

    ```c
    const char *ssid = "YOUR_WIFI_SSID";
    const char *password = "YOUR_WIFI_PASSWORD";

    const char *host = "YOUR_SERVER_IP"
    const uint16_t port = "YOUR_SERVER_PORT";
    ```

#### Installation

1. Connect the Inkplate to your computer via USB.
2. Update `sketch.yaml` with your appropriate Inkplate `fqbn` and `port`:

    ```sh
    arduino-cli board list # Identify device's port
    ```

3. Compile and upload the `inkplate/inkplate.ino` sketch to the Inkplate.

    ```sh
    arduino-cli compile --verbose --upload --profile default
    ```

> [!NOTE]
> If you encounter a "Bad CPU type in executable" error on Apple Silicon, install Rosetta using `softwareupdate --install-rosetta`

## Reference

- https://inkplate.readthedocs.io/en/latest/get-started.html
- https://github.com/SolderedElectronics/Inkplate-Arduino-library/tree/master
