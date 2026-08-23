#!/usr/bin/env python3
"""Compare a WMS sound-board WAV against a MAME reference WAV."""

from __future__ import annotations

import argparse
import math
import subprocess
import sys
import wave
from pathlib import Path


def read_wav(path: Path) -> tuple[int, list[float]]:
    with wave.open(str(path), "rb") as wav:
        channels, width, rate = wav.getnchannels(), wav.getsampwidth(), wav.getframerate()
        frames = wav.readframes(wav.getnframes())
    if width == 1:
        scale, offset = 128.0, 128.0
    elif width == 2:
        scale, offset = 32768.0, 0.0
    else:
        raise ValueError(f"{path}: unsupported sample width {width * 8} bit")
    size = channels * width
    samples = []
    for start in range(0, len(frames), size):
        total = 0.0
        for channel in range(channels):
            raw = frames[start + channel * width : start + (channel + 1) * width]
            value = int.from_bytes(raw, "little", signed=width == 2)
            total += (value - offset) / scale
        samples.append(total / channels)
    return rate, samples


def score(reference: list[float], candidate: list[float], max_lag: int) -> tuple[int, float, float]:
    best = (-1.0, 0, float("inf"))
    for lag in range(-max_lag, max_lag + 1):
        ref_start, candidate_start = max(0, lag), max(0, -lag)
        count = min(len(reference) - ref_start, len(candidate) - candidate_start)
        if count < 2:
            continue
        ref = reference[ref_start : ref_start + count]
        candidate_slice = candidate[candidate_start : candidate_start + count]
        ref_mean, candidate_mean = sum(ref) / count, sum(candidate_slice) / count
        ref_energy = sum((v - ref_mean) ** 2 for v in ref)
        candidate_energy = sum((v - candidate_mean) ** 2 for v in candidate_slice)
        correlation = 0.0 if not ref_energy or not candidate_energy else sum(
            (a - ref_mean) * (b - candidate_mean) for a, b in zip(ref, candidate_slice)
        ) / math.sqrt(ref_energy * candidate_energy)
        rms = math.sqrt(sum((a - b) ** 2 for a, b in zip(ref, candidate_slice)) / count)
        if correlation > best[0]:
            best = (correlation, lag, rms)
    correlation, lag, rms = best
    return lag, correlation, rms


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mame_wav", type=Path, help="reference WAV captured by MAME")
    parser.add_argument("--gazm-wav", type=Path, help="Gazm WAV; generate it when omitted")
    parser.add_argument("--rom", type=Path, help="sound ROM for wms-sound")
    parser.add_argument("--command", type=lambda value: int(value, 0), default=0x2B)
    parser.add_argument("--warmup-cycles", type=int, default=1_000)
    parser.add_argument("--cycles", type=int, default=200_000)
    parser.add_argument("--max-lag", type=int, default=2_000)
    args = parser.parse_args()
    gazm_wav = args.gazm_wav
    if gazm_wav is None:
        if args.rom is None:
            parser.error("--rom is required when --gazm-wav is omitted")
        gazm_wav = args.mame_wav.with_name(args.mame_wav.stem + ".gazm.wav")
        workspace = Path(__file__).resolve().parents[1]
        manifest = workspace.parent / "crates" / "Cargo.toml"
        commands = f"run {args.warmup_cycles}\nsound 0x{args.command:02x}\nwav {gazm_wav} {args.cycles}\nquit\n"
        result = subprocess.run(
            ["cargo", "run", "--quiet", "--manifest-path", str(manifest), "-p", "wms-sound", "--", str(args.rom)],
            input=commands, text=True, capture_output=True,
        )
        if result.returncode:
            sys.stderr.write(result.stdout + result.stderr)
            return result.returncode
        print(result.stdout, end="")
    ref_rate, reference = read_wav(args.mame_wav)
    gazm_rate, candidate = read_wav(gazm_wav)
    if ref_rate != gazm_rate:
        print(f"warning: sample rates differ (MAME {ref_rate}, Gazm {gazm_rate})")
    lag, correlation, rms = score(reference, candidate, args.max_lag)
    print(f"MAME: {args.mame_wav} ({len(reference)} samples @ {ref_rate} Hz)")
    print(f"Gazm: {gazm_wav} ({len(candidate)} samples @ {gazm_rate} Hz)")
    print(f"best_lag_samples={lag} correlation={correlation:.6f} rms_error={rms:.6f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
