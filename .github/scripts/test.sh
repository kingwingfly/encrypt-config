#!/bin/bash
set -e

export TERM=xterm-256color

# Statements waiting to be executed
statements=(
    "cargo clippy --all-targets -- -D warnings"
    "cargo clippy --all-targets --features persist -- -D warnings"
    "cargo clippy --all-targets --features secret -- -D warnings"
    "cargo clippy --all-targets --features secret,mock -- -D warnings"

    "cargo test"
    "cargo test --features persist"
    "cargo test --features persist,default_config_dir"
    "cargo test --features secret,mock"
    "cargo test --features secret,mock,default_config_dir"

    "cargo run --example example --features full,mock"

    "cargo doc --no-deps --features full"
)

# loop echo and executing statements
for statement in "${statements[@]}"; do
    echo "$(tput setaf 3)$statement$(tput sgr0)"
    eval $statement
    echo
done
