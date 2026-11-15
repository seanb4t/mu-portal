#!/bin/bash
# Starts Redis, builds a fresh database (rake init + initmigrations, as upstream
# bin/install does), then runs the game in the foreground.
set -euo pipefail

redis-server /home/ares/redis.conf
until redis-cli ping >/dev/null 2>&1 || redis-cli ping 2>&1 | grep -q NOAUTH; do sleep 0.2; done

cd /home/ares/aresmush
bundle exec rake init
bundle exec rake initmigrations
exec bundle exec rake startares
