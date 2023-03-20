#!/usr/bin/env python3
import sys
lines = sys.stdin.readlines()

in_section = False
for line in lines:
    if "PROPSPEC_START" in line:
        in_section = True
        continue
    elif "PROPSPEC_STOP" in line:
        in_section = False
        continue
    if in_section and not line.isspace():
        print(line.rstrip())
