# Inkarta

This repository includes two parts for a wirelessly configurable e-ink picture frame:

1. Arduino sketch for the Inkplate ESP32-based e-paper display
2. Web server for image serving and dashboard for image management

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

2. Update your `arduino-cli.yaml` to include the Inkplate board definitions:

```yaml
board_manager:
  additional_urls:
    - https://github.com/SolderedElectronics/Dasduino-Board-Definitions-for-Arduino-IDE/raw/master/package_Dasduino_Boards_index.json
```

3. Install the Inkplate board definition:

```sh
arduino-cli core update-index
arduino-cli core search inkplate # Check that Inkplate definition is available
arduino-cli core install Inkplate_Boards:esp32
```

4. Install the Inkplate Arduino library:

```sh
arduino-cli lib install InkplateLibrary
```

5. Create `inkplate/secrets.h` with your Wi-Fi credentials:

    ```c
    const char *ssid = "YOUR_WIFI_SSID";
    const char *password = "YOUR_WIFI_PASSWORD";
    ```

#### Installation

1. Connect the Inkplate to your computer via USB.
2. Attach via appropriate Inkplate FQBN and port; for example, for the Soldered Inkplate10:

```sh
arduino-cli board search Inkplate10 # Identify FQBN for device
arduino-cli board list # Identify port device is connected to
arduino-cli board attach --fqbn Inkplate_Boards:esp32:Inkplate10V2 --port /dev/cu.usbserial-2140
```

3. Compile and upload the `inkplate/inkplate.ino` sketch to the Inkplate. The following commands also set the correct upload speed and erases the flash before uploading the sketch:

```sh
# Option 1: Compile and upload separately
arduino-cli compile --fqbn Inkplate_Boards:esp32:Inkplate10V2 inkplate
arduino-cli upload --fqbn Inkplate_Boards:esp32:Inkplate10V2:UploadSpeed=115200,EraseFlash=all --port /dev/cu.usbserial-2140 inkplate

# Option 2: Compile and upload together
arduino-cli compile --fqbn Inkplate_Boards:esp32:Inkplate10V2:UploadSpeed=115200,EraseFlash=all --port /dev/cu.usbserial-2140 --upload inkplate
```

> [!TIP]
> You can list all supported board options using `arduino-cli board details --fqbn Inkplate_Boards:esp32:Inkplate10V2`

### Server

The server requires a [Rust](https://www.rust-lang.org/) installation in order to build.

1. Run `cargo build --release` in the project directory.
2. Copy the binary at `target/release/server` to your server.

## Reference

- https://inkplate.readthedocs.io/en/latest/get-started.html
- https://github.com/SolderedElectronics/Inkplate-Arduino-library/tree/master
