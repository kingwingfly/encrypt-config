#!/bin/bash
set -e

# clippy
cargo clippy --all-features --all-targets -- -D warnings
cargo clippy --features full --all-targets -- -D warnings

# derive tests
cargo test -p encrypt_config_derive --features encrypt_config/derive
cargo test -p encrypt_config_derive --features persist,encrypt_config/persist,encrypt_config/derive
cargo test -p encrypt_config_derive --features secret,encrypt_config/secret,encrypt_config/derive
cargo test -p encrypt_config_derive --features persist,default_config_dir,encrypt_config/persist,encrypt_config/default_config_dir,encrypt_config/derive
cargo test -p encrypt_config_derive --features secret,default_config_dir,encrypt_config/secret,encrypt_config/default_config_dir,encrypt_config/derive

# main crate tests
cargo test -p encrypt_config
cargo test -p encrypt_config --features persist
cargo test -p encrypt_config --features secret,mock
cargo test -p encrypt_config --features persist,default_config_dir
cargo test -p encrypt_config --features secret,default_config_dir,mock

# integration tests
cargo test -p tests --features derive
cargo test -p tests --features derive,persist
cargo test -p tests --features derive,persist,default_config_dir
cargo test -p tests --features derive,secret
cargo test -p tests --features derive,secret,default_config_dir

# examples test
cargo run --example derive_normal_example --features derive,normal
cargo run --example derive_persist_example --features derive,persist
cargo run --example derive_secret_example --features derive,secret
cargo run --example derive_persist_example --features derive,persist,default_config_dir
cargo run --example derive_secret_example --features derive,secret,default_config_dir
cargo run --example impl_normal_example --features normal
cargo run --example impl_persist_example --features persist
cargo run --example impl_secret_example --features secret,mock
cargo run --example impl_persist_example --features persist,default_config_dir
cargo run --example impl_secret_example --features secret,mock,default_config_dir
