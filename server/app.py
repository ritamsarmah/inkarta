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

app = Flask(__name__)
app.config['UPLOAD_FOLDER'] = IMAGES_DIR
app.config['MAX_CONTENT_LENGTH'] = 16 * 1000 * 1000  # Limit uploads to 16 MB


''' Database '''


def get_db() -> dict:
    if not DB_PATH.exists():
        DB_PATH.write_text("{\"images\":{}, \"next\":\"\"}")

    with DB_PATH.open("r") as f:
        return json.load(f)


def set_db(db):
    with DB_PATH.open("w+") as f:
        json.dump(db, f)


''' Utilities '''


def has_valid_extension(filename):
    return '.' in filename and \
           filename.rsplit('.', 1)[1].lower() in ALLOWED_EXTENSIONS


''' Routes '''


@app.route("/fetch", methods=['GET'])
def fetch():
    return get_db()


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

    # Convert image to B&W
    img = Image.open(file)  # type: ignore
    img = img.convert('1', dither=Image.NONE)

    # Create unique identifier by hashing title and artist
    identifier = md5(f"{title}{artist}".encode()) \
        .hexdigest()[:16]
    filename = f"{identifier}.bmp"
    filepath = IMAGES_DIR / filename

    # Check that artwork doesn't already exist
    db = get_db()
    if identifier in db['images']:
        abort(
            400, description=f"Cancelled upload of duplicate artwork: {title} by {artist}")

    # Store info to disk
    db['images'][identifier] = {
        'id': identifier,
        'title': title,
        'artist': artist,
        'pad': pad
    }
    set_db(db)

    # Save image
    img.save(filepath)

    return "Success", 200


@app.route("/delete", methods=['DELETE'])
def delete():
    if not (identifier := request.args.get('id', type=str)):
        abort(400, description="Invalid file identifier")

    db = get_db()
    if identifier not in db['images']:
        abort(400, description="File identifier not found")

    # Remove file
    filepath = IMAGES_DIR / f"{identifier}.bmp"
    filepath.unlink()

    # Remove art from database
    db['images'].pop(identifier)

    # Reset next if it was the deleted artwork
    if db['next'] == identifier:
        db['next'] = ""

    set_db(db)

    return "Success", 200


@app.route("/next", methods=['GET', 'PUT'])
def next_id():
    db = get_db()

    if request.method == 'GET':
        identifier = db['next']
    elif request.method == 'PUT':
        if not (identifier := request.args.get('id', type=str)):
            abort(400, "Invalid file identifier")

        if identifier not in db['images']:
            abort(400, description="File identifier not found")

        db['next'] = identifier
        set_db(db)
    else:
        abort(405)

    return identifier


@app.route("/random", methods=['GET'])
def download():
    if not (width := request.args.get('w', type=int)):
        abort(400, "Invalid width")
    if not (height := request.args.get('h', type=int)):
        abort(400, "Invalid height")

    # Retrieve artwork ids
    db = get_db()
    ids = list(db['images'].keys())
    if len(ids) == 0:
        abort(500, "No artwork was found")

    # Select the saved next id as the current id
    next = db['next']
    identifier = ids[0] if next == "" else next

    if len(ids) == 1:
        identifier = ids[0]
        next = ids[0]
    else:
        # Queue random, different image
        while identifier == next:
            next = random.choice(ids)

    # Update next image
    db['next'] = next
    set_db(db)

    # Processes the image to the desired dimensions
    artwork = db['images'][identifier]
    dimensions = (width, height)
    img = Image.open(IMAGES_DIR / f"{identifier}.bmp")
    img = ImageOps.contain(img, dimensions)

    pad = 0xFFFFFF if artwork['pad'] else 0x0
    img = ImageOps.pad(img, dimensions, color=pad)

    # Send the bitmap image
    img_io = BytesIO()
    img.save(img_io, 'BMP')
    img_io.seek(0)
    return send_file(img_io, mimetype='image/bmp')
