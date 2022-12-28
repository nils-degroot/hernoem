# Hernoem

Simple tool to rename audio files by tags.

# Installation

To install, download the correct binary for your system at the releases
section.

# Usage

Usage via the command file. For possible options, use the `-h` flag.

## Supported types

Currently, the following file types are supported

- mp3
- mp4
- m4a
- flac

## Format

A format can be specified by the user, the following keys would result to the
following tags of the audio.

| Key  | Result to    | Specialties                       |
|------|--------------|-----------------------------------|
| `%a` | Artist       | Multiple would be joined with `,` |
| `%A` | Album artist | Multiple would be joined with `,` |
| `%t` | Title        |                                   |
| `%b` | Album        |                                   |
| `%y` | Release year |                                   |
| `%n` | Track number | Track 1 would result to 01        |
| `%g` | Genre        |                                   |
| `%c` | Composer     |                                   |
| `%d` | Disc         |                                   |

