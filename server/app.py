from hashlib import md5
from io import BytesIO
from pathlib import Path
from PIL import Image, ImageOps

from flask import Flask, abort, request, send_file

import json
import random

IMAGES_DIR = Path('images/')
ALLOWED_EXTENSIONS = {'bmp', 'png', 'jpg', 'jpeg', 'tiff', 'tif'}
DB_PATH = Path('db.json')

# Flask
app = Flask(__name__)
app.config['UPLOAD_FOLDER'] = IMAGES_DIR
app.config['MAX_CONTENT_LENGTH'] = 16 * 1000 * 1000  # Limit uploads to 16 MB

# Database
if not DB_PATH.exists():
    DB_PATH.write_text("{\"artworks\":{}, \"next\":\"\"}")

with DB_PATH.open("r") as f:
    db = json.load(f)

''' Database '''


def save_db(db):
    with DB_PATH.open("w+") as f:
        json.dump(db, f)


''' Utilities '''


def has_valid_extension(filename):
    return '.' in filename and \
           filename.rsplit('.', 1)[1].lower() in ALLOWED_EXTENSIONS


def image_path(identifier):
    return IMAGES_DIR / f"{identifier}.bmp"


''' Routes '''


@app.route("/fetch", methods=['GET'])
def fetch():
    if not (identifier := request.args.get('id', type=str)):
        abort(400, description="Invalid file identifier")

    if identifier not in db['artworks']:
        abort(400, description="File identifier not found")

    return db['artwork']


@app.route("/next", methods=['GET', 'PUT'])
def next_id():
    if request.method == 'GET':
        identifier = db['next']
    elif request.method == 'PUT':
        if not (identifier := request.args.get('id', type=str)):
            abort(400, "Invalid file identifier")

        if identifier not in db['artworks']:
            abort(400, description="File identifier not found")

        db['next'] = identifier
        save_db(db)
    else:
        abort(405)

    return identifier


@app.route("/delete", methods=['DELETE'])
def delete():
    if not (identifier := request.args.get('id', type=str)):
        abort(400, description="Invalid file identifier")

    if identifier not in db['artworks']:
        abort(400, description="File identifier not found")

    # Remove file
    image_path(identifier).unlink()

    # Remove art from database
    db['artworks'].pop(identifier)

    # Reset next if it was the deleted artwork
    if db['next'] == identifier:
        db['next'] = ""

    save_db(db)

    return "Success", 200


@app.route("/upload", methods=['POST'])
def upload():
    # Parse response parameters
    if not (file := request.files.get('image')):
        abort(400, description="Image not provided")

    if not (title := request.args.get('title', type=str)):
        abort(400, description="Invalid title")

    # Check file extension
    if not has_valid_extension(file.filename):
        abort(415, description="Unsupported image file extension")

    artist = request.args.get('artist', default="none", type=str)
    pad = request.args.get('pad', default=True, type=bool)
    force = request.args.get('force', default=False, type=bool)

    # Convert image to B&W
    img = Image.open(file)  # type: ignore
    img = img.convert('1', dither=Image.NONE)

    # Create unique identifier by hashing title and artist
    identifier = md5(f"{title}{artist}".encode()) \
        .hexdigest()[:16]

    # Check that artwork doesn't already exist
    if not force and identifier in db['artworks']:
        abort(
            400, description=f"Cancelled upload of duplicate artwork: {title} by {artist}")

    # Store info to disk
    db['artworks'][identifier] = {
        'id': identifier,
        'title': title,
        'artist': artist,
        'pad': pad
    }
    save_db(db)

    # Save image
    img.save(image_path(identifier))

    return "Success", 200


@app.route("/image", methods=['GET'])
def download():
    identifier = request.args.get('id', type=str)
    width = request.args.get('w', type=int)
    height = request.args.get('h', type=int)

    # If identifier not provided, return random image
    if not identifier:
        ids = list(db['artworks'].keys())
        if len(ids) == 0:
            abort(500, "No artwork was found")

        # Select the saved next id
        next = db['next']
        identifier = ids[0] if next == "" else next

        if len(ids) == 1:
            identifier = ids[0]
            next = ids[0]
        else:
            # Queue random, different image
            while next == "" or identifier == next:
                next = random.choice(ids)

        # Update next image
        db['next'] = next
        save_db(db)

    # Retrieve the image
    artwork = db['artworks'][identifier]
    img = Image.open(image_path(identifier))

    # Processes the image to the desired dimensions
    if width and height:
        dimensions = (width, height)
        img = ImageOps.contain(img, dimensions)
        img = ImageOps.pad(
            img, dimensions, color=0xFFFFFF if artwork['pad'] else 0x0)

    # Send the bitmap image
    img_io = BytesIO()
    img.save(img_io, 'BMP')
    img_io.seek(0)
    return send_file(img_io, mimetype='image/bmp')
