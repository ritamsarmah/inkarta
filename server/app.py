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
        DB_PATH.write_text("{\"images\":{}, \"next_id\":\"\"}")

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


@app.route("/upload", methods=['POST'])
def upload():
    if not (file := request.files.get('image')):
        abort(400, description="Image not provided")

    if not (title := request.args.get('title', type=str)):
        abort(400, description="Invalid title")

    if not (artist := request.args.get('artist', type=str)):
        abort(400, description="Invalid artist")

    if not has_valid_extension(file.filename):
        abort(400, description="Unsupported image file extension")

    # Convert image to B&W
    img = Image.open(file)  # type: ignore
    img = img.convert('1', dither=Image.NONE)

    # Create unique identifier by hashing title and artist
    id = md5(f"{title}{artist}".encode()) \
        .hexdigest()[:16]
    filename = f"{id}.bmp"
    filepath = IMAGES_DIR / filename

    # Check that artwork doesn't already exist
    db = get_db()
    if id in db['images']:
        abort(
            400, description="Art with title ({title}) and artist ({artist}) already exists")

    # Store info to disk
    db['images'][id] = {
        'title': title,
        'artist': artist,
        'file': filename
    }
    set_db(db)

    # Save image
    img.save(filepath)

    return "Artwork successfully uploaded"


@app.route("/delete", methods=['DELETE'])
def delete():
    if not (id := request.args.get('id', type=str)):
        abort(400, description="Invalid file identifier")

    db = get_db()
    if id not in db:
        abort(400, description="File identifier not found")

    # Remove file
    filepath = IMAGES_DIR / db['images'][id]['file']
    filepath.unlink()

    # Remove art from database
    db.pop(id)
    set_db(db)

    return "Artwork successfully deleted"


@app.route("/queue", methods=['POST'])
def queue_next_image():
    if not (id := request.args.get('id', type=str)):
        abort(400, "Invalid file identifier")

    db = get_db()
    if id not in db['images']:
        abort(400, description="File identifier not found")

    db['next_id'] = id
    set_db(db)

    art = db['images'][id]
    return f"Queued art: {art['title']} by {art['artist']}"


@app.route("/random", methods=['GET'])
def download():
    if not (width := request.args.get('width', type=int)):
        abort(400, "Invalid width")
    if not (height := request.args.get('height', type=int)):
        abort(400, "Invalid height")

    dimensions = (width, height)

    # Get random image path
    db = get_db()
    ids = list(db['images'].keys())
    if len(ids) == 0:
        abort(500, "No artwork was found")

    # Select the saved next id as the current id
    next_id = db['next_id']
    current_id = next_id

    if current_id == "" or len(ids) == 1:
        current_id = ids[0]
        next_id = ids[0]
    else:
        # Queue random, different image
        while current_id == next_id:
            next_id = random.choice(ids)

    # Update next image
    db['next_id'] = next_id
    set_db(db)

    # Processes the image to the desired dimensions
    art = db['images'][current_id]
    img = Image.open(IMAGES_DIR / art['file'])
    img = ImageOps.contain(img, dimensions)
    img = ImageOps.pad(img, dimensions, color=0xFFFFFF)

    # Send the bitmap image
    img_io = BytesIO()
    img.save(img_io, 'BMP')
    img_io.seek(0)
    return send_file(img_io, mimetype='image/bmp')
