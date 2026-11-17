# Tasks

## 1. Documentation and dependencies

- [x] 1.1 Read the sources of each fact that the design uses:
  - RFC 854 (the commands and `CR NUL`), RFC 855 (IAC in a subnegotiation) and RFC 860 (TIMING-MARK)
  - RFC 1073 (NAWS), RFC 1091 (TTYPE), RFC 1143 (section 7) and RFC 2066 (CHARSET and `[TTABLE ]`)
  - the MTTS and MCCP2 specifications
  - the current documentation of `miniz_oxide`, with Context7: `inflate::stream::inflate`, `InflateState`, the count of used input bytes, and the deflate side

  Fill the table "Checked facts" of the design with one row for each fact and its source section. Make sure that each difference from the sources is an edit of the design that its row names.
- [x] 1.2 Add `miniz_oxide` and `hex` to `[workspace.dependencies]` in `Cargo.toml`. Add `miniz_oxide` to the dependencies of `mu-core`, and `hex` to its dev-dependencies. Add both to the list of `mu-core` in `ALLOWED_CRATES` of `crates/mu-cli/tests/workspace_boundary.rs`. Make sure that `cargo deny check` and `cargo nextest run -p mu-cli --test workspace_boundary --locked` pass, and that Renovate reads both pins from `Cargo.toml`.

## 2. The parser

Load the `rust-best-practices` skill before this group.

- [ ] 2.1 Make `telnet` a public module of `mu-core`. Add each command code from 239 to 255 and each option code of the design to `src/telnet.rs`. Add `telnet::escape` and `telnet::MARK` there, and no other logic. In `crates/mu-cli/tests/source_rules.rs`, add the commands from 239 to 250 to `CODES`. Move the case `let x = 250;` of `bytes_that_are_not_codes_pass` to 238, and add a case for 239 to `telnet_code_is_found_in_each_form`. Add the rule for an option code after `WILL`, `WONT`, `DO`, `DONT` or `SB`, with a case for each name and the passing case `[IAC, WILL, opt::GMCP]`. Rename the private `Side` of `framer.rs` to `Role`. Add `opt::STATUS`, and change the `[IAC, WILL, 5]` cases of the `LineFramer` tests to it. Make `LineFramer::mark` and the tests of `fake.rs` use `telnet::MARK`. Add `src/telnet/q.rs` with the states and steps of RFC 1143, section 7, and a unit test for each row of its table. Make sure that `cargo nextest run -p mu-core --locked` and `cargo nextest run -p mu-cli --test source_rules --locked` pass.
- [ ] 2.2 Add `src/telnet/parser.rs` with `Parser`, `Policy`, `Side`, `Event` and `ProtocolError`, as the design describes, with no MCCP2 yet. Add `reads(bytes, points)` to `crates/mu-core/tests/support/mod.rs`. Add `crates/mu-core/tests/telnet_parser.rs` with these tests:
  - `escaped_byte_in_data` and `byte_255_in_a_subnegotiation`, for the scenarios "An escaped byte in data" and "The byte 255 in a subnegotiation to the game"
  - `ayt_before_data`, `each_two_byte_command`, `se_outside_a_subnegotiation` and `byte_that_is_no_command`, for the scenarios of the requirement "Two-byte commands are two bytes"
  - `accepted_option`, `refused_option_twice`, `repeated_request`, `crossing_requests` and `refusal_of_a_client_request`, for the five scenarios of the Q method
  - `payload_with_escaped_byte`, `payload_at_the_bound` and `payload_over_the_bound`, for the three scenarios of subnegotiation
  - `mark_from_the_game`, for the scenario "A mark from the game"
  - `malformed_framing`, with one case for each row of the table "Malformed framing" of the design

  Make sure that `cargo nextest run -p mu-core --test telnet_parser --locked` passes.
- [ ] 2.3 Add MCCP2 to the parser with the streaming inflate of `miniz_oxide`. This task covers two requirements: "MCCP2 is inflated inside the parser" and "A bad compressed stream ends the reading". Add these tests:
  - `stream_starts_inside_a_read`, `plain_bytes_after_the_stream`, `start_inside_a_stream`, `second_stream_after_end` and `start_while_mccp2_is_off`, for the five scenarios of MCCP2
  - `stream_that_is_not_zlib`, for the scenario "A stream that is not zlib"
  - `bad_stream_after_data`: a stream that breaks after some data, split at every byte, gives one protocol error and no event after it each time

  Make sure that `cargo nextest run -p mu-core --test telnet_parser --locked` passes.
