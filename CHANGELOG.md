# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

## [0.3.3] - 2024-06-23

- fix doc error

## [0.3.2] - 2024-06-23

- improve code quality

## [0.3.1] - 2024-06-23

- fix doc and spell mistakes

## [0.3.0] - 2024-06-23

- refactor: Now, multi-config-sources can be saved and loaded through `Config` in one go. But `add_xx_source`s are removed. By the way, one can defined their own sources by implementing `Source` trait while `NormalSource` `PersistSource` `SecretSource` are still provided.

## [0.2.9] - 2024-06-21

- fix a doc mistake

## [0.2.8] - 2024-06-21

- doc: for linux doc
- remove unused error enum

## [0.2.7] - 2024-06-21

- conditional compilation for `keyring`

## [0.2.6] - 2024-06-19

- improve: (perf) speed up when reusing the Encrypter

## [0.2.5] - 2024-06-19

- fix: (bug) race condition when reusing the Encrypter

## [0.2.4] - 2024-06-19

- fix: (bug) always use the same encrypter even if different keyring entries are given
- doc: (derive macros) add doc for `PersistSource` and `SecretSource`
