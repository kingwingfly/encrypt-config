# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

## [0.2.7] - 2024-06-21

- doc: for linux doc
- remove unused error enum

## [0.2.6] - 2024-06-21

- conditional compilation for `keyring`

## [0.2.6] - 2024-06-19

- improve: (perf) speed up when reusing the Encrypter

## [0.2.5] - 2024-06-19

- fix: (bug) race condition when reusing the Encrypter

## [0.2.4] - 2024-06-19

- fix: (bug) always use the same encrypter even if different keyring entries are given
- doc: (derive macros) add doc for `PersistSource` and `SecretSource`
