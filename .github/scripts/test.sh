#!/bin/bash

cargo clippy --all-features --all-targets -- -D warnings
cargo test
cargo test --all-features
