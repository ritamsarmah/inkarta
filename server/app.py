from hashlib import md5
from io import BytesIO
from pathlib import Path
from PIL import Image, ImageOps

from flask import Flask, request, make_response, send_file
from flask import abort as fabort

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


def abort(status_code, message):
    response = make_response(message)
    response.status_code = status_code
    fabort(response)


''' Routes '''


@app.route("/all", methods=['GET'])
def fetch_all():
    ''' Returns entire database '''
    return db


@app.route("/next", methods=['GET', 'PUT'])
def next_id():
    ''' Retrieve or set the next artwork returned by /image '''

    if request.method == 'GET':
        identifier = db['next']
    elif request.method == 'PUT':
        identifier = request.args.get('id', default="", type=str)

        if identifier not in db['artworks']:
            abort(400, "File identifier not found")

        db['next'] = identifier
        save_db(db)

    return identifier  # type: ignore


@app.route("/delete", methods=['DELETE'])
def delete():
    ''' Delete artwork by id '''

    identifier = request.args.get('id', type=str)
    if not identifier:
        abort(400, "Invalid file identifier")

    if identifier not in db['artworks']:
        abort(400, "File identifier not found")

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
    ''' Upload new artwork '''

    # Parse response parameters
    file = request.files.get('file')
    if not file:
        abort(400, "Invalid image")

    title = request.args.get('title', type=str)
    if not title:
        abort(400, "Invalid title")

    # Check file extension
    if not has_valid_extension(file.filename):  # type: ignore
        abort(415, "Unsupported image file extension")

    artist = request.args.get('artist', default="Anonymous", type=str)
    dark = request.args.get('dark', default=False,
                            type=lambda q: q.lower() == 'true')
    overwrite = request.args.get(
        'overwrite', default=False, type=lambda q: q.lower() == 'true')

    # Create unique identifier by hashing title and artist
    identifier = md5(f"{title}{artist}".encode()) \
        .hexdigest()[:16]

    # Check that artwork doesn't already exist
    if not overwrite and identifier in db['artworks']:
        abort(
            400, f"Cancelled upload of duplicate artwork: {title} by {artist}")

    # Store info to disk
    db['artworks'][identifier] = {
        'id': identifier,
        'title': title,
        'artist': artist,
        'dark': dark
    }
    save_db(db)

    # Convert image to B&W and save
    img = Image.open(file)  # type: ignore
    img = img.convert('1', dither=Image.NONE)
    img.save(image_path(identifier))

    return "Success", 200


@app.route("/image", methods=['GET'])
def download():
    ''' Retrieve artwork, either randomly or by ID, with optional dimensions '''

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
            img, dimensions, color=0x0 if artwork['dark'] else 0xFFFFFF)

    # Send the bitmap image
    img_io = BytesIO()
    img.save(img_io, 'BMP')
    img_io.seek(0)
    return send_file(img_io, mimetype='image/bmp')


if __name__ == '__main__':
    app.run(host="0.0.0.0", port=5000, debug=False)
