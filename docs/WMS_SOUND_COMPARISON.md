# Comparing the WMS Sound Board with MAME

MAME is the reference for the Stargate board. The installed `stargate` set is
complete and uses the same sound ROM as `wms-sound`, so comparisons can focus on
emulation rather than ROM differences.

## Capture a reference WAV

Start MAME with a fixed output rate and capture duration. This captures the
normal boot sequence, including the sound played while the initial screen is
cleared:

```sh
mame stargate -samplerate 44100 -seconds_to_run 5 \
  -wavwrite /tmp/mame-stargate.wav
```

No Lua or input injection is required for the boot sound. For a scripted
startup/diagnostic hook, the helper is also available:

```sh
mame stargate -autoboot_script tools/mame_wms_capture.lua \
  -samplerate 44100 -seconds_to_run 5 \
  -wavwrite /tmp/mame-stargate.wav
```

`-wavwrite` records the final mixer output. The Lua script currently provides a
stable capture hook and identifies the sound CPU; sound commands should still be
generated through the game input path so that the PIA handshake is genuine.

## Generate and compare Gazm output

From this repository:

```sh
python3 tools/compare_wms_sound.py /tmp/mame-stargate.wav \
  --rom ../stargate/roms/sound.bin --command 0x2b --cycles 200000
```

The script runs `wms-sound`, writes a companion `.gazm.wav`, converts both WAVs
to mono floating-point samples, searches for a small timing offset, and reports
the best sample lag, correlation, and RMS error. MAME generally writes signed
16-bit stereo while Gazm currently writes unsigned 8-bit mono; the comparison
normalizes those formats automatically.

The `0x2b` value is the wire command for Stargate's logical laser sound `0x14`.
Use another command and matching cycle window to compare other effects.

The comparison gives the sound CPU 1,000 warm-up cycles before sending the
command. This lets the ROM finish its reset/startup code; change it with
`--warmup-cycles` if testing a different reset sequence.

## Capture and replay sound-board events

To capture the complete boot command stream from MAME, run its debugger script
for the same duration as the reference WAV:

```sh
mame stargate -video none -sound none -debug \
  -debugscript tools/mame_sound_events.cmd -seconds_to_run 8
python3 tools/mame_events_to_replay.py \
  /tmp/mame-sound-events-trace.log /tmp/stargate-events.log
```

The converter changes the main-CPU timestamps into sound-CPU cycles and adds the
`$c0` bits applied by MAME's `snd_cmd_w` handler.

To capture a portable debugger checkpoint at the first sound IRQ:

```sh
mame stargate -debug \
  -debugscript tools/mame_sound_checkpoint.cmd \
  -seconds_to_run 8
```

This produces a register/cycle trace plus `$0000-$00ff` sound RAM and the
four-byte PIA register window. The checkpoint is intended for instruction-level
comparison; MAME's native save-state file is not portable to `wms-sound`.

The harness can record the values delivered to the sound PIA while driving it
interactively:

```text
record /tmp/wms-events.log
sound 0x2b
run 200000
quit
```

The resulting file contains one event per line, as `cycle pia_value`, for
example `1000 eb`. The value is the actual PIA byte (the Williams interface
forces bits 6 and 7 high), not the game's logical command. Replay it with:

```text
replay /tmp/wms-events.log 200000
```

Events are applied at 6800 instruction boundaries. This is intentionally a
small, portable format so a MAME debugger/Lua capture can produce it without
coupling the emulator to MAME.

This is an end-to-end test. If it disagrees, the next refinement is an internal
trace mode that compares PIA port-A/DAC writes before mixer and resampling
differences are involved.
