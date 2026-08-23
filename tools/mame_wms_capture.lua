-- Deterministic MAME capture helper.
-- Example:
--   mame stargate -autoboot_script tools/mame_wms_capture.lua \
--       -seconds_to_run 5 -wavwrite /tmp/mame.wav -samplerate 44100
-- Trigger the sound through normal Stargate controls while capture runs.

local frames = 0
local machine = manager.machine

emu.register_frame_done(function()
    frames = frames + 1
    if frames == 1 then
        print("WMS capture started; use the game's sound-triggering input")
    end
end)

print("sound CPU: " .. tostring(machine.devices[":soundcpu"]))
