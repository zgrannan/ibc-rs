#!/usr/bin/env python3
import sys
lines = sys.stdin.readlines()

in_section = False
for line in lines:
    if "PROPSPEC_START" in line:
        if in_section:
            print("DOUBLE START", file=sys.stderr)
            exit(1)
        if len(sys.argv) > 1 and not sys.argv[1] in line:
            continue
        in_section = True
        continue
    elif "PROPSPEC_STOP" in line:
        if not in_section and len(sys.argv) <= 1:
            print("DOUBLE STOP", file=sys.stderr)
            exit(1)
        in_section = False
        continue
    if in_section and not line.isspace():
        print(line.rstrip())
