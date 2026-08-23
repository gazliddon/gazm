#!/usr/bin/env python3
"""Compare MAME and Gazm 6800 instruction traces by PC/opcode."""

import argparse
import re
from pathlib import Path

LINE = re.compile(r"^([0-9A-Fa-f]{4}):(?: opcode=([0-9A-Fa-f]{2})|\s+\S+)")


def parse(path: Path):
    result = []
    for line in path.read_text().splitlines():
        match = LINE.match(line)
        if match:
            pc = int(match.group(1), 16)
            opcode = int(match.group(2), 16) if match.group(2) else None
            result.append((pc, opcode, line))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mame", type=Path)
    parser.add_argument("gazm", type=Path)
    parser.add_argument("--start", default="FC8C", help="PC at which to align traces")
    parser.add_argument("--count", type=int, default=500)
    args = parser.parse_args()
    mame, gazm = parse(args.mame), parse(args.gazm)
    start = int(args.start, 16)
    mame = mame[next((i for i, item in enumerate(mame) if item[0] == start), len(mame)) :]
    gazm = gazm[next((i for i, item in enumerate(gazm) if item[0] == start), len(gazm)) :]
    for index, (left, right) in enumerate(zip(mame[: args.count], gazm[: args.count])):
        if left[0] != right[0] or (left[1] is not None and right[1] is not None and left[1] != right[1]):
            print(f"first divergence at instruction {index}")
            print(f"MAME: {left[2]}")
            print(f"Gazm: {right[2]}")
            return 1
    compared = min(args.count, len(mame), len(gazm))
    print(f"matched {compared} instructions after ${start:04X}")
    if len(mame) != len(gazm):
        print(f"trace lengths: MAME={len(mame)} Gazm={len(gazm)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
