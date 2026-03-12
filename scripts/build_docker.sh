#!/usr/bin/env bash

IMAGE_NAME=tinyexpenses
HASH=$(git rev-parse HEAD)

echo "Building image" $IMAGE_NAME:patch-"${HASH::5}"

docker build -t $IMAGE_NAME:latest -t $IMAGE_NAME:patch-"$(git rev-parse HEAD:)" .

echo "Build complete"
