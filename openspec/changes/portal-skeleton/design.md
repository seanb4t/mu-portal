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
mu-cli     -> mu-core, mu-server, mu-tui, mu-client, mu-proto   verbs, socket path, private directory, wiring
```

Two crates are new. `mu-server` keeps the conversion between engine types and protobuf types in one place. `mu-client` holds the client of the API with its resume logic, so the terminal client, the tests and a later agent client share one implementation.

At head, no crate depends on another. The allowlist in `crates/mu-cli/tests/workspace_boundary.rs` and the base fixture of that test list the edge from `mu-core` to `mu-proto`. This change removes the edge from both, adds the two new crates to both, and adds the new edges.

The allowlist checks only the edges between members. A protobuf type can also reach `mu-core` through `buffa` or `connectrpc` from `[workspace.dependencies]`. So the boundary test gets a second allowlist: the third-party crates that each member can use, in the same form as the list of edges. `mu-core` and `mu-tui` get no crate of the wire format. Fixture tests refuse each new route around the rules, with the names of the existing edge tests: `core_to_proto_fails`, `core_to_buffa_fails`, `tui_to_connectrpc_fails`, `client_to_core_fails` and `tui_to_server_fails`.

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

The `Command` enum is the only way to act on a world: `Send`, `Connect`, `Disconnect`, `Login`, `Echo` and `Emit`. The API, a handler and a later timer all produce the same enum. Each command travels in an `Envelope` with its origin and its depth. The origin is `Player`, `Script` or `System`. The API sets `Player`, a handler sets `Script`, and the server start sets `System`.

The source of an event comes from its kind. A line or a prompt from the game has the source game. A connect and a disconnect have the source system, whatever command caused them. A login has the source credential. A custom event and a script error have the source script. A message of a side protocol has the source game. Only an input and an echo take their source from the origin of their command: `Player` gives player, `Script` gives script, and `System` gives system. Thus the event of a send from a handler has the source script. A `Connect` from a handler or from the API has the source system.

Each event in the engine carries a depth. An event from a command gets the depth of the command. A line from the game has the depth 0. A command from a handler gets the depth of its event plus one. At the depth limit, the engine drops the command and records one `script_error` event. No handler runs on a `script_error` event, so an error cannot start a second loop. The limit is 8, a constant in `mu-core`. The depth stays in the engine, and the schema does not carry it.

A line from the game starts again at the depth 0. So a handler that sends a line, and gets the echo of the game, makes a loop that the depth cannot see. A rate limit bounds that loop. Each world accepts commands from handlers at a rate of 20 in one second, with a burst of 20. The engine drops each command over the limit, and records at most one `script_error` event for each second. After a burst, the budget comes back one command each 50 milliseconds. A loop through the game stops at its first dropped command, because a dropped send gets no echo. The limit is a constant in `mu-core`, and a test with the paused clock checks it, also after the budget comes back.

The `governor` crate gives the limit, with `Quota::per_second(20)`. It is a token bucket, so it has no fixed window, and a burst at the edge of a window cannot pass 40 commands. Its `RateLimiter` takes any clock through its `Clock` trait. A small clock in `mu-core` reads `tokio::time::Instant`, so the paused clock of a test drives the limit. The engine adds the rule of one script error for each second on top.

The world handlers run in the task of their world. The server handlers run in one more task, the server-handler task. That task reads the journal through `subscribe`, as a client does. If it lags, it subscribes again from the number after the last event it read, because the journal keeps every event. Thus no server handler sees an event twice. Its commands go to the `Portal` with the depth of the event plus one.

The other choice was one task for all worlds. It is simpler, but a slow hook stalls every world, which the [concurrent worlds decision](/decisions/concurrent-worlds/) rules out.

Enforced by the world-sessions tests below.

### Seams and their first implementations

| Seam | Trait or type in `mu-core` | In this change | Filled in by |
|---|---|---|---|
| Transport | `Transport`: opens a byte stream for a world | `FakeTransport` with an in-process fake game | #54 TCP, #58 TLS |
| Framer | `Framer`: bytes in, lines, prompts and reply bytes out, and a line in, bytes out | `LineFramer`, which splits on LF and drops CR | #52 telnet |
| Journal | `Journal`: `append`, `subscribe(from)` | `MemoryJournal` | #57 journal on disk |
| World configuration | `ResolvedWorld`: id, login template, login name and secret reference. `WorldSetup` adds the password, the transport, the hooks and the handlers | built from the `--fake-world` flags | #53 layers and `extends` |
| Secrets | `SecretStore`: `get(SecretRef) -> Secret` | `EnvSecretStore` for `env:<NAME>` | Keychain and Secret Service later |
| Hooks | `Hook`: `on_game_line`, `on_player_line` | none in the binary, a test hook in tests | #59 rules, #60 Lua |
| Handlers | `Handler`: `on_event -> Vec<Command>`, for a world and for the server | none in the binary, a test handler in tests | #59, #60 |

The layer seam is the `Provider` trait of `figment`, which the [world-type profiles decision](/decisions/world-type-profiles/) chose. #53 adds `figment` with a TOML provider, and #60 adds a Lua provider. This change adds no trait of its own for it. `ResolvedWorld` is the output of the merge, and the sessions read only that type.

The transport returns an `AsyncRead + AsyncWrite` stream. The framer sits between that stream and the session. `Framer::feed` takes the bytes of one read and returns a list of `Inbound`. An `Inbound` is a line, a prompt with no line end, or bytes to write back. It can also be a message of a side protocol, with its name and its body. GMCP is an example of a side protocol. `LineFramer` never gives a message. The session records a message as an event of the kind `message`. `Framer::encode` takes one line and returns its bytes. The telnet engine of #52 returns its negotiation replies as `Inbound` bytes and flushes a prompt as a prompt. So #52 adds a framer, and the session does not change.

### The journal and the stream

`MemoryJournal` holds a `Vec` of events and a `tokio::sync::broadcast` sender under one `Mutex`. `subscribe(from)` takes the backlog from `from` and a new receiver under the same lock, so no event falls between them. The ConnectRPC decision records this pattern, and the spike tested it.

Each event has `seq`, `world`, `kind`, `source`, `text` and `time`. The kinds are `connected`, `disconnected`, `line`, `prompt`, `message`, `input`, `echo`, `login`, `custom` and `script_error`. An `Echo` command records one event of the kind `echo`, and sends nothing to the game. The sources are `game`, `player`, `script`, `system` and `credential`.

The live channel holds 1024 events. tokio rounds a broadcast capacity up to a power of two and panics at 0. So `MemoryJournal::new` refuses 0, a capacity that is not a power of two, and a capacity above 65536. A receiver that lags gets `RecvError::Lagged`. `Subscription::into_stream` in `mu-core` turns the backlog and the receiver into one stream that ends after a lag, and the adapter maps the lag to `resource_exhausted`. The receiver stays inside `mu-core`, so the adapter needs no `tokio-stream`. The edge is in the receiver, before the stream polls it. So a unit test in `mu-core` holds a receiver with the capacity 4 and reads nothing. After 4 events, its first read gives event 1. After 5 events, its first read gives the lag. The adapter test checks only that a lagged receiver ends the stream with `resource_exhausted`.

Each server start makes a new random instance id with the `uuid` crate. The first message of each stream is `Hello { instance_id, last_seq }`, through a `oneof` in the stream message. A `from_seq` of 0 or 1 starts at the first event. A `from_seq` of `last_seq + 1` sends only the `Hello` and then the live events. A `from_seq` greater than `last_seq + 1` sends the `Hello` and then ends the stream with `out_of_range`. Thus a client that resumes after a restart always reads the new instance id first.

The other choice was to send the instance id in a response header. A header is harder to read in a generated client, and `buf curl` does not show it in the stream.

### Secrets and redaction

The server reads the secret of each world at the start, before it takes the lock. A missing variable, an empty value and a value that is not Unicode are errors that name the variable. The server exits with status 1, and no world connects.

`Secret` wraps the `secrecy` crate, so `Debug` and `Display` print no value, and the memory is zeroed on drop. Only the session can expose the value.

The session fills the login template `connect {login} {password}` itself. It writes the real line to the transport. It records the event with the text that the template gives for a password of `****`. Thus the login text with the password never exists as an event.

The player can also type the password, and the game can echo it. So the session masks each event of its world before it records the event. It replaces each occurrence of the password of that world with `****`. A password can hold `*`, so the replacement alone can form the password again. For example, the password `a*` in the text `aa*` gives `a****`, which still holds `a*`. So if the masked text still holds the password, the session replaces the whole text with `****`. A password that `****` itself holds, one to four stars, cannot be masked at all. So step 1 of the server refuses it, and also a password with CR or LF, which splits the login line. The error names the world and the variable, never the password. A property test makes sure that no masked text holds its password, for any text and any password. The journal is the only sink, so a later sink, for example a log on disk, gets the masked text.

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

The API carries binary protobuf only. `connectrpc-build` runs with `generate_json(false)`, and `connectrpc` has its default features off, so no crate needs `serde`. A JSON request gets `unimplemented`. `buf curl` still works, because it takes JSON on its command line and sends binary protobuf.

The server serves the axum router of `connectrpc` on a tokio `UnixListener`, as the spike did. The client uses `Http2Connection::lazy_unix` from `connectrpc`. A task reads the current documentation of `connectrpc` before the work depends on these two facts.

### The client and its resume

`PortalClient` in `mu-client` wraps the generated client. Its method `events()` returns a stream of `ClientEvent`, one of `Reset`, `Resumed`, `Event` and `Lost`. The method keeps the last sequence number and the instance id.

The stream can end with an error. If the server stops on a signal, the stream ends with no error. In both cases, `events()` yields `Lost` and subscribes again from the next number. The ConnectRPC decision says to resume on every error, and not to choose by code. After the `Hello` of each new stream, it yields `Resumed` for the same instance id. On a new instance id, it yields `Reset` and starts from 1. The id belongs to the start of the server, because the journal lives in memory. When #57 keeps the journal on disk, the id moves into the journal, so a restart does not reset each client. Thus the client learns at once that the stream is back. A new event is not necessary.

The `backon` crate gives the wait between two tries: an exponential backoff from 100 milliseconds to at most 5 seconds, with no end. The `ExponentialBuilder` of `backon` stops after 3 retries by default, so the client calls `without_max_times`. A test makes sure that the backoff runs past the fourth try. The start of a server by the client uses the same crate, with a total limit of 5 seconds. If no server answers in that time, the client writes an error that names the socket and `server.log`, and exits with status 1. A unit test in `mu-client` checks the schedule of the backoff with the paused clock.

The resume tests need a real server that records events and restarts. `mu-client` cannot depend on `mu-server`. So the resume tests live in `crates/mu-cli/tests/resume.rs` and run the built binary through the shared test helpers. The relay for these tests lives in `crates/mu-cli/tests/support/`.

### Arguments

`clap` parses each `--fake-world` value with a value parser into a `WorldId` and a `SecretRef`. A bad id is then a usage error, with the exit status 2 of `clap`. A missing variable is found later, at step 1 of the server, with the exit status 1.

### The socket path and its directory

The flag `--socket` with the variable `MU_SOCKET` is one `SocketArgs` struct in `mu-cli`. `ClientArgs` and the arguments of `mu server` both flatten it. The `env` feature of `clap` reads the variable. One function in `mu-cli` gives the default path from the spec.

One function in `mu-cli`, `private_dir`, creates a missing socket directory with mode 0700 and refuses an existing one that is not private. It takes the expected owner as an argument, so a test can pass another user id. `rustix` reads the user id and the owner. The verbs `mu server` and `mu client` both call it before they write a file there. `mu-server` does not call it. It gets the socket path after the check.

An empty or relative `XDG_RUNTIME_DIR` counts as unset, so the default path is always absolute. An empty `MU_SOCKET` also counts as unset. The value parser of `SocketArgs` turns an empty value into none, so `clap` does not refuse it.

### The server process

`mu server` runs these steps. `mu-cli` does steps 1 and 2, and then calls `mu-server` for the rest:

1. Read the secret of each world through the `SecretStore`. A failure ends the start with status 1.
2. Call `private_dir` for the socket directory.
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

`mu-tui` has an `App` with four methods: `on_event`, `on_reply`, `on_key` and `draw`. `on_key` returns an action, for example `Send { world, text }` or `Quit`, and does no I/O. The loop runs the call of each action. It gives the result to `on_reply`, for example a refused `Send` with its reason. A loop in `mu-tui` joins three inputs: the `EventStream` of `crossterm`, the stream of `PortalClient` and the replies of the calls. Thus a test drives `App` with `ClientEvent` values, replies and keys, with no terminal and no server. It reads the screen through the `TestBackend` of `ratatui`.

The worlds are in the order of their game part and then their character part. A server that the client started has no world. Then the top row shows `no worlds`, and Enter shows a line with the prefix `-- ` that says that no world has the focus. Ctrl+N and Ctrl+P do nothing with no world. The focus is an index that the code checks, never a division by the count. A property test runs any sequence of keys and events. It makes sure that the focus is then on a world that exists, or on none with no world.

The input line is a `TextArea` of `ratatui-textarea` from crates.io, with one line. The input editor decision chose that crate. Its two patches matter for long poses with wide text, so they wait for #56. Every item under Not tested of that decision also waits for #56. That change replaces this input line with the pose editor.

The tests of the built binary run `mu client` in a pseudo-terminal with `portable-pty`, and read the screen with `vt100`. `portable-pty` puts stdin, stdout and stderr all on the terminal. The two tests of a missing terminal need the terminal on one stream only. So a helper opens a pseudo-terminal with `rustix::pty`. It starts the binary with `std::process::Command`, with the terminal on one stream and a pipe on the other. A task reads the documentation of these crates before the tests depend on them.

### Test helpers

`crates/mu-cli/tests/support/mod.rs` holds every helper that more than one test file uses:

- `mu(args)`, moved from `cli.rs`, which runs the built binary to its exit, and `mu_with(args, env)`, which also sets and removes environment variables
- `ServerProcess`, which starts `mu server` on a socket in a temporary directory and kills it on drop. Its builder takes the arguments and the environment
- `Relay`, which forwards a Unix socket. It can cut the connection and hold, which refuses each new connection until the test releases it
- `mu_on_pty(args, stream)`, which starts the binary with a pseudo-terminal on stdin only or on stdout only. `server.rs` and `client.rs` both use it
- `wait_until(bound, check)`, the one bounded wait. It fails the test at the bound and prints what it saw.

### Tests and the clock

The engine tests use `#[tokio::test(start_paused = true)]`, so the backoff and the timeouts do not wait on the wall clock. A test of the built binary waits through `wait_until` on an event, for example a line on stderr or an event on the stream. Nothing sleeps.

