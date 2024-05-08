#!/usr/bin/env bash
set -euo pipefail

JSON_FILE=${1:-"http-config.template.json"}
ENRICHED_JSON_FILE=${2:-"http-config.json"}

# Read the JSON file
JSON_CONTENT=$(cat "$JSON_FILE")

# Replace the placeholders with environment variable values
JSON_CONTENT=$(echo "$JSON_CONTENT" | sed "s/HTTP_API_TEST_ACCOUNT_ID/$HTTP_API_TEST_ACCOUNT_ID/g")
JSON_CONTENT=$(echo "$JSON_CONTENT" | sed "s/HTTP_API_TEST_SECRET_KEY/$HTTP_API_TEST_SECRET_KEY/g")
JSON_CONTENT=$(echo "$JSON_CONTENT" | sed "s/HTTP_API_TEST_NAMESPACE/$HTTP_API_TEST_NAMESPACE/g")

# Write the updated JSON content back to the file
echo "$JSON_CONTENT" > "$ENRICHED_JSON_FILE"
