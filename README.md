# Inkarta

A self-hostable, wireless e-ink picture frame system for the [Soldered Inkplate 10](https://soldered.com/product/inkplate-10-9-7-e-paper-board-copy/).

## Features

This repository contains two components:

1. `inkplate` - Arduino sketch for the Soldered Inkplate, an ESP32-based e-paper display:
    - Displays pictures hosted on server.
    - Automatically updates the picture at midnight.
    - Enters low power mode until the next scheduled refresh or a manual refresh (via wake button), extending battery life to several months.
2. `server` - Server written with Go + HTMX:
    - Web dashboard for uploading and managing images.
    - Handles image processing and storage to a SQLite database.
    - Produces single binary for easy deployment.

## Getting Started

### Server

The server requires a [Go](https://go.dev) installation in order to run.

1. Navigate to the `server/` directory.
2. Run `go run .`

See `deploy.sh` for deploying the binary to a Raspberry Pi server.

### Inkplate

#### Prerequisites

1. Install the `arduino-cli`

    ```sh
    brew install arduino-cli
    ```

2. Create `inkplate/secrets.h` with your Wi-Fi credentials and server info:

    ```c
    #define SSID "YOUR_WIFI_SSID"
    #define PASSWORD "YOUR_WIFI_PASSWORD"
    #define SERVER_ADDRESS "YOUR_SERVER_ADDRESS"
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
