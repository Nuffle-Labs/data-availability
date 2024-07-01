# Default recipe
default:
    just --list

GHCR_BASE := "ghcr.io/nuffle-labs/data-availability"

docker-sidecar:
    docker build -t {{GHCR_BASE}}/sidecar:dev -f bin/sidecar/Dockerfile .
    # For backwards compat
    docker tag {{GHCR_BASE}}/sidecar:dev {{GHCR_BASE}}/http-api:dev

docker-sidecar-push:
    docker push {{GHCR_BASE}}/sidecar:dev
    # Backwards compat
    docker push {{GHCR_BASE}}/http-api:dev

devnet:
    docker compose up -d --build near-localnet
    docker compose up -d near-localnet-set-key

changelog:
    git-cliff > CHANGELOG.md
