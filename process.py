# Processes source images in SOURCE_DIR and converts them into bitmap for use with e-ink display

from pathlib import Path
from PIL import Image, ImageOps

import hashlib

# https://www.rawpixel.com/public-domain

SOURCE_DIR = Path("images/source")
EXPORT_DIR = Path("images/export")

# Dimensions to aspect fit the image
DIMENSIONS = (800, 480)

# Re-exports all images even if they exist
EXPORT_ALL = False

# The exported image format extension
# NOTE: Don't use JPEG or other compressed format that introduces artifacts to pure B&W
EXTENSION = "bmp"


if __name__ == "__main__":
    count = 0

    for img_path in SOURCE_DIR.iterdir():
        # Ignore hidden files
        img_name = img_path.stem
        if img_name.startswith('.'):
            continue

        # Retrieve image data
        artist, title = img_name.split("+")

        # Create hashed filename based on original filename
        export_name = hashlib.md5(img_name.encode()) \
            .hexdigest()[:16] + "." + EXTENSION

        # Skip already processed images
        export_file = EXPORT_DIR / export_name
        if not EXPORT_ALL and export_file.is_file():
            continue

        # Process image to B&W and resize to specified dimensions (maintaining aspect ratio)
        img = Image.open(img_path)
        img = img.convert('1', dither=Image.NONE)
        img = ImageOps.contain(img, DIMENSIONS)
        img = ImageOps.pad(img, DIMENSIONS, color=0xFFFFFF)

        print(f"> {title} ({artist})")
        img.save(export_file)

        count += 1

    print(f"Exported {count} images")
