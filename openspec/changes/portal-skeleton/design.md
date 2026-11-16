# Design

## Context

The four crates are empty. The decision pages fix the shape of the product: [one binary](/decisions/one-binary/), [one server for many clients](/decisions/server-centered/), the [ConnectRPC API](/decisions/connectrpc-api/), the [structured journal](/decisions/structured-journal/), [concurrent worlds](/decisions/concurrent-worlds/), [world-type profiles](/decisions/world-type-profiles/) and [Lua scripting](/decisions/lua-scripting/). The specs of this change name the behavior. This design names the seams, the crates and the libraries.

The change builds on two spikes:

- The ConnectRPC spike: the page `docs/src/content/docs/decisions/connectrpc-api.md` and pull request 42. Pull request 42 has no review thread, and its one review approves it with no finding.
- The input editor spike: the page `docs/src/content/docs/decisions/input-editor.md` and pull request 50. Pull request 50 has no review thread, and its one review approves it with no finding.

No code comes from `spikes/`. The spike code is an example only.

## Goals / Non-Goals

**Goals:**

- Every seam that a later change of Phase 1 fills in exists, with one real or fake implementation and its tests.
- The engine in `mu-core` has no protobuf type, no socket and no terminal. A test drives it with fakes and with a paused clock.
- A test can start the built binary, attach over the socket and decide each scenario.

**Non-Goals:**

- A stable API. The schema is `mu.v1`, but a later change of Phase 1 can change it. No client outside this repository exists yet.
- Speed. The journal and the stream need only to keep up with one person and a few worlds.
- Windows. The socket, the signals and the process group are Unix only, as the platforms in `openspec/config.yaml` say.

## Decisions

### Crates and their edges

```text
mu-proto   <- schema, generated messages and service stubs
mu-core    <- engine: Portal, sessions, journal, config, secrets, hooks (no proto)
mu-server  -> mu-core, mu-proto   API adapter, lock, signals
mu-client  -> mu-proto            PortalClient: calls and the stream with resume
mu-tui     -> mu-client, mu-proto terminal view
mu-cli     -> mu-core, mu-server, mu-tui, mu-client, mu-proto   verbs, socket path, wiring
```

Two crates are new. `mu-server` keeps the conversion between engine types and protobuf types in one place. `mu-client` holds the client of the API with its resume logic, so the terminal client, the tests and a later agent client share one implementation.

At head, no crate depends on another. The allowlist in `crates/mu-cli/tests/workspace_boundary.rs` and the base fixture of that test list the edge from `mu-core` to `mu-proto`. This change removes the edge from both, adds the two new crates to both, and adds the new edges.

The allowlist checks only the edges between members. A protobuf type can also reach `mu-core` through `buffa` or `connectrpc` from `[workspace.dependencies]`. So the boundary test gets a second list: the third-party crates that a member must not use. `mu-core` and `mu-tui` must not use `buffa`, `connectrpc`, `axum` or `connectrpc-build`. Fixture tests refuse each new route around the rule: `mu-core` to `buffa`, `mu-client` to `mu-core`, and `mu-tui` to `mu-server`.

The other choice was one crate for the server and the engine. It is smaller, but a protobuf type can then reach the engine, and no test fails.

Enforced by the boundary test.

### The engine: one actor for each world

`Portal` is the handle that the API adapter holds. It owns a map from `WorldId` to a session handle, one journal and the server handlers. Each session is one tokio task with an `mpsc` channel for commands. The task owns the transport of its world, its framer, its hooks and its handlers, so a world handles its events one at a time.

```text
API --Envelope--> Portal --mpsc--> Session task --> Framer --> Transport --> game
                                        |
                                     hooks (pass | replace | gag)
                                        v
                                    Journal.append --> broadcast --> Subscribe streams
                                        |
                                     handlers --> Vec<Command> --> Envelope --> Portal
```

The `Command` enum is the only way to act on a world: `Send`, `Connect`, `Disconnect`, `Login`, `Echo` and `Emit`. The API, a handler and a later timer all produce the same enum. Each command travels in an `Envelope` with its origin and its depth. The origin is `Player`, `Script` or `System`. The API sets `Player`, a handler sets `Script`, and the server start sets `System`. The source of each event comes from the origin of its command. Thus the event of a send from a handler has the source script.

A command from the API has the depth 0. A command from a handler gets the depth of its event plus one. At the depth limit, the engine drops the command and records one `script_error` event. No handler runs on a `script_error` event, so an error cannot start a second loop. The limit is 8, a constant in `mu-core`.

