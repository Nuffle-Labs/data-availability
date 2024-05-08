# Justfile

# Set the path to the JSON file
JSON_FILE := "http-config.template.json"
ENRICHED_JSON_FILE := "http-config.json"

# Default recipe
default:
    just --list

# Enrich JSON file with environment variable values
enrich:
   scripts/enrich.sh  {{JSON_FILE}} {{ENRICHED_JSON_FILE}}

GHCR_BASE := "ghcr.io/near/rollup-data-availability"

docker-sidecar:
    docker build -t {{GHCR_BASE}}/http-api:dev -f bin/http-api/Dockerfile .

docker-push-sidecar:
    docker push {{GHCR_BASE}}/http-api:dev

devnet:
    docker compose up -d --build near-localnet
    docker compose up -d near-localnet-set-key
