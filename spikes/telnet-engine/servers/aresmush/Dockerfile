FROM ruby:3.3.13-slim-trixie@sha256:db3ad6dd68a886d51a187fb9bf486052f2976cdbe3b750169c25092aba1297fa

# Build tools for the native gems (eventmachine, bcrypt, sassc, thin, redcarpet).
# git is required at run time: rake configure shells out to `git config`.
RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential git redis-server ca-certificates \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash ares
USER ares
WORKDIR /home/ares

# renovate: datasource=github-tags depName=AresMUSH/aresmush
ARG ARESMUSH_VERSION=v2.16.0

RUN git clone --depth 1 --branch "${ARESMUSH_VERSION}" https://github.com/AresMUSH/aresmush.git aresmush

WORKDIR /home/ares/aresmush
ENV LANG=C.UTF-8 LC_ALL=C.UTF-8 BUNDLE_PATH=/home/ares/bundle

RUN bundle install --jobs 4

# Same steps as upstream bin/install: copy the starter game, then answer
# rake configure through its non-interactive "key=value~key=value" form.
# bind_address 0.0.0.0 makes the telnet port reachable from outside the container.
RUN cp -r install/game.distr game \
 && mkdir -p game/logs \
 && bundle exec rake configure "server_port=4201~host_name=localhost" \
 && sed -i "s/^\(\s*bind_address:\).*/\1 '0.0.0.0'/" game/config/server.yml

# init_db sets requirepass and runs CONFIG REWRITE, so Redis needs a writable config file.
COPY --chown=ares:ares redis.conf /home/ares/redis.conf
COPY --chown=ares:ares --chmod=755 start.sh /home/ares/start.sh

EXPOSE 4201
ENTRYPOINT ["/home/ares/start.sh"]
