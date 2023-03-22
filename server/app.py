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
        DB_PATH.write_text("{\"images\":{}, \"last_id\":\"\"}")

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
        abort(400, description="Title not specified")

    if not (artist := request.args.get('artist', type=str)):
        abort(400, description="Artist not specified")

    if not has_valid_extension(file.filename):
        abort(400, description="Image file extension not supported")

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

    return "Image successfully uploaded"


@app.route("/delete", methods=['DELETE'])
def delete():
    if not (id := request.args.get('id')):
        abort(400, description="File identifier not specified")

    db = get_db()
    if id not in db:
        abort(400, description="File identifier not found")

    filepath = IMAGES_DIR / db[id]['file']
    if not filepath.exists():
        abort(
            400, description=f"Image with doesn't exist for file")

    # Remove image file
    filepath.unlink()

    # Remove art from database
    db.pop(id)
    set_db(db)

    return "Image successfully deleted"


@app.route("/random", methods=['GET'])
def download():
    width = request.args.get('width', type=int)
    height = request.args.get('height', type=int)

    if width is None:
        abort(400, "Valid width not specified")
    elif height is None:
        abort(400, "Valid height not specified")

    dimensions = (width, height)

    # Get random image path
    db = get_db()
    ids = list(db['images'].keys())
    if len(ids) == 0:
        abort(500, "No artwork was found")

    # Select random, different image if needed
    selected_id = ids[0]
    if len(ids) > 1:
        last_id = db['last_id']
        while selected_id == last_id:
            selected_id = random.choice(ids)

    # Update last seen image
    db['last_id'] = selected_id
    set_db(db)

    # Processes the image to the desired dimensions
    artwork = db['images'][selected_id]
    img = Image.open(IMAGES_DIR / artwork['file'])
    img = ImageOps.contain(img, dimensions)
    img = ImageOps.pad(img, dimensions, color=0xFFFFFF)

    # Send the bitmap image
    img_io = BytesIO()
    img.save(img_io, 'BMP')
    img_io.seek(0)
    return send_file(img_io, mimetype='image/bmp')
