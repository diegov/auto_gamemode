#!/usr/bin/env bash

set -e
set -o pipefail

function print_usage {
    echo "Usage: $0" >&2
    echo "       $0 check" >&2
    echo "       $0 docker [DOCKER_ARGS...]" >&2
}

argcount="$#"

function check_max_args {
    max="$1"
    
    if [ "$argcount" -gt "$max" ]; then
        print_usage
        exit 1
    fi
}

if [ "$1" == "check" ]; then
    check_max_args 1
    cargo clean -p "$(cargo read-manifest | jq -r '.name')"
    cargo clippy -- -D warnings
    cargo fmt -- --check
    cargo test --verbose
elif [ "$1" == "audit" ]; then
    # We'll do audit separately, it takes too long to install so we're skipping it for github's action
    check_max_args 1
    cargo audit
elif [ "$1" == "docker" ]; then
    shift;
    DOCKER_BUILDKIT=1 docker build --file Dockerfile --output docker_target . "$@"
elif [ "$1" == "" ]; then
    check_max_args 1
    cargo build
else
    print_usage
    exit 1
fi
