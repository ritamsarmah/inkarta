#!/bin/bash

set -e

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

host="pi@192.168.1.5"
dest="/home/pi/inkarta"

# Stop the server
ssh "$host" "sudo systemctl stop inkarta"

# Copy binary and HTML templates
scp "$output_dir/$target_name" "$host:$dest"
scp -r templates/ "$host:$dest"

# Restart the server
ssh "$host" "sudo systemctl start inkarta"
