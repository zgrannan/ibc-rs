#!/usr/bin/env sh

set -ex

cargo prusti
cargo prusti --features resource
