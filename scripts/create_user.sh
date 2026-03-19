#!/usr/bin/env bash

set -e

usage() {
    echo "Usage: $0 <DATABASE PATH> <USERNAME>"
}

create() {
    local file_content=""
    local database=""
    local username=$2
    local default_passowrd='$scrypt$ln=17,r=8,p=1$wQBNCmLImSiNWI9Zw5ep3w$Tc+JpYXuwfIGmAdtDtkB+awtvepDGHGBg6lobVbePo0'

    database=$(realpath "$1")

    if [ ! -d "$database" ]; then
        echo "Database '$database' does not exists."
        echo "Fix by 'mkdir $database'."
        exit 1 
    fi

    if [ -e "$database/$username/config.toml" ]; then
        echo "User already exists!"
        exit 2
    fi

    mkdir -p "$database/$username"

    file_content=$(cat << EOF
[user]
full_name = "$username"
username = "$username"
password_hash = "$default_passowrd"
active = true

[tinyexpenses]
currency = ""
api_token = ""
dark_color_scheme = true
EOF
)

    echo "$file_content" > "$database/$username/config.toml"
    echo "User '$username' created!"
    echo "Login using '$username' with '1234' password. After login CHANGE your password!"
}

if [ "$#" -ne 2 ]; then
    usage
    exit 1
fi

create "$1" "$2"
exit 0