#!/bin/sh
# Runs the Evennia Portal, which owns the telnet port, in the foreground; the Portal starts the Server.
# Before that the launcher creates account #1 as superuser from EVENNIA_SUPERUSER_* if it is missing.
set -eu
export EVENNIA_SUPERUSER_USERNAME="${EVENNIA_SUPERUSER_USERNAME:-admin}"
export EVENNIA_SUPERUSER_PASSWORD="${EVENNIA_SUPERUSER_PASSWORD:-mu-spike-admin}"
export EVENNIA_SUPERUSER_EMAIL="${EVENNIA_SUPERUSER_EMAIL:-}"
cd /evennia/game
exec evennia ipstart
