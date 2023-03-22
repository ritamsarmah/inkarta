from hashlib import md5
from io import BytesIO
from pathlib import Path
import random
from PIL import Image, ImageOps

from flask import Flask, abort, request, send_file

import json

IMAGES_DIR = Path('images/')
ALLOWED_EXTENSIONS = {'bmp', 'png', 'jpg', 'jpeg', 'tiff', 'tif'}
DB_PATH = Path('db.json')

app = Flask(__name__)
app.config['UPLOAD_FOLDER'] = IMAGES_DIR
app.config['MAX_CONTENT_LENGTH'] = 16 * 1000 * 1000  # Limit uploads to 16 MB


''' Database Management '''


def get_db():
    if not DB_PATH.exists():
        DB_PATH.write_text("{}")

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

    if not (title := request.args.get('title')):
        abort(400, description="Title not specified")

    if not (artist := request.args.get('artist')):
        abort(400, description="Artist not specified")

    if not has_valid_extension(file.filename):
        abort(400, description="Image file extension not supported")

    # Convert image to B&W
    img = Image.open(file)  # type: ignore
    img = img.convert('1', dither=Image.NONE)

    # Create unique filename by hashing title and artist
    key = md5(f"{title}{artist}".encode()) \
        .hexdigest()[:16]
    filename = f"{key}.bmp"
    filepath = IMAGES_DIR / filename

    # Check that artwork doesn't already exist
    db = get_db()
    if key in db:
        return f"Art with title ({title}) and artist ({artist}) already exists", 400

    # Store info to disk
    db[key] = {
        'title': title,
        'artist': artist,
        'path': str(filepath)
    }
    set_db(db)

    # Save image
    img.save(filepath)

    return "Image successfully uploaded"


@app.route("/delete", methods=['DELETE'])
def delete():
    if not (filename := request.args.get('filename')):
        abort(400, description="Filename not specified")

    filepath = Path(IMAGES_DIR / filename)
    if not filepath.exists():
        abort(
            400, description=f"Image with filename ({filename}) doesn't exist")

    return "Image successfully deleted"


@app.route("/random", methods=['GET'])
def download():
    width = request.args.get('width', type=int)
    height = request.args.get('height', type=int)

    if width is None:
        abort(400, "Width not specified")
    elif height is None:
        abort(400, "Height not specified")

    dimensions = (width, height)

    # Get random image path
    db = get_db()
    if len(db.keys()) == 0:
        abort(500, "No artwork was found")

    selection = random.choice(list(db.values()))

    # Processes the image to the desired dimensions
    img = Image.open(selection['path'])
    img = ImageOps.contain(img, dimensions)
    img = ImageOps.pad(img, dimensions, color=0xFFFFFF)

    img_io = BytesIO()
    img.save(img_io, 'BMP')
    img_io.seek(0)
    return send_file(img_io, mimetype='image/bmp')
