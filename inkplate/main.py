import machine
import network
import socket
import time

from soldered_inkplate10 import Inkplate
from secrets import ssid, password

display = Inkplate(Inkplate.INKPLATE_1BIT)

# 1200 x 825 BMP is 125KB
# width = display.height()
# height = display.width()
width = 64
height = 64

host = "192.168.1.5"
port = 5000
path = f"/image?w={width}&h={height}"
request = f"GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"

# sleep_time_us = 8.64e+10
sleep_time_us = 1e+7


def connect_wifi():
    sta_if = network.WLAN(network.STA_IF)
    if not sta_if.isconnected():
        print(f"Connecting to \"{ssid}\"")
        sta_if.active(True)
        sta_if.connect(ssid, password)
        while not sta_if.isconnected():
            pass

    print(f"Connected to \"{ssid}\"")


def disconnect_wifi():
    sta_if = network.WLAN(network.STA_IF)
    sta_if.disconnect()
    print(f"Disconnected from \"{ssid}\"")


def download_artwork():
    s = socket.socket()
    s.connect((host, port))
    s.send(bytes(request, "utf8"))

    response = bytearray()

    while True:
        data = s.recv(100)
        if data:
            response += data
        else:
            break

    s.close()

    print("Downloaded artwork")
    headers, data = bytes(response).split(b"\r\n\r\n")

    return data


# NOTE: Parsing and displaying full image can take up to 5 minutes
def print_artwork(bmp):
    print("Parsing bitmap")

    # Calculate the number of bytes per row, taking into account row padding
    bytes_per_row = (width + 7) // 8  # 1 bit per pixel
    row_padding = (4 - (bytes_per_row % 4)) % 4  # Row padding in bytes
    bytes_per_row += row_padding

    bmp_offset = bmp[10]
    for y in range(height):
        row_start = y * bytes_per_row

        for offset, x in enumerate(range(0, width, 8)):
            # Get next byte in row (which holds 8 pixels)
            byte = bmp[bmp_offset + row_start + offset]

            # Draw each pixel bit from the byte
            for i, bit in enumerate(range(7, -1, -1)):
                color_bit = (byte >> bit) & 0x01
                color = display.BLACK if color_bit == 0 else display.WHITE
                display.drawPixel(x + i, height - y - 1, color)

    # Update screen
    print("Displaying artwork")
    display.display()


if __name__ == "__main__":
    display.begin()
    display.setRotation(3)

    while True:
        connect_wifi()

        bmp = download_artwork()
        print_artwork(bmp)

        disconnect_wifi()

        print("Sleeping...")
        machine.deepsleep(sleep_time_us)
