# RhostMUSH built from upstream source, telnet on port 4201.
FROM debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2

# The trunk repository has no tags, so the source is a commit SHA on its master branch.
# The custom manager for RhostMUSH in renovate.json reads this SHA and follows master.
ARG RHOSTMUSH_VERSION=9ae9a39e3badce61c7f92a35d4dbbd9db0087255

# Toolchain plus OpenSSL, which the default build links. asksource.sh needs bash.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates curl bash build-essential libssl-dev \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --home-dir /rhost rhost
USER rhost
WORKDIR /rhost

# src/udb_ocache.c calls rhost_time() but does not include externs.h, which defines
# it, so the link fails; the sed adds the include. The grep fails the build if the
# anchor line is gone.
# `make default` is the non-interactive build: it runs `asksource.sh default`, which
# applies the upstream default options (bin/asksource.default and
# bin/asksource.save_template) without the menu, then compiles. `make links` puts the
# binaries in game/. The txt indexes are what Startmush rebuilds on each start.
RUN curl -fsSL "https://github.com/RhostMUSH/trunk/archive/${RHOSTMUSH_VERSION}.tar.gz" \
      | tar -xz --strip-components=1 \
 && cd Server \
 && grep -q '^#include[[:space:]]*"debug.h"' src/udb_ocache.c \
 && sed -i '/^#include[[:space:]]*"debug.h"/a #include "externs.h"' src/udb_ocache.c \
 && make default </dev/null \
 && make links \
 && cd game \
 && for t in news help wizhelp error doorconf; do ./mkindx "txt/$t.txt" "txt/$t.indx"; done

COPY --chmod=755 start.sh /usr/local/bin/start-rhostmush

WORKDIR /rhost/Server/game
EXPOSE 4201
ENTRYPOINT ["/usr/local/bin/start-rhostmush"]
