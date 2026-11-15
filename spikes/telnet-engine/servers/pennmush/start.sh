#!/bin/sh
# Runs netmush in the foreground. --no-session stops it from forking into the background.
set -eu
cd /mush/game
export GAMEDIR=/mush/game
export DATEMSK=/mush/game/getdate.template
exec ./netmush --no-session --pid-file=netmush.pid /mush/game/mush.cnf
