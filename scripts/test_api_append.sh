#!/usr/bin/env bash


IP=${1:-"0.0.0.0"}
PORT=${2:-"8080"}
USER=${3:-"demo"}
SUBCATEGORY=${4:-"Coffee"}
DATE=${5:-$(date -I)}
VALUE=${6:-"-12.3"}
DESCRIPTION=${6:-""}

read -r -d '' PAYLOAD <<EOF
{
    "subcategory": "$SUBCATEGORY",
    "date": "$DATE",
    "value": "$VALUE",
    "description": "$DESCRIPTION"
}
EOF

echo "Sending:"
echo "$PAYLOAD"

curl -v http://"$IP":"$PORT"/api/v1/"$USER"/report/append \
     -X PUT \
     -H "Content-Type: application/json" \
     -H "X-API-Key:QoMovvY9dfn3T-75rytRxFp_7w9QtCrk-wqzZxLVys0" \
     -d "$PAYLOAD"

printf "\n"