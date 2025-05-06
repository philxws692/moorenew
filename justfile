#!/usr/bin/env just --justfile

default:
    @just --choose

release-all: release-macos release-linux
    mkdir -p builds/
    cp target/aarch64-apple-darwin/release/moorenew builds/moorenew_aarch64
    cp target/x86_64-unknown-linux-musl/release/moorenew builds/moorenew_linux

release-linux:
    cargo build --release --target x86_64-unknown-linux-musl --features ssh2/vendored-openssl

release-macos:
    cargo build --release --target aarch64-apple-darwin

lint:
    cargo clippy