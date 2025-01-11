#!/bin/bash

# Configuration for cross-compiling to Raspberry Pi
export GOOS=linux
export GOARCH=arm
export GOARM=7

output_dir="build"
target_name="server"

mkdir -p "$output_dir"

echo "Building for Raspberry Pi ($GOARCH, $GOOS)..."
go build -o $output_dir/$target_name

if [ $? -eq 0 ]; then
  echo "Build succeeded"
else 
  echo "Build failed"
  exit 1
fi

echo "Deploying to Raspberry Pi..."

dest="pi@192.168.1.5:/home/pi/inkarta"

# Copy binary and HTML templates
scp "$output_dir/$target_name" "$dest"
scp -r templates/ "$dest/templates"