- [ ] 2.4 Add the oracle to `crates/mu-core/tests/telnet_parser.rs`. It generates random streams from parts, with the events that each part must give. An optional last part is a stream with a bad zlib header. Malformed framing is a part too: `IAC SB IAC SE`, a negotiation verb inside a subnegotiation, and `IAC` as the last byte. Add the property tests `parser_matches_the_oracle` and `parser_ignores_read_splits`, for the scenario "Random streams in random reads". Add `parser_round_trip`, which sends data through `telnet::escape` and payloads through `send_subnegotiation`. Make sure that `cargo nextest run -p mu-core --test telnet_parser --locked` passes.

## 3. The transcripts

- [ ] 3.1 Copy the six files of `spikes/telnet-engine/transcripts/` to `crates/mu-core/tests/fixtures/telnet/`. Add a parser of the transcript format to `crates/mu-core/tests/support/mod.rs`, with `hex` for the bytes. Add `crates/mu-core/tests/telnet_transcripts.rs` with these tests:
  - `transcripts_have_no_protocol_error` and `transcripts_in_other_reads`, for the scenario "The recorded transcripts in other reads"

  In `crates/mu-cli/tests/workspace_boundary.rs`, add `Violation::SpikeLink` for a symbolic link in a member directory whose target resolves into the `spikes` directory of the workspace root. Add the probe test `spike_link_fails`, which makes such a link in a `TreeCopy` of the fixture. Make sure that `cargo nextest run -p mu-core --test telnet_transcripts --locked` and `cargo nextest run -p mu-cli --test workspace_boundary --locked` pass.

## 4. The telnet framer

Load the `rust-best-practices` skill before this group.

- [ ] 4.1 Add the crate-private `LineBuffer` to `framer.rs`, with a unit test for each of its parts. Move these parts of `LineFramer` into it: the partial line, the LF split, the drop of CR and the cut at `MAX_LINE`. Change the cut so that a full partial line followed by LF is one line, and so that a CR never starts a cut. Add the unit test `line_framer_line_of_exactly_the_bound`, for the scenario "A line of exactly the bound", with LF and with CR LF. Make sure that `cargo nextest run -p mu-core --locked` passes with every existing `line_framer_*` test unchanged.
- [ ] 4.2 Add `Charset`, `TelnetSettings`, with the character set, the window size and the accept policy, and `TelnetFramer` to `framer.rs` and export both from `lib.rs`. The framer gives lines, prompts, replies, marks, GMCP messages and `telnet` error messages, as the design describes. Add `crates/mu-core/tests/telnet_framer.rs` with these tests:
  - `line_to_the_game`, for the scenario "A line to the game"
  - `line_ends`, `cr_nul_across_reads`, `long_run_with_no_line_end` and `line_of_exactly_the_bound`, with LF and with CR LF, for the four scenarios of lines
  - `latin1_line_from_the_game`, `utf8_line_from_the_game` and `latin1_line_to_the_game`, and the property test `latin1_round_trip`, for the four scenarios of the character set
  - `prompt_with_no_ga`, `prompt_with_ga`, `line_split_across_reads`, `read_that_ends_at_a_line_end`, `ga_after_a_line_end`, `negotiation_alone` and `same_prompt_after_a_line`, for the prompt scenarios of the same names
  - the property test `telnet_framer_lines_ignore_read_splits`, for the scenario "Lines in any split"
  - `two_marks_in_a_row` and `answer_to_a_mark`, for the scenarios of the same names
  - `gmcp_message`, for the scenario "A GMCP message"
  - `four_ttype_requests`, for the scenario "Four requests"
  - `naws_turns_on` and `size_with_byte_255`, for the two scenarios of NAWS
  - `charset_is_offered`, `charset_request_with_ttable`, `charset_is_not_offered` and `ttable_prefix_with_no_version`, for the four CHARSET scenarios
  - the property test `framer_survives_any_payload`, for the scenario "Any payload"
  - `subnegotiation_over_the_bound` and `bad_compressed_stream`, for the two error scenarios

  Make sure that `cargo nextest run -p mu-core --test telnet_framer --locked` passes.
- [ ] 4.3 Add a paragraph to `docs/src/content/docs/architecture/server-and-clients.mdx` on how a game session reads telnet: the parser, the framer, and what the session gets from them. Follow the `simple-english` skill and link the telnet-engine decision page. Make sure that `mise run docs:check` passes.

## 5. Checks

- [ ] 5.1 Run `mise run check` and `uv run ~/.agents/plugins/sb/skills/review-lint/scripts/review_lint.py --base origin/main`. Make sure that both pass.

## 6. Review

- [ ] 6.1 Run `/sb:openspec-review telnet-engine implementation`. Make sure that the verdict in `review.md` is not changes-requested.
