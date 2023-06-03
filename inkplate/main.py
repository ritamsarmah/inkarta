import socket
import time

from soldered_inkplate10 import Inkplate
from secrets import ssid, password

display = Inkplate(Inkplate.INKPLATE_1BIT)

# 1200 x 825 BMP is 125KB
# width = display.width()
# height = display.height()
width = 100
height = 100

host = "192.168.1.5"
port = 5000
path = f"/image?w={width}&h={height}"
request = f"GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"


def connect_wifi():
    import network

    sta_if = network.WLAN(network.STA_IF)
    if not sta_if.isconnected():
        print(f"Connecting to \"{ssid}\"")
        sta_if.active(True)
        sta_if.connect(ssid, password)
        while not sta_if.isconnected():
            pass

    print(f"Connected to \"{ssid}\"")


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


# NOTE: Parsing and displaying a large image will take a few minutes
# TODO: Consider optimizations for this
def print_artwork(bmp):
    print("Parsing artwork")

    # Calculate the number of bytes per row, taking into account row padding
    bytes_per_row = (width + 7) // 8  # 1 bit per pixel
    row_padding = (4 - (bytes_per_row % 4)) % 4  # Row padding in bytes
    bytes_per_row += row_padding

    bmp_offset = bmp[10]
    for y in range(0, height):
        for x in range(0, width):
            # Calculate the offset to the current pixel
            # Each byte represents 8 pixels
            px_offset = (y * bytes_per_row) + (x // 8)
            color_bit = (bmp[bmp_offset + px_offset] >> (7 - (x % 8))) & 0x01

            color = display.BLACK if color_bit == 0 else display.WHITE
            display.drawPixel(x, height - y - 1, color)

    # Update screen
    print("Displaying artwork")
    display.display()


if __name__ == "__main__":
    display.begin()
    display.setRotation(3)

    connect_wifi()
    bmp = download_artwork()
    print_artwork(bmp)

    # TODO: Add support for refreshing with button
    # Set schedule
