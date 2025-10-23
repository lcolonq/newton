#!/bin/sh
set -xu
pactl load-module module-null-sink sink_name=lcolonq-hls
./target/debug/newton_renderer overlay &
sleep 3
pw-link lcolonq-hls:monitor_FL alsa_capture.newton_renderer:input_FL
pw-link lcolonq-hls:monitor_FR alsa_capture.newton_renderer:input_FR
wait