The other choice was one task for all worlds. It is simpler, but a slow hook stalls every world, which the [concurrent worlds decision](/decisions/concurrent-worlds/) rules out.

Enforced by the world-sessions tests below.

### Seams and their first implementations

| Seam | Trait or type in `mu-core` | In this change | Filled in by |
|---|---|---|---|
| Transport | `Transport`: opens a byte stream for a world | `FakeTransport` with an in-process fake game | #54 TCP, #58 TLS |
| Framer | `Framer`: bytes in, lines, prompts and reply bytes out, and a line in, bytes out | `LineFramer`, which splits on LF and drops CR | #52 telnet |
| Journal | `Journal`: `append`, `subscribe(from)` | `MemoryJournal` | #57 journal on disk |
| World configuration | `ResolvedWorld`: id, login template, login name, secret reference, hooks and handlers | built from the `--fake-world` flags | #53 layers and `extends` |
| Layer source | `LayerSource`: returns the fields of one layer | not used yet, the trait only | #53 TOML, #60 Lua |
| Secrets | `SecretStore`: `get(SecretRef) -> Secret` | `EnvSecretStore` for `env:<NAME>` | Keychain and Secret Service later |
| Hooks | `Hook`: `on_game_line`, `on_player_line` | none in the binary, a test hook in tests | #59 rules, #60 Lua |
| Handlers | `Handler`: `on_event -> Vec<Command>`, for a world and for the server | none in the binary, a test handler in tests | #59, #60 |

`LayerSource` has no caller in this change. It is in the list because #53 needs the trait and #60 adds Lua as a second source. A test in `mu-core` makes sure that a fake source returns its fields, so the trait does not rot.

The transport returns an `AsyncRead + AsyncWrite` stream. The framer sits between that stream and the session. `Framer::feed` takes the bytes of one read and returns a list of `Inbound`: a line, a prompt with no line end, or bytes to write back. `Framer::encode` takes one line and returns its bytes. The telnet engine of #52 returns its negotiation replies as `Inbound` bytes and flushes a prompt as a prompt. So #52 adds a framer, and the session does not change.

### The journal and the stream

`MemoryJournal` holds a `Vec` of events and a `tokio::sync::broadcast` sender under one `Mutex`. `subscribe(from)` takes the backlog from `from` and a new receiver under the same lock, so no event falls between them. The ConnectRPC decision records this pattern, and the spike tested it.

Each event has `seq`, `world`, `kind`, `source`, `text` and `time`. The kinds are `connected`, `disconnected`, `line`, `prompt`, `input`, `login`, `custom` and `script_error`. The sources are `game`, `player`, `script`, `system` and `credential`.

The live channel holds 1024 events. tokio rounds a broadcast capacity up to a power of two and panics at 0. So `MemoryJournal::new` refuses 0 and a capacity that is not a power of two. A receiver that lags gets `RecvError::Lagged`. `tokio-stream` turns the receiver into a stream with `BroadcastStream`, as the spike did, and the adapter maps the lag to `resource_exhausted`. A test with the capacity 4 makes sure that 4 unread events do not cut the stream, and that a fifth one cuts it.

Each server start makes a new random instance id with the `uuid` crate. The first message of each stream is `Hello { instance_id, last_seq }`, through a `oneof` in the stream message. A `from_seq` of 0 or 1 starts at the first event. A `from_seq` of `last_seq + 1` sends only the `Hello` and then the live events. A `from_seq` greater than `last_seq + 1` sends the `Hello` and then ends the stream with `out_of_range`. Thus a client that resumes after a restart always reads the new instance id first.

The other choice was to send the instance id in a response header. A header is harder to read in a generated client, and `buf curl` does not show it in the stream.

### Secrets and redaction

The server reads the secret of each world at the start, before it takes the lock. A missing variable, an empty value and a value that is not Unicode are errors that name the variable. The server exits with status 1, and no world connects.

`Secret` wraps the `secrecy` crate, so `Debug` and `Display` print no value, and the memory is zeroed on drop. Only the session can expose the value.

The session fills the login template `connect {login} {password}` itself. It writes the real line to the transport. It records the event with the text that the template gives for a password of `****`. Thus the login text with the password never exists as an event.

The player can also type the password, and the game can echo it. So the session masks each event of its world before it records the event. It replaces each occurrence of the password of that world with `****`. The journal is the only sink, so a later sink, for example a log on disk, gets the masked text.

Enforced by the redaction tests below, and by review for a new sink.

### The API

The schema is `crates/mu-proto/proto/mu/v1/portal.proto`, package `mu.v1`, service `PortalService`:

