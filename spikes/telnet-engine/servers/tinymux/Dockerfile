# TinyMUX built from upstream source, telnet on port 4201.
FROM debian:trixie-slim@sha256:8aa5a7bf5a1955bddbe7d74fcdd0b91ab30e74362a8b73af672b0481b35b44a0 AS build

# renovate: datasource=github-tags depName=brazilofmux/tinymux
ARG TINYMUX_VERSION=Build-2.14.0.11

# Toolchain plus the libraries configure requires: OpenSSL and PCRE2.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates git build-essential pkg-config libssl-dev libpcre2-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /src
RUN git clone --depth 1 --branch "${TINYMUX_VERSION}" https://github.com/brazilofmux/tinymux.git .

# The repository ships the generated autotools files, but a git checkout gives them
# arbitrary mtimes and make would try to regenerate them. Giving them all one
# timestamp later than the checkout keeps the shipped files in use.
# Default configure options. `make install` puts netmux in game/bin and symlinks
# libmux.so and engine.so there from the build tree.
WORKDIR /src/mux
RUN find . \( -name aclocal.m4 -o -name configure -o -name Makefile.in -o -name autoconf.h.in \) -exec touch -d "@$(($(date +%s) + 1))" {} + \
 && ./configure \
 && make -j"$(nproc)" \
 && make install

# Copy the game directory with the symlinks resolved, so it runs without the build tree.
# The port edit fails the build if netmux.conf has no `port 2860` line.
RUN cp -rL game /game \
 && grep -q '^port 2860$' /game/netmux.conf \
 && sed -i 's/^port 2860$/port 4201/' /game/netmux.conf

FROM debian:trixie-slim@sha256:8aa5a7bf5a1955bddbe7d74fcdd0b91ab30e74362a8b73af672b0481b35b44a0

RUN apt-get update \
 && apt-get install -y --no-install-recommends libssl3t64 libpcre2-8-0 \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --home-dir /mux mux
COPY --from=build --chown=mux:mux /game /mux/game
COPY --chmod=755 start.sh /usr/local/bin/start-tinymux

USER mux
WORKDIR /mux/game
EXPOSE 4201
ENTRYPOINT ["/usr/local/bin/start-tinymux"]
