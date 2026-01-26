#!/usr/bin/env bash

set -e

# Configuration for cross-compiling to Raspberry Pi
export GOOS=linux
export GOARCH=arm
export GOARM=7

project="inkarta"

echo "Building for $GOOS ($GOARCH)"
go build .

echo "Deploying to server"

host="pi@homehub"
dest="/home/pi/$project"

# Stop the server
ssh "$host" "sudo systemctl stop inkarta"

# Copy files
rsync "$project" "$host:$dest"
rsync "$project.service" "$host:$dest"

# Restart the server
ssh "$host" "sudo systemctl daemon-reload && sudo systemctl start $project"