| Call | Request | Reply |
|---|---|---|
| `Subscribe` | `from_seq` | a stream of `Hello` or `Event` |
| `ListWorlds` | none | each world with its id and state |
| `Send` | `world`, `text` | an optional sequence number of the player event |
| `Connect` | `world` | empty |
| `Disconnect` | `world` | empty |

If a hook gagged the line, the reply of `Send` holds no sequence number. The server refuses a text with CR or LF with `invalid_argument`. Such a text writes two lines to the game, and the second one skips the hooks.

The engine returns a `PortalError` enum. The adapter maps `UnknownWorld` to `not_found`, `NotConnected` to `failed_precondition`, `LineBreak` to `invalid_argument` and `PastEnd` to `out_of_range`. One function holds the map, with a test for each arm. `NotConnected` covers the states `connecting` and `disconnected`.

The server serves the axum router of `connectrpc` on a tokio `UnixListener`, as the spike did. The client uses `Http2Connection::lazy_unix` from `connectrpc`. A task reads the current documentation of `connectrpc` before the work depends on these two facts.

### The client and its resume

`PortalClient` in `mu-client` wraps the generated client. Its method `events()` returns a stream of `ClientEvent`, one of `Reset`, `Event` and `Lost`. The method keeps the last sequence number and the instance id.

The stream can end with an error. If the server stops on a signal, the stream ends with no error. In both cases, `events()` yields `Lost` and subscribes again from the next number. The ConnectRPC decision says to resume on every error, and not to choose by code. On a new instance id, it yields `Reset` and starts from 1.

The `backon` crate gives the wait between two tries: an exponential backoff from 100 milliseconds to at most 5 seconds, with no end. The start of a server by the client uses the same crate, with a total limit of 5 seconds. A unit test in `mu-client` checks the schedule of the backoff with the paused clock.

The resume tests need a real server that records events and restarts. `mu-client` cannot depend on `mu-server`. So the resume tests live in `crates/mu-cli/tests/resume.rs` and run the built binary through the shared test helpers. The relay for these tests lives in `crates/mu-cli/tests/support/`.

### The socket path and its directory

The flag `--socket` with the variable `MU_SOCKET` is one `SocketArgs` struct in `mu-cli`. `ClientArgs` and the arguments of `mu server` both flatten it. The `env` feature of `clap` reads the variable. One function in `mu-cli` gives the default path from the spec.

One function in `mu-cli`, `private_dir`, creates a missing socket directory with mode 0700 and refuses an existing one that is not private. `rustix` reads the user id and the owner. The server and the client both call it before they write a file there.

### The server process

`mu-server` owns the life of the process:

1. Read the secret of each world. A failure ends the start with status 1.
2. Take the private directory from `private_dir`.
3. Take an exclusive lock on `<socket>.lock` with `File::try_lock` from std. If the lock is taken, the server reports `already` and exits with status 1.
4. If a file exists at the socket path, check its type. A socket file is stale, because the lock makes sure that no other server serves it, and the server removes it. Any other file is an error that names the path, and the server exits with status 1.
5. Bind, set mode 0600, and write the line with the path to stderr.
6. Build the `Portal` with the resolved worlds, and send `Connect` with the origin `System` to each world. Each session sends its login as soon as it connects.
7. Serve. On SIGTERM or SIGINT from `tokio::signal::unix`, stop the sessions, end the streams, remove the socket file and exit with status 0.

A lock file decides ownership. The other choice was to probe the socket with a connect. A probe has a race: two servers can both find no answer and both bind. The lock closes that race, so two clients that start at the same time get one server.

### The client starts a server

`mu client` follows these steps:

1. If stdin or stdout is not a terminal, write the error and exit with status 1. `std::io::IsTerminal` makes the check. Nothing else runs before it, so no directory and no file appears.
2. Call `private_dir` for the socket directory.
3. Try `ListWorlds` on the socket. If the connect fails, run `current_exe()` with `server --socket <path>`, with `process_group(0)` from std, stdin from `/dev/null`, and stdout and stderr appended to `server.log`.
4. Try again with `backon`, for up to 5 seconds.

A second client that starts at the same moment starts a second server. That server finds the lock taken, writes `already` to `server.log` and exits. Both clients attach to the first one.

### The terminal client

`mu-tui` has an `App` with three methods: `on_event`, `on_key` and `draw`. `on_key` returns an action, for example `Send { world, text }` or `Quit`, and does no I/O. A loop in `mu-tui` joins the key events of `crossterm`, the stream of `PortalClient` and the replies of the calls. Thus a test drives `App` with `ClientEvent` values and keys, with no terminal and no server. It reads the screen through the `TestBackend` of `ratatui`.

