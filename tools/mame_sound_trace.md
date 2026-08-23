# MAME sound CPU trace

Run from the Gazm directory:

```sh
mame stargate -video none -debug -debugger none \
  -debugscript tools/mame_sound_trace.cmd \
  -seconds_to_run 2
```

The MAME debugger trace is written to `/tmp/mame-sound-trace.log`. It contains
the sound CPU's instruction-level execution and is the reference trace for
comparison with the Gazm 6800 emulator.
