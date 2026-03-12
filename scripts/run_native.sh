#!/usr/bin/env bash

IP=${1:-"0.0.0.0"}
PORT=${2:-"8080"}
ACCOUNTS_DIR=${3:-"./accounts"}
LOGS_DIR="./logs"

if [[ -z "$SECRET_KEY" ]]; then
    echo "Env variable 'SECRET_KEY' is not set - using debug key"
    SECRET_KEY="5b4e240b952d47f2db3bsh12louw1mxs2lsp21pskw"
fi

if [ ! -d "$ACCOUNTS_DIR" ]; then
    echo "$ACCOUNTS_DIR does not exists."
    exit 1
fi

mkdir -p "$LOGS_DIR"

SECRET_KEY=$SECRET_KEY cargo run --release -- --bind "$IP:$PORT" --db "$ACCOUNTS_DIR"