The worlds are in the order of their game part and then their character part. A server that the client started has no world. Then the top row shows `no worlds`, and Enter shows a line with the prefix `-- ` that says that no world has the focus.

The input line is a `TextArea` of `ratatui-textarea` from crates.io, with one line. The input editor decision chose that crate. Its two patches matter for long poses with wide text, so they wait for #56. Every item under Not tested of that decision also waits for #56. That change replaces this input line with the pose editor.

The tests of the built binary run `mu client` in a pseudo-terminal with `portable-pty`, and read the screen with `vt100`. A task reads the documentation of both crates before the tests depend on them.

### Test helpers

`crates/mu-cli/tests/support/mod.rs` holds every helper that more than one test file uses:

- `mu(args)`, moved from `cli.rs`, which runs the built binary to its exit
- `ServerProcess`, which starts `mu server` on a socket in a temporary directory and kills it on drop
- `Relay`, which forwards a Unix socket and can cut the connection
- `wait_until(bound, check)`, the one bounded wait. It fails the test at the bound and prints what it saw.

### Tests and the clock

The engine tests use `#[tokio::test(start_paused = true)]`, so the backoff and the timeouts do not wait on the wall clock. A test of the built binary waits through `wait_until` on an event, for example a line on stderr or an event on the stream. Nothing sleeps.

Two rules get property tests with `proptest`, which the workspace already has. `WorldId` parses each id that it displays, and refuses each string outside the rule. A journal of any length, cut at any point, gives each number once to a reader that resumes from the cut.

### Third-party crates

Each crate enters through `[workspace.dependencies]`, at the latest version at the time of the work. Renovate moves the pins after that.

| Crate | For |
|---|---|
| `tokio`, `tokio-stream` | the runtime, channels, signals, the Unix socket, and `BroadcastStream` |
| `connectrpc`, `connectrpc-build`, `buffa` | the API, as the spike used them |
| `axum` | the router that `connectrpc` serves |
| `futures` | stream combinators |
| `backon` | the backoff of the resume and of the start of a server |
| `thiserror` | error enums |
| `secrecy` | the password type |
| `uuid` | the server instance id |
| `rustix` | the user id and the owner of the socket directory |
| `ratatui`, `crossterm`, `ratatui-textarea` | the terminal client |
| `tracing`, `tracing-subscriber` | the log lines on stderr |
| `portable-pty`, `vt100` | tests of the terminal client only |

`clap` gets its `env` feature.

### What enforces each rule

| Rule | Enforced by |
|---|---|
| `mu-core` has no protobuf type | the boundary test, with the list of third-party crates |
| One sequence with no gap | the event-journal tests and the property test |
| No password in an event or on stderr | the redaction tests, and review for each new sink |
| One server for each socket | the server-process tests |
| No handler loop without a bound | the depth limit test and the script error test |
| No I/O in a hook or a handler | review, until #60 adds the Lua limits |
| No sleep in a test | `review-lint`, gate `time-based-test` |

## Risks / Trade-offs

- [The `connectrpc` crate is young and can change its API.] → The adapter in `mu-server` and the client in `mu-client` are the only users. `tonic` can serve the same schema, as the ConnectRPC decision says.
- [Tests in a pseudo-terminal can be slow or flaky on CI.] → Most client scenarios run against `App` and `TestBackend`. The pseudo-terminal tests cover only the scenarios that need the real binary. Each wait has a bound, and a failure prints the screen.
- [The Lua decision gives a hook a time budget. No hook in this change has a limit.] → The trait takes no budget yet. #60 adds the limit, with the Lua instruction limit.
- [A short password masks common text in the journal. For example, the password `a` masks every `a`.] → The mask keeps the password out of the journal, which matters more. #53 can refuse a password shorter than a minimum length.
- [`ratatui-textarea` without the patches counts width wrongly for wide text.] → The input line is a stand-in, and #56 replaces it.
- [A background server keeps running after the last client.] → This matches the decision that the server holds the game connections. `mu server` stops on SIGTERM.

## Migration Plan

None. No release exists. The stub output `not implemented` goes, and its tests go with it.

## Open Questions

- The default socket path on macOS uses the temporary directory of the user, because macOS has no `XDG_RUNTIME_DIR`. If the path length limit of a Unix socket allows it, a later change can move it to `~/Library/Application Support/mu/`. No test depends on the default on macOS. The tests name the socket with the flag, with `MU_SOCKET`, or with `XDG_RUNTIME_DIR`, so this choice does not change them.
