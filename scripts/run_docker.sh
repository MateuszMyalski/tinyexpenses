#!/usr/bin/env bash

ACCOUNTS_DIR=${1:-"./accounts"}
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

docker run \
        -v "$ACCOUNTS_DIR":/app/accounts \
        -v "$LOGS_DIR":/app/logs \
        -e SECRET_KEY="$SECRET_KEY" \
        -p 8080:8080 \
        -u "$(id -u):$(id -g)" \
        tinyexpenses:latest
