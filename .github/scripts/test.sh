#!/bin/bash
set -e

export TERM=xterm-256color

# Statements waiting to be executed
statements=(
    "cargo clippy --no-default-features -- -D warnings"
    "cargo clippy --no-default-features --features persist -- -D warnings"
    "cargo clippy --no-default-features --features secret,mock -- -D warnings"

    "cargo test --no-default-features"
    "cargo test --no-default-features --features persist"
    "cargo test --no-default-features --features persist,default_config_dir"
    "cargo test --no-default-features --features secret,mock"
    "cargo test --no-default-features --features secret,mock,default_config_dir"
    "cargo test --no-default-features --features full,mock"
    "cargo test --no-default-features --features full,mock,default_config_dir"

    "RUSTFLAGS=\"--cfg loom\" cargo test --no-default-features --features full,mock --release"

    "cargo run --example example --no-default-features --features full,mock"
    "cargo run --example example --no-default-features --features full,mock,default_config_dir"

    "cargo doc --no-deps --no-default-features --features full,mock"
)

# loop echo and executing statements
for statement in "${statements[@]}"; do
    echo "$(tput setaf 3)$statement$(tput sgr0)"
    eval $statement
    echo
done
