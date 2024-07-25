#!/bin/bash
set -e

export TERM=xterm-256color

# Statements waiting to be executed
statements=(
    "RUSTFLAGS=\"--cfg loom\" cargo test --no-default-features --features derive --test concurrent_test --release"
)

# loop echo and executing statements
for statement in "${statements[@]}"; do
    echo "$(tput setaf 3)$statement$(tput sgr0)"
    eval $statement
    echo
done
