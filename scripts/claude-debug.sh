#!/bin/bash
# Debug wrapper for claude CLI in VHS environment

LOG="/tmp/claude-debug.log"

echo "=== Claude Debug Wrapper ===" > "$LOG"
echo "Date: $(date)" >> "$LOG"
echo "PWD: $PWD" >> "$LOG"
echo "PATH: $PATH" >> "$LOG"
echo "HOME: $HOME" >> "$LOG"
echo "TERM: $TERM" >> "$LOG"
echo "LANG: $LANG" >> "$LOG"
echo "TTY: $(tty 2>&1)" >> "$LOG"
echo "isatty stdin: $([[ -t 0 ]] && echo yes || echo no)" >> "$LOG"
echo "isatty stdout: $([[ -t 1 ]] && echo yes || echo no)" >> "$LOG"
echo "isatty stderr: $([[ -t 2 ]] && echo yes || echo no)" >> "$LOG"
echo "Claude path: $(which claude 2>&1)" >> "$LOG"
echo "Claude version: $(claude --version 2>&1)" >> "$LOG"
echo "" >> "$LOG"

echo "Args passed: $@" >> "$LOG"
echo "Arg count: $#" >> "$LOG"
echo "CLAUDECODE: ${CLAUDECODE:-<unset>}" >> "$LOG"
echo "" >> "$LOG"

echo "=== Starting Claude ===" >> "$LOG"
exec claude "$@" 2>> "$LOG"
