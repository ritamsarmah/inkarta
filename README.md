# Inkarta

This repository includes two parts for a wirelessly configurable e-ink picture frame:

1. Arduino sketch for the Inkplate ESP32-based e-paper display
2. Server for hosting and managing images via web interface

## Features

- Automatically changes the picture at midnight
- Enters low power mode until next refresh (or wake button triggers manual refresh) so battery lasts a *long* time.
- Server supports processing, storing, and retrieving images
- Web dashboard for image management

## Getting Started

### Server

The server requires a [Rust](https://www.rust-lang.org/) installation in order to build.

1. Run `cargo build --release` in the project directory.
2. Deploy the binary at `target/release/server` to your server.

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
2. Update `sketch.yaml` with your appropriate Inkplate `fqbn` and `port`

    ```sh
    arduino-cli board list # Identify port device is connected to
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
