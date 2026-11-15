#!/bin/sh
# Startmux without the background launch: netmux runs in the foreground as PID 1.
# With no database in data/, netmux creates a minimal one (god: wizard / potrzebie).
set -eu
cd /mux/game
. ./mux.config
rm -f restart.db restart.db.tmp "$PIDFILE"
LD_LIBRARY_PATH=$BIN
export LD_LIBRARY_PATH
exec "$BIN/netmux" -c "$GAMENAME.conf" -p "$PIDFILE" -e .
