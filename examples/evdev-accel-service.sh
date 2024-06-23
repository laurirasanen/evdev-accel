#!/bin/sh
#
# Script for running evdev-accel again on device loss
#

DEVICE="Your device name here"

while true; do
    evdev-accel -d "$DEVICE"
    ex="$?"
    if [ $ex = 0 ]; then
        echo "evdev-accel exited with code 0, quitting"
        break
    fi
    echo "evdev-accel exited with code $ex, retrying in 1 second"
    sleep 1
done


