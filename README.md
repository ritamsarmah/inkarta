# Inkarta

This repository includes two parts for a wirelessly configurable e-ink picture frame:

1. Arduino sketch for the Inkplate ESP32-based e-paper display
2. Web server for managing image database via web dashboard

## Features

- Automatically changes the picture at midnight
- Enters low power mode until next refresh (or wake button triggers manual refresh) so battery lasts a *long* time.
- Flask server supports processing, storing, and retrieving images via REST API
- App provides an easy-to-use interface for managing images and metadata

## Getting Started

### Inkplate

#### Prerequisites

1. Install the `arduino-cli`

    ```sh
    brew install arduino-cli
    ```

2. Create `inkplate/secrets.h` with your Wi-Fi credentials:

    ```c
    const char *ssid = "YOUR_WIFI_SSID";
    const char *password = "YOUR_WIFI_PASSWORD";
    ```

#### Installation

1. Connect the Inkplate to your computer via USB.
2. Update `sketch.yaml` with the appropriate Inkplate `fqbn` and `port`; for example, for the Soldered Inkplate10:

    ```sh
    arduino-cli board search Inkplate10 # Identify fqbn for device
    arduino-cli board list # Identify port device is connected to
    ```

> [!TIP]
> You can list all supported board options using `arduino-cli board details --fqbn <FQBN>` (e.g., `UploadSpeed`, `EraseFlash`). Modify them in `sketch.yaml` if needed.

3. Compile and upload the `inkplate/inkplate.ino` sketch to the Inkplate.

    ```sh
    arduino-cli compile --verbose --upload --profile default
    ```

> [!NOTE]
> If you encounter a "Bad CPU type in executable" error on macOS with Apple Silicon, install Rosetta using `softwareupdate --install-rosetta`

### Server

The server requires a [Rust](https://www.rust-lang.org/) installation in order to build.

1. Run `cargo build --release` in the project directory.
2. Copy the binary at `target/release/server` to your server.

## Reference

- https://inkplate.readthedocs.io/en/latest/get-started.html
- https://github.com/SolderedElectronics/Inkplate-Arduino-library/tree/master
