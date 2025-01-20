#!/usr/bin/env bash

set -e

# Configuration for cross-compiling to Raspberry Pi
export GOOS=linux
export GOARCH=arm
export GOARM=7

build_dir="build"
name="inkarta"

mkdir -p "$build_dir"

echo "Building for $GOOS ($GOARCH)"
go build -o "$build_dir/$name" 'cmd/inkarta/main.go'

echo "Deploying to server"

host="pi@192.168.1.5"
dest="/home/pi/inkarta"

# Stop the server
ssh "$host" "sudo systemctl stop inkarta"

# Copy files
scp "$build_dir/$name" "$host:$dest"
scp -r templates/ "$host:$dest"
scp 'init/inkarta.service' "$host:$dest"

# Restart the server
ssh "$host" "sudo systemctl daemon-reload && sudo systemctl start inkarta"
