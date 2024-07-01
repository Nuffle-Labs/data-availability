# Default recipe
default:
    just --list

GHCR_BASE := "ghcr.io/nuffle-labs/data-availability"

docker-sidecar:
    docker build -t {{GHCR_BASE}}/http-api:dev -f bin/http-api/Dockerfile .

docker-push-sidecar:
    docker push {{GHCR_BASE}}/http-api:dev

devnet:
    docker compose up -d --build near-localnet
    docker compose up -d near-localnet-set-key

changelog:
    git-cliff > CHANGELOG.md
