# Mixrs

A practical multimedia controller for PulseAudio written in Rust.

## Installation
There is currently no convenient way to install Mixrs. To use this application, you have to compile it yourself and run it in the background.

## Requirements
- [PulseAudio](https://www.freedesktop.org/wiki/Software/PulseAudio/)
- [playerctl](https://wiki.archlinux.org/title/MPRIS#Playerctl)
- [libnotify](https://gitlab.gnome.org/GNOME/libnotify) (required unless started with `--silent`)

## Usage
Mixrs will create a unix socket at `/tmp/mixrs` and listen for instructions. Instructions are issued by sending a specific byte to the socket.

## Example
`echo -ne '\x2' | nc -N -U /tmp/mixrs` will send a byte containing `2` to the `/tmp/mixrs` socket and instruct Mixrs to mute / unmute the currently selected sink input.

### Instructions
|Byte|Instruction|Detail|
|---|---|---|
|0|SelectNext|Selects the next sink input|
|1|SelectPrevious|Selects the previous sink input|
|2|ToggleMuteCurrent|Toggles the current sink input's muted state|
|3|IncreaseCurrent|Increases the current sink input's volume by 5%|
|4|DecreaseCurrent|Decreases the current sink input's volume by 5%|
|5|GetCurrent|Displays the current sink input's name<br>*Has no effect when using `--silent`*|
|6|PlayPauseCurrent|Tells the current sink input to toggle its `playing` state.<br>*Behavior varies based on the current sink input's player*|
|7|PlayNext|Tells the current sink input to play the next item (e.g. the next song).<br>*Behavior varies based on the current sink input's player*|
|8|PlayPrevious|Tells the current sink input to play the previous item (e.g. the previous song).<br>*Behavior varies based on the current sink input's player*|
|9|GetCurrentOutput|Gets information about the currently selected sink input and sends it through the requesting unix socket|
