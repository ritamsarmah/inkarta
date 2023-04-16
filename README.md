# Picosso

An e-ink picture frame controlled by a Raspberry Pi Pico W.

## Features

- [ ] Downloads an image from the specified server
- [ ] Changes the picture every day at midnight
- [ ] Powers down until next refresh time

Find Pico ("usbmodem")
ls /dev/cu.usb*

Connect to Pico
screen /dev... 115200

## Reference

https://github.com/raspberrypi/pico-examples/blob/master/pico_w/wifi/tcp_client/picow_tcp_client.c
https://github.com/raspberrypi/pico-examples/blob/master/flash/program/flash_program.c
https://www.makermatrix.com/blog/read-and-write-data-with-the-pi-pico-onboard-flash/
https://github.com/waveshare/Pico_ePaper_Code/blob/main/c/lib/e-Paper/EPD_7in5_V2.c
