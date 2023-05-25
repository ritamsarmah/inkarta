import socket
import time

from soldered_inkplate10 import Inkplate
from secrets import ssid, password

# width = 1200
# height = 825
width = 300
height = 300

host = "192.168.1.5"
port = 5000
path = f"/image?w={width}&h={height}"
request = f"GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"

display = Inkplate(Inkplate.INKPLATE_1BIT)


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

    # Save image data to SD card
    display.SDCardWake()
    with open("sd/image.bmp", "wb") as f:
        f.write(data)
    display.SDCardSleep()
    print("Saved artwork")

    return data


def print_artwork():
    print("Printing artwork")
    display.drawImageFile(0, 0, "sd/image.bmp")
    # TODO: display.setRotation()
    display.display()


if __name__ == "__main__":
    display.begin()
    display.initSDCard()

    import os
    print(os.listdir("/sd"))

    # connect_wifi()
    # download_artwork()
    # print_artwork()

    # TODO: Add support for refreshing with button
    # Set schedule
