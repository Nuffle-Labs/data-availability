# Justfile

# Set the path to the JSON file
JSON_FILE := "http-config.template.json"
ENRICHED_JSON_FILE := "http-config.ci.json"

# Default recipe
default:
    just --list

# Enrich JSON file with environment variable values
enrich:
    #!/usr/bin/env bash
    set -euo pipefail

    # Read the JSON file
    JSON_CONTENT=$(cat "{{JSON_FILE}}")

    # Replace the placeholders with environment variable values
    JSON_CONTENT=$(echo "$JSON_CONTENT" | sed "s/HTTP_API_TEST_ACCOUNT_ID/$HTTP_API_TEST_ACCOUNT_ID/g")
    JSON_CONTENT=$(echo "$JSON_CONTENT" | sed "s/HTTP_API_TEST_SECRET_KEY/$HTTP_API_TEST_SECRET_KEY/g")
    JSON_CONTENT=$(echo "$JSON_CONTENT" | sed "s/HTTP_API_TEST_NAMESPACE/$HTTP_API_TEST_NAMESPACE/g")

    # Write the updated JSON content back to the file
    echo "$JSON_CONTENT" > "{{ENRICHED_JSON_FILE}}"

    echo "JSON file enriched successfully."

GHCR_BASE := "ghcr.io/near/rollup-data-availability"

docker-sidecar:
    docker build -t {{GHCR_BASE}}/http-api:dev -f bin/http-api/Dockerfile .
