#!/usr/bin/env bash

IMAGE_BASE_NAME=tinyexpenses
HASH=$(git rev-parse HEAD)
IMAGE_PATCH_NAME=$IMAGE_BASE_NAME:patch-"${HASH::5}"

echo "Building image" "$IMAGE_PATCH_NAME"

docker build -t $IMAGE_BASE_NAME:latest -t "$IMAGE_PATCH_NAME" .

echo "Build complete"
