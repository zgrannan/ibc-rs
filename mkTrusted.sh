#!/usr/bin/env bash
set -euo pipefail

grep '> modules/src' < out | sed 's/.*--> //' | xargs -n1 python3 tools.py trust
