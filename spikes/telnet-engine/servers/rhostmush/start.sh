#!/bin/sh
# Startmush without the background launch and the first-start questions.
# With no database, netrhost -s creates a new one (god: #1 / Nyctasia).
# The shell stays as PID 1 to pass SIGTERM and SIGINT on to netrhost; the
# netrhost.debugmon parent does not forward them.
set -eu
cd /rhost/Server/game
. ./mush.config

if [ -r "$CRASH_DB" ]; then
    if [ "$(tail -1 "$CRASH_DB")" = "***END OF DUMP***" ]; then
        mv -f "$CRASH_DB" "$NEW_DB"
    else
        rm -f "$CRASH_DB"
    fi
fi
if [ -r "$INPUT_DB" ]; then
    mv -f "$INPUT_DB" "$SAVE_DB"
fi
if [ -r "$NEW_DB" ]; then
    mv -f "$NEW_DB" "$INPUT_DB"
elif [ -r "$SAVE_DB" ]; then
    cp -f "$SAVE_DB" "$INPUT_DB"
fi
rm -f "$GAMENAME.pid"

if [ -f "$INPUT_DB" ]; then
    ./netrhost.debugmon "$DEBUG_ID" ./netrhost "$GAMENAME.conf" &
else
    ./netrhost.debugmon "$DEBUG_ID" ./netrhost -s "$GAMENAME.conf" &
fi
monitor=$!

stop() {
    if [ -r "$GAMENAME.pid" ]; then
        kill -TERM "$(cat "$GAMENAME.pid")" 2>/dev/null || true
    else
        kill -TERM "$monitor" 2>/dev/null || true
    fi
}
trap stop TERM INT

status=0
while kill -0 "$monitor" 2>/dev/null; do
    wait "$monitor" || status=$?
done
exit "$status"
