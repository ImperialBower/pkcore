#!/bin/bash

# Script to strip ANSI color codes from files or stdin
# Usage:
#   ./strip_colors.sh <input_file> [output_file]
#   cat file.log | ./strip_colors.sh
#   ./strip_colors.sh file.log stripped_file.log

strip_ansi() {
    # Use sed to remove ANSI escape sequences
    sed 's/\x1b\[[0-9;]*[a-zA-Z]//g'
}

if [ $# -eq 0 ]; then
    # No arguments - read from stdin
    strip_ansi
elif [ $# -eq 1 ]; then
    # One argument - read from file, output to stdout
    strip_ansi < "$1"
elif [ $# -eq 2 ]; then
    # Two arguments - read from file, write to file
    strip_ansi < "$1" > "$2"
    echo "Stripped ANSI codes from $1 to $2"
else
    echo "Usage: $0 [input_file] [output_file]"
    echo "  No args: read from stdin, write to stdout"
    echo "  1 arg:   read from file, write to stdout"
    echo "  2 args:  read from file, write to file"
    exit 1
fi

