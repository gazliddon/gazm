#!/usr/bin/env python3
"""Convert MAME sound-command capture lines to wms-sound replay events."""
import argparse
import re

MAIN_HZ = 4_000_000       # Stargate MC6809E: 12 MHz master / 3
SOUND_HZ = 894_886        # M6808 effective clock: 3.579545 MHz / 4
EVENT = re.compile(r"WMS_EVENT\s+main_cycles=([0-9a-fA-F]+)\s+value=([0-9a-fA-F]+)")

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="MAME trace containing WMS_EVENT lines")
    parser.add_argument("output", help="wms-sound replay log")
    args = parser.parse_args()
    count = 0
    with open(args.input, encoding="utf-8") as source, open(args.output, "w", encoding="utf-8") as dest:
        for line in source:
            match = EVENT.search(line)
            if not match:
                continue
            # MAME's debugger printf uses hexadecimal for %X.
            main_cycles = int(match.group(1), 16)
            value = int(match.group(2), 16) | 0xc0
            sound_cycles = (main_cycles * SOUND_HZ + MAIN_HZ // 2) // MAIN_HZ
            dest.write(f"{sound_cycles} {value:02x}\n")
            count += 1
    print(f"converted {count} events")

if __name__ == "__main__":
    main()
