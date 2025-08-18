# Get image
GET http://{{HOST}}:{{PORT}}/image/1
HTTP 200

# Set next image to ID 1
PUT http://{{HOST}}:{{PORT}}/image/next/1
HTTP 200

# Fetch next image
GET http://{{HOST}}:{{PORT}}/image/next
HTTP 200
