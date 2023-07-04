# Inkarta


This repository includes three parts for a wirelessly configurable e-ink picture frame:

1. Arduino sketch for the Inkplate ESP32-based e-paper display
2. Flask server for processing and storing artwork
3. iOS app for convenient artwork management

## Features

- Automatically changes the picture at midnight
- Enters low power mode until next refresh (or wake button triggers manual refresh) so battery lasts a *long* time.
- Flask server supports processing, storing, and retrieving artwork via REST API
- App provides an easy-to-use interface for managing artwork + metadata

## Getting Started

### Inkplate

1. Follow instructions for setting up the [Inkplate with Arduino IDE](https://github.com/SolderedElectronics/Inkplate-Arduino-library/tree/master). In the Arduino IDE:
    - Select the correct board (e.g., sketch was written for Inkplate10)
    - Select the correct port
    - Select upload speed of 115200
2. Upload `inkplate/inkplate.ino` program to the Inkplate.

## Reference

- https://inkplate.readthedocs.io/en/latest/get-started.html
- https://github.com/SolderedElectronics/Inkplate-Arduino-library/tree/master
