# Evennia installed from PyPI, telnet on port 4201.
FROM python:3.15-slim-trixie@sha256:63bb298d2b7b88f176f51b5189f54ce6535c997682ea70885d2132f5d473a8a5

# renovate: datasource=pypi depName=evennia
ARG EVENNIA_VERSION=6.1.0

# tini is PID 1: the evennia launcher neither reaps its children nor passes on SIGTERM.
RUN apt-get update \
 && apt-get install -y --no-install-recommends tini \
 && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir --root-user-action=ignore "evennia==${EVENNIA_VERSION}"

RUN useradd --create-home --home-dir /evennia evennia
USER evennia
WORKDIR /evennia

# The game directory with the telnet port moved from the default 4000 to 4201.
# Every other setting, including the telnet protocol options, is the Evennia default.
RUN evennia --init game \
 && printf '\nTELNET_PORTS = [4201]\n' >> game/server/conf/settings.py

WORKDIR /evennia/game

# The SQLite schema is created in the image; start.sh creates the superuser at first start.
RUN evennia migrate

COPY --chmod=0755 start.sh /evennia/start.sh

# Must match TELNET_PORTS in game/server/conf/settings.py.
EXPOSE 4201

# -g sends signals to the whole process group, so the Portal under the launcher gets SIGTERM too.
ENTRYPOINT ["/usr/bin/tini", "-g", "--", "/evennia/start.sh"]
