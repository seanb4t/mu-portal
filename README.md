# mu-portal

mu-portal is a client and server for MU* text games: MUSH, MUX, AresMUSH and Evennia games. One program, `mu`, carries every function as a verb.

## Status

The project is at its skeleton. `mu server` holds the game connections, and `mu client` is a terminal client that attaches to it. `mu world` reads the configured worlds from the configuration directory. The server holds the configured worlds, but only the fake worlds for tests can connect. The first-run guide in the documentation site shows how to start the server and the client.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The documentation site is in `docs/`.

## License

Apache-2.0. See [LICENSE](LICENSE).
