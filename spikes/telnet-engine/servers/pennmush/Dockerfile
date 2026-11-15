# PennMUSH built from upstream source, telnet on port 4201.
FROM debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2

# renovate: datasource=github-tags depName=pennmush/pennmush versioning=regex:^(?<major>\d+)p(?<minor>\d+)$
ARG PENNMUSH_VERSION=188p0

# Toolchain plus the optional libraries configure detects: OpenSSL, zlib,
# libevent (ssl_slave) and libcurl (@http).
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates curl build-essential perl gettext \
      libssl-dev zlib1g-dev libevent-dev libcurl4-openssl-dev \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --home-dir /mush mush
USER mush
WORKDIR /mush

# configure builds a bundled PCRE2 of the exact version it names, from a tarball in the source root.
# Its built-in download URL (ftp.pcre.org) is gone, so the tarball comes from the PCRE2 GitHub releases.
# --disable-info_slave: the libevent info_slave does reverse DNS even with use_dns off (below).
# `make update` writes game/mush.cnf from mushcnf.dst; stdin is closed so it never waits for an answer.
RUN curl -fsSL "https://github.com/pennmush/pennmush/archive/refs/tags/${PENNMUSH_VERSION}.tar.gz" \
      | tar -xz --strip-components=1 \
 && pcre2=$(sed -n 's/^PCRE2_VERSION="\(pcre2-[0-9.]*\)"$/\1/p' configure) \
 && test -n "$pcre2" \
 && curl -fsSLo "$pcre2.tar.gz" "https://github.com/PCRE2Project/pcre2/releases/download/$pcre2/$pcre2.tar.gz" \
 && ./configure --disable-info_slave \
 && make update </dev/null \
 && make install

# Reverse DNS of the client address times out on the Docker network and holds back the
# login screen until it does, so hostname lookup is off. This is the only change to mush.cnf.
RUN sed -i 's/^use_dns yes$/use_dns no/' game/mush.cnf \
 && grep -qx 'use_dns no' game/mush.cnf

COPY --chown=mush:mush --chmod=0755 start.sh /mush/start.sh

# Must match `port` in game/mush.cnf (upstream default 4201).
EXPOSE 4201

# No database is shipped: netmush creates the minimal world (god "One", no password) at first start.
ENTRYPOINT ["/mush/start.sh"]