Six rules get property tests with `proptest`, which the workspace already has:

- `WorldId` parses each id that it displays, and refuses each string outside the rule.
- A journal of any length, cut at any point, gives each number once to a reader that resumes from the cut.
- Appends and subscribes in any order give each subscriber the numbers in order, with no gap.
- No masked text holds its password, for any text and any password.
- `LineFramer` gives the same lines for any split of the same bytes into reads.
- Any sequence of keys and events leaves the focus of `App` on a world that exists.

### Third-party crates

Each crate enters through `[workspace.dependencies]`, at the latest version at the time of the work. Renovate moves the pins after that. A Renovate group named `crossterm` holds `crossterm` and `ratatui`, so the two always update in one pull request. `cargo deny` refuses two copies of `crossterm` in the build, so a pull request that splits them fails.

| Crate | For |
|---|---|
| `tokio` | the runtime, channels, signals and the Unix socket. The paused clock needs the `test-util` feature of `tokio` in the dev-dependencies |
| `connectrpc`, `connectrpc-build`, `buffa` | the API, as the spike used them |
| `axum` | the router that `connectrpc` serves |
| `futures` | stream combinators |
| `backon` | the backoff of the resume and of the start of a server |
| `thiserror` | error enums |
| `secrecy` | the password type |
| `uuid` | the server instance id |
| `rustix` | the user id and the owner of the socket directory |
| `ratatui`, `ratatui-textarea`, `crossterm` | the terminal client. `crossterm` is its own dependency, because the async key stream needs its `event-stream` feature, and `ratatui` cannot turn that feature on. Its version is the one that `ratatui` uses, so the build has one copy |
| `tracing`, `tracing-subscriber` | the log lines on stderr |
| `portable-pty`, `vt100` | tests of the terminal client only |
| `governor` | the rate limit of handler commands |

`clap` gets its `env` feature.

`connectrpc-build` runs `protoc`, so `mise.toml` pins `protoc` as a tool. CI and each developer get it from there.

### What enforces each rule

| Rule | Enforced by |
|---|---|
| `mu-core` has no protobuf type | the boundary test, with the list of third-party crates |
| One sequence with no gap | the event-journal tests and the property test |
| No password in an event or on stderr | the redaction tests, and review for each new sink |
| One server for each socket | the server-process tests |
| No handler loop without a bound | the depth limit test, the script error test and the rate limit test |
| The focus is always a world that exists | the property test of `App` |
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

- The default socket path on macOS uses the temporary directory of the user, because macOS has no `XDG_RUNTIME_DIR`. If the path length limit of a Unix socket allows it, a later change can move it to `~/Library/Application Support/mu/`. Most tests of the binary name the socket with the flag, with `MU_SOCKET`, or with `XDG_RUNTIME_DIR`. Two tests run without a usable `XDG_RUNTIME_DIR`: the unit test of the default and the binary test with an empty value. Both check the path against the temporary directory of the platform, not a fixed path. So this choice does not change the tests.
