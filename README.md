# Picosso

An e-ink picture frame using the Inkplate 10.

## Features

- [ ] Server processes 3-bit grayscale (black, white, and six different shades of gray)
- [ ] Downloads an image from the specified server
- [ ] Changes the picture every day at midnight
- [ ] Powers down until next refresh time

- Erase ESP32 Flash
```
esptool.py --chip esp32 --port /dev/cu.usbserial-110 write_flash -z 0x1000 esp32spiram-20220117-v1.18.bin
```

- Copy library files
```
python3 pyboard.py --device /dev/tty.usbserial-110 -f cp PCAL6416A.py soldered_inkplate10.py image.py shapes.py gfx.py gfx_standard_font_01.py :
```

- Run an example file
```
python3 pyboard.py --device /dev/tty.usbserial-110 "Examples/Soldered_Inkplate10/basicBW.py"
```

- Run script
```
python3 pyboard.py --device /dev/tty.usbserial-110 "main.py"
```


## Reference

https://inkplate.readthedocs.io/en/latest/get-started.html
https://github.com/SolderedElectronics/Inkplate-micropython/tree/master/Examples/Inkplate10