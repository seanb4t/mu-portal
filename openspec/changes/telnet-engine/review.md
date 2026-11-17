---
verdict: approve-with-notes
phase: implementation
reviewed_at: 2026-11-17T02:25:35+00:00
head: 3515e234ce3b95811611dcbc7f0d2e51b4f03bfa
round: 5
next_id: 86
lenses:
  spec-lint: not-applicable
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: ok
  invariants: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-76 | approach,invariants | should-fix | in-change | app-1: TelnetFramer::encode now sends each CR of a line as CR NUL, as the spec asks, but both design decisions on encode still say it gives the escaped line in the character set and then CR LF, with no CR NUL step and no recorded deviation.; inv-2: Each CR of a line to the game goes as CR NUL, but the only test puts the CR in the middle of the line. No test has a CR as the last byte before the CR LF end, two adjacent CRs, or a CR next to an escaped 255.; inv-3: TelnetFramer::encode now combines two escapes, IAC doubling and CR NUL, over an unbounded domain of lines. No property test feeds encode's output back to a TelnetFramer; parser_round_trip uses telnet::escape alone and latin1_round_trip skips CR. | crates/mu-core/src/framer.rs:425 — "if byte == b'\\r' {" | app-1: Add the CR NUL step to the encode decision in 'The answers of a MU client' (after escape, each CR gets a NUL, then CR LF), and name it in 'The character set of the text'.; inv-2: Extend bare_cr_in_a_line_to_the_game with the cases "a\\r" -> a CR NUL CR LF, "\\r\\r" -> CR NUL CR NUL CR LF, and a Latin-1 "\\r\\u{ff}" -> CR NUL IAC IAC CR LF.; inv-3: Add a proptest (the repository uses proptest; see parser_round_trip in crates/mu-core/tests/telnet_parser.rs) to telnet_framer.rs: for any string with no LF and shorter than MAX_LINE, framer().feed(&framer().encode(s)) == [Inbound::Line(s with each CR removed)], for both Charset values. |  |  |
| R-77 | approach,invariants | should-fix | in-change | app-2: The parser now answers IAC DO TIMING-MARK with WONT before the Q table, whatever the policy, but the decision still says DO TIMING-MARK goes through the table like a refused option; read as written, a policy that accepts TIMING-MARK would answer WILL, against the scenario with such a policy.; inv-4: The new TIMING-MARK branch of negotiate drops IAC DONT TIMING-MARK in its catch-all arm, where before it went through the Q method. WILL, WONT and DO each have a test; no test sends DONT TIMING-MARK to show it gives no reply, no event and no state change. | crates/mu-core/src/telnet/parser.rs:383 — "DO => self.send_negotiation(Side::Local, option, false)," | app-2: Rewrite the decision: the parser answers IAC DO TIMING-MARK with IAC WONT TIMING-MARK outside the option table, whatever the policy, and changes no state.; inv-4: Add DONT to mark_refused_whatever_the_policy or a new parser test: receive IAC DONT TIMING-MARK and assert no event, no output and TIMING-MARK off on both sides. |  | severity? blocker 0.70 |
| R-78 | approach | should-fix | in-change | The checked fact says that after a decompression error the client may send DONT MCCP2 and go on as plain telnet, and that this design does that. The design and the parser drop every byte until IAC WONT MCCP2 instead, so the design does not go on as plain telnet. | design.md > Checked facts — "After a decompression error, the MCCP specification lets the client send `DONT MCCP2` and go on as plain telnet. This de" | Change 'This design does that.' to say the design sends DONT MCCP2 and then drops bytes until WONT MCCP2, a difference that the design chooses, as the spike rows do. |  |  |
| R-63 | fit | should-fix | in-change | The framer tests still build the MCCP2 start IAC SB MCCP2 IAC SE inline twice, and the parser test keeps its own START constant, though support::subnegotiation(opt::MCCP2, &[]) gives the same bytes. | crates/mu-core/tests/telnet_framer.rs:363 — "let first = [&[IAC, SB, opt::MCCP2, IAC, SE][..], &[0x00, 0x01, 0x02]].concat();" | Write the MCCP2 start as subnegotiation(opt::MCCP2, &[]) in telnet_framer.rs, or move START into tests/support/mod.rs and use it from both files. | regression |  |
| R-79 | fit | should-fix | in-change | TelnetFramer::data now drops each CR itself before it feeds LineBuffer, so the drop of CR has two homes: LineBuffer::push and TelnetFramer::data. The design names LineBuffer as the one home of the CR drop. | crates/mu-core/src/framer.rs:317 — "if byte == b'\\r' \|\| (after_cr && byte == 0) {" | Let LineBuffer::push report whether it kept the byte, and have TelnetFramer::data drop only the NUL after a CR and set `added` from that result. |  |  |
| R-80 | fit | should-fix | in-change | events() of the transcript test feeds each read to a parser, collects the events and joins them, which run_reads of the parser test already does. The fix of the join moved only join_data to support, so the read loop is still written twice. | crates/mu-core/tests/telnet_transcripts.rs:16 — "fn events(parts: &[&[u8]]) -> Vec<Event> {" | Move run_reads (with drain) to crates/mu-core/tests/support/mod.rs beside join_data and reads, and have the transcript test call run_reads(Parser::new(policy()), parts).0. |  |  |
| R-81 | fit | should-fix | in-change | The new test bad_stream_that_breaks_inside_a_command spells the answer IAC WONT MCCP2 inline, as plain_bytes_after_the_answer_to_dont does, while the same file holds the constant WONT_MCCP2 for these bytes. | crates/mu-core/tests/telnet_parser.rs:400 — "&[IAC, WONT, opt::MCCP2]," | Use WONT_MCCP2 in both tests, and move the constant up beside START. |  |  |
| R-82 | invariants | should-fix | in-change | The TTYPE gate reads the option state after the parser has read the whole read, not at the request. A read holding the request and then IAC DO TTYPE gets an IS answer (the scenario's own case), and a request then IAC DONT TTYPE in one read gets none, so the reply also differs by read split. The only test sends the request alone. | specs/telnet-protocol/spec.md > Scenario: A request while TTYPE is off — "the game sends `IAC SB TTYPE SEND IAC SE` before `IAC DO TTYPE`" | Track the TTYPE state in TelnetFramer from Event::Enabled/Disabled(Side::Local, opt::TTYPE) in event order and gate on that flag. Add a test that feeds the request and IAC DO TTYPE in one read and expects only IAC WILL TTYPE, and one that feeds the request and IAC DONT TTYPE after TTYPE is on and expects the IS answer. |  |  |
| R-83 | fit | suggestion | in-change | mark_refused_whatever_the_policy repeats the body of mark_from_the_game: the same two DO TIMING-MARK and the same drained answer. Only the policy differs. | crates/mu-core/tests/telnet_parser.rs:423 — "let mut parser = Parser::new(Policy::none().accept(Side::Local, &[opt::TIMING_MARK]));" | Add one helper that takes the policy, sends two DO TIMING-MARK and checks the two WONT answers; call it from both scenario tests. |  |  |
| R-84 | fit | suggestion | in-change | bad_stream_that_breaks_inside_a_command repeats the split loop of plain_bytes_after_the_answer_to_dont: each split point, two reads, the error text removed, the same compare. Only the bytes and the expected events differ. | crates/mu-core/tests/telnet_parser.rs:405 — "let (events, _) = run_reads(mccp2_parser(), &[&bytes[..point], &bytes[point..]]);" | Add a helper with the parameters bytes and expected events that checks every split in two reads, and call it from both tests. |  |  |
| R-85 | fit | suggestion | in-change | spike_directory_link_fails repeats spike_link_fails line for line; only the target in spikes/ and the link name differ. The same file runs its other spike cases from one list, as spike_include_fails does. | crates/mu-cli/tests/workspace_boundary.rs:710 — "let copy = fixture().append_file(Path::new("spikes/telnet/transcripts/a.txt"), "x");" | Keep one probe test of the link rule that loops over (target, link) cases: a file and a directory. |  |  |

## Repeats

| id | lens | severity | claim | evidence |
|---|---|---|---|---|
| R-73 | test-coverage | should-fix | covered row 'Random streams in random reads' has no verification run | review > not-run — "Random streams in random reads" |
| R-75 | test-coverage | should-fix | covered row 'A line of exactly the bound' has no verification run | review > not-run — "A line of exactly the bound" |

## Out of bound

| lens | severity | claim | evidence |
|---|---|---|---|
| test-coverage | note | Two tests match no scenario and no task line: six_transcripts_are_present and answer_to_a_mark_changes_no_option. | crates/mu-core/tests/telnet_transcripts.rs:27 — "fn six_transcripts_are_present() {" |

## Accepted

None.

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | 2a79f9d68b56c2d9e17e2b3b107f0e20538971d7 | d58807b3825d91749fd40b328c5cd343fa4d76ac7afc17ee30af462eec2798d0 | changes-requested | 29 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | 77e5faf2b3f7aec28818dcfc7ccc7f9e7a9a131f | bccb84cd0e1f28d44dd6e28d4e0632d1f621c4d94068fbc6047ff6739030e924 | changes-requested | 16 | 0 | 2 | 0 |  |
| 3 | plan | follow-up | e75dc3405a6b87176b209c3f52cd660fef8f7073 | 8b7fd5733aaaba9bea9fff517c379c81f7d5a753ddff336748ac81cb9ae8b795 | approve-with-notes | 13 | 0 | 0 | 0 |  |
| 4 | implementation | full | 9a60b3922a66172ac5e87f0ce81a4e1a7e0c8120 | 3af2d398e309a213c030a3c38bd5626ca4031fc35795d57a8effadcb738ed57e | approve-with-notes | 17 | 0 | 1 | 0 |  |
| 5 | implementation | follow-up | 3515e234ce3b95811611dcbc7f0d2e51b4f03bfa | c728e94c6f93b9a5e32a48c743007432f64dfaff9f40e2c1f40e3fff5517c6e5 | approve-with-notes | 10 | 2 | 1 | 1 |  |

## Judge

judge: 22 questions, 0.001 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| An escaped byte in data | crates/mu-core/tests/telnet_parser.rs: escaped_byte_in_data | covered | pass |
| The byte 255 in a subnegotiation to the game | crates/mu-core/tests/telnet_parser.rs: byte_255_in_a_subnegotiation | covered | pass |
| A line to the game | crates/mu-core/tests/telnet_framer.rs: line_to_the_game | covered | pass |
| AYT before data | crates/mu-core/tests/telnet_parser.rs: ayt_before_data | covered | pass |
| Each two-byte command | crates/mu-core/tests/telnet_parser.rs: each_two_byte_command | covered | pass |
| SE outside a subnegotiation | crates/mu-core/tests/telnet_parser.rs: se_outside_a_subnegotiation | covered | pass |
| A byte that is no command | crates/mu-core/tests/telnet_parser.rs: byte_that_is_no_command | covered | pass |
| An accepted option | crates/mu-core/tests/telnet_parser.rs: accepted_option | covered | pass |
| A refused option, twice | crates/mu-core/tests/telnet_parser.rs: refused_option_twice | covered | pass |
| A repeated request | crates/mu-core/tests/telnet_parser.rs: repeated_request | covered | pass |
| A request of the client that crosses a request of the game | crates/mu-core/tests/telnet_parser.rs: crossing_requests | covered | pass |
| A refusal of a request of the client | crates/mu-core/tests/telnet_parser.rs: refusal_of_a_client_request | covered | pass |
| An answer that the client did not ask for | crates/mu-core/tests/telnet_parser.rs: unrequested_answer_is_an_error | covered | pass |
| A payload with an escaped byte | crates/mu-core/tests/telnet_parser.rs: payload_with_escaped_byte | covered | pass |
| A payload at the bound | crates/mu-core/tests/telnet_parser.rs: payload_at_the_bound | covered | pass |
| A payload over the bound | crates/mu-core/tests/telnet_parser.rs: payload_over_the_bound | covered | pass |
| A stream that starts inside a read | crates/mu-core/tests/telnet_parser.rs: stream_starts_inside_a_read | covered | pass |
| Plain bytes after the stream | crates/mu-core/tests/telnet_parser.rs: plain_bytes_after_the_stream | covered | pass |
| A start inside a stream | crates/mu-core/tests/telnet_parser.rs: start_inside_a_stream | covered | pass |
| A second stream after the end of the first | crates/mu-core/tests/telnet_parser.rs: second_stream_after_end | covered | pass |
| A start while MCCP2 is off | crates/mu-core/tests/telnet_parser.rs: start_while_mccp2_is_off | covered | pass |
| A stream that is not zlib | crates/mu-core/tests/telnet_parser.rs: stream_that_is_not_zlib | covered | pass |
| Plain bytes after the answer to DONT | crates/mu-core/tests/telnet_parser.rs: plain_bytes_after_the_answer_to_dont | covered | pass |
| A stream that breaks inside a command | crates/mu-core/tests/telnet_parser.rs: bad_stream_that_breaks_inside_a_command | covered | pass |
| Random streams in random reads | crates/mu-core/tests/telnet_parser.rs: parser_matches_the_oracle, parser_ignores_read_splits | covered | not-run |
| The recorded transcripts in other reads | crates/mu-core/tests/telnet_transcripts.rs: transcripts_in_other_reads | covered | pass |
| Line ends | crates/mu-core/tests/telnet_framer.rs: line_ends | covered | pass |
| A bare CR in a line to the game | crates/mu-core/tests/telnet_framer.rs: bare_cr_in_a_line_to_the_game | covered | pass |
| Lines in a compressed stream | crates/mu-core/tests/telnet_framer.rs: compressed_lines | covered | pass |
| CR NUL across reads | crates/mu-core/tests/telnet_framer.rs: cr_nul_across_reads | covered | pass |
| A long run with no line end | crates/mu-core/tests/telnet_framer.rs: long_run_with_no_line_end | covered | pass |
| A line of exactly the bound | crates/mu-core/tests/telnet_framer.rs: line_of_exactly_the_bound; crates/mu-core/src/framer.rs: line_framer_line_of_exactly_the_bound | covered | not-run |
| A Latin-1 line from the game | crates/mu-core/tests/telnet_framer.rs: latin1_line_from_the_game | covered | pass |
| A UTF-8 line from the game | crates/mu-core/tests/telnet_framer.rs: utf8_line_from_the_game | covered | pass |
| Every Latin-1 byte | crates/mu-core/tests/telnet_framer.rs: latin1_round_trip | covered | pass |
| A Latin-1 line to the game | crates/mu-core/tests/telnet_framer.rs: latin1_line_to_the_game | covered | pass |
| A prompt with no GA | crates/mu-core/tests/telnet_framer.rs: prompt_with_no_ga | covered | pass |
| A prompt with GA | crates/mu-core/tests/telnet_framer.rs: prompt_with_ga | covered | pass |
| A prompt with EOR | crates/mu-core/tests/telnet_framer.rs: prompt_with_eor | covered | pass |
| A CR or a NUL alone | crates/mu-core/tests/telnet_framer.rs: cr_or_nul_alone_gives_no_prompt | covered | pass |
| A line split across reads | crates/mu-core/tests/telnet_framer.rs: line_split_across_reads | covered | pass |
| A read that ends at a line end | crates/mu-core/tests/telnet_framer.rs: read_that_ends_at_a_line_end | covered | pass |
| GA after a line end | crates/mu-core/tests/telnet_framer.rs: ga_after_a_line_end | covered | pass |
| Negotiation alone | crates/mu-core/tests/telnet_framer.rs: negotiation_alone | covered | pass |
| The same prompt after a line | crates/mu-core/tests/telnet_framer.rs: same_prompt_after_a_line | covered | pass |
| Lines in any split | crates/mu-core/tests/telnet_framer.rs: telnet_framer_lines_ignore_read_splits | covered | pass |
| Two marks in a row | crates/mu-core/tests/telnet_framer.rs: two_marks_in_a_row | covered | pass |
| The answer to a mark | crates/mu-core/tests/telnet_framer.rs: answer_to_a_mark | covered | pass |
| A mark from the game | crates/mu-core/tests/telnet_parser.rs: mark_from_the_game | covered | pass |
| A mark from the game with a policy that accepts it | crates/mu-core/tests/telnet_parser.rs: mark_refused_whatever_the_policy | covered | pass |
| A GMCP message | crates/mu-core/tests/telnet_framer.rs: gmcp_message | covered | pass |
| Four requests | crates/mu-core/tests/telnet_framer.rs: four_ttype_requests | covered | pass |
| A request while TTYPE is off | crates/mu-core/tests/telnet_framer.rs: ttype_request_while_ttype_is_off | covered | pass |
| NAWS turns on | crates/mu-core/tests/telnet_framer.rs: naws_turns_on | covered | pass |
| A size with the byte 255 | crates/mu-core/tests/telnet_framer.rs: size_with_byte_255 | covered | pass |
| The character set is offered | crates/mu-core/tests/telnet_framer.rs: charset_is_offered | covered | pass |
| A request with the TTABLE prefix | crates/mu-core/tests/telnet_framer.rs: charset_request_with_ttable | covered | pass |
| The character set is not offered | crates/mu-core/tests/telnet_framer.rs: charset_is_not_offered | covered | pass |
| A TTABLE prefix with no version | crates/mu-core/tests/telnet_framer.rs: ttable_prefix_with_no_version | covered | pass |
| An empty list | crates/mu-core/tests/telnet_framer.rs: empty_charset_list | covered | pass |
| Any payload | crates/mu-core/tests/telnet_framer.rs: framer_survives_any_payload | covered | pass |
| A subnegotiation over the bound | crates/mu-core/tests/telnet_framer.rs: subnegotiation_over_the_bound | covered | pass |
| A bad compressed stream | crates/mu-core/tests/telnet_framer.rs: bad_compressed_stream | covered | pass |

### Not run

- Random streams in random reads: Property tests, not run by this lens; the controller records the run.
- A line of exactly the bound: no verification run names this row

## Verification runs

- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=escaped_byte_in_data)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=byte_255_in_a_subnegotiation)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=line_to_the_game)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=ayt_before_data)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=each_two_byte_command)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=se_outside_a_subnegotiation)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=byte_that_is_no_command)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=accepted_option)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=refused_option_twice)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=repeated_request)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=crossing_requests)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=refusal_of_a_client_request)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=unrequested_answer_is_an_error)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=payload_with_escaped_byte)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=payload_at_the_bound)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=payload_over_the_bound)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=stream_starts_inside_a_read)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=plain_bytes_after_the_stream)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=start_inside_a_stream)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=second_stream_after_end)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=start_while_mccp2_is_off)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=stream_that_is_not_zlib)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=plain_bytes_after_the_answer_to_dont)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=bad_stream_that_breaks_inside_a_command)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=parser_matches_the_oracle)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=parser_ignores_read_splits)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_transcripts --locked -E 'test(=transcripts_in_other_reads)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=line_ends)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=bare_cr_in_a_line_to_the_game)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=compressed_lines)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=cr_nul_across_reads)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=long_run_with_no_line_end)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=line_of_exactly_the_bound)'`
- exit 0: `cargo nextest run -p mu-core --lib --locked -E 'test(=framer::tests::line_framer_line_of_exactly_the_bound)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=latin1_line_from_the_game)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=utf8_line_from_the_game)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=latin1_round_trip)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=latin1_line_to_the_game)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=prompt_with_no_ga)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=prompt_with_ga)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=prompt_with_eor)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=cr_or_nul_alone_gives_no_prompt)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=line_split_across_reads)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=read_that_ends_at_a_line_end)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=ga_after_a_line_end)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=negotiation_alone)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=same_prompt_after_a_line)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=telnet_framer_lines_ignore_read_splits)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=two_marks_in_a_row)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=answer_to_a_mark)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=mark_from_the_game)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_parser --locked -E 'test(=mark_refused_whatever_the_policy)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=gmcp_message)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=four_ttype_requests)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=ttype_request_while_ttype_is_off)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=naws_turns_on)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=size_with_byte_255)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=charset_is_offered)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=charset_request_with_ttable)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=charset_is_not_offered)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=ttable_prefix_with_no_version)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=empty_charset_list)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=framer_survives_any_payload)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=subnegotiation_over_the_bound)'`
- exit 0: `cargo nextest run -p mu-core --test telnet_framer --locked -E 'test(=bad_compressed_stream)'`

## Dropped

None.

## Lens notes

- approach: Follow-up round: no git access, so the changed lines were inferred from the commit subjects since the base and the fixed rows of prior.json (R-59, R-61, R-65, R-66, R-67).
- approach: src/telnet.rs and src/telnet/q.rs are not in diff-files.txt and were not read; their decisions are outside the bound.
- approach: The enforcement row 'The mark changes no option' names answer_to_a_mark (the framer test, which checks no option state); the parser test that checks it is answer_to_a_mark_changes_no_option. Not raised: the row may be outside the bound.
- approach: crates/mu-core/tests/telnet_parser.rs line 183 holds a doc comment left from the move of subnegotiation to support, now attached to payload_at_the_bound. Not an approach finding.
- fit: The fit lens has no Bash, so it could not list the changed lines since 9a60b39. It took the changed text from diff-files.txt, tasks 4.4 and the prior rows fixed in this round (R-59 to R-72).
- fit: crates/mu-core/tests/telnet_parser.rs line 183 keeps the doc comment of a removed helper, '/// The bytes of a GMCP subnegotiation of `payload`.', which now documents the test payload_at_the_bound.
- fit: For another lens: design.md 'The mark is outside the option table' says IAC DO TIMING-MARK goes through the option table, while Parser::negotiate answers it before the table (parser.rs line 380).
- invariants: I1: The parser refuses IAC DO TIMING-MARK each time, whatever its policy \| enforced at crates/mu-core/src/telnet/parser.rs:380-386 (early return before the Q method) \| boundary test mark_refused_whatever_the_policy (accepting policy, twice)
- invariants: I2: WILL and WONT TIMING-MARK both end a mark and change no option \| enforced at parser.rs:382 \| boundary test answer_to_a_mark (both verbs) and the oracle Part at telnet_parser.rs:508
- invariants: I3: DONT TIMING-MARK gives nothing \| enforced at parser.rs:384 \| boundary test none (inv-4)
- invariants: I4: After an inflate error the parse state is Data and no payload is open \| enforced at parser.rs:440-441, after the parse of the written bytes at 427 \| boundary test bad_stream_that_breaks_inside_a_command (states Iac and Sb, every two-part split). The reset is unconditional, so the untested states Negotiation, SbOption and SbIac take the same line; not raised.
- invariants: I5: A read that adds no byte to the partial line gives no prompt; a CR or a NUL after CR adds no byte \| enforced at framer.rs:317-320 and 388, 413 \| boundary test cr_or_nul_alone_gives_no_prompt
- invariants: I6: GA and EOR end a prompt \| enforced at framer.rs:389-392 \| boundary test prompt_with_ga, prompt_with_eor
- invariants: I7: Each CR of a line to the game goes as CR NUL and the line ends CR LF \| enforced at framer.rs:421-430 \| boundary test bare_cr_in_a_line_to_the_game (mid-line only; inv-2, inv-3)
- invariants: I8: The framer answers a TTYPE request only while TTYPE is on for the client \| enforced at framer.rs:342, checked after the whole read is parsed \| boundary test ttype_request_while_ttype_is_off (request alone; inv-1)
- invariants: I9: An empty CHARSET list, with or without [TTABLE ] and a version byte, gives REJECTED \| enforced at framer.rs:358-370 (split_first on an empty slice) \| boundary test empty_charset_list
- invariants: I10: No symbolic link in a member directory resolves into spikes/, for a file or a directory \| enforced at crates/mu-cli/tests/workspace_boundary.rs:310-334; files_under does not follow a symlinked directory because DirEntry::file_type does not follow links \| boundary test spike_link_fails, spike_directory_link_fails, link_outside_spikes_passes
- invariants: I11: The framer's copy of the policy and the parser's policy never diverge \| enforced at framer.rs:303 (clone at construction; the framer never reads settings.policy again) \| boundary test none needed
- invariants: Not raised: Parser::request is public and does not exclude TIMING-MARK, so request(Side::Remote, TIMING_MARK, true) would leave the remote Q at WantYes, since WILL TIMING-MARK returns early as a mark. No caller does this; parser.rs:228 is outside the follow-up bound.
- invariants: No enforcement point in the changed text is a script, so no probe was run.
- invariants: Follow-up round: only text changed since 9a60b39 was reviewed; unchanged requirements and decisions are outside the bound.
- test-honesty: two_marks_in_a_row now compares with a literal IAC DO TIMING-MARK, so a wrong MARK constant fails (R-69 fixed).
- test-honesty: latin1_round_trip checks bytes 128 to 159 only by round trip; a symmetric encode/decode bug would pass, but the spec scenario asks for a round trip and literal cases cover byte 233.
- test-coverage gap: Property-test generators beyond their names (oracle parts) were not read line by line.
- test-coverage gap: Test results were not run, per lens rules.

## Ledger

| id | first | last | severity | scope | location | disposition | basis | override | claim |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 1 | blocker | in-change | ["artifact",["specs/telnet-protocol/spec.md","Scenario: A line with the byte 255 to the game"],"**WHEN** the telnet framer encodes a line that holds the byte 255"] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The scenario 'A line with the byte 255 to the game' cannot happen: Framer::encode takes a &str, valid UTF-8 never holds the byte 0xFF, and the design keeps the Framer trait unchanged. |
| R-2 | 1 | 1 | blocker | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: The engine reads IAC IAC as one data byte"],"When it sends data, it SHALL double each byte 255."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The requirement that the engine doubles each byte 255 when it sends data has no decision: the engine API names no method that sends data, only send_subnegotiation doubles 255, and no decision says how TelnetFramer::encode escapes. |
| R-3 | 1 | 1 | blocker | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: Options are negotiated with the Q method"],"SHALL NOT answer a request that does not change the state of the option."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The requirement forbids an answer to a request that does not change the option state, but the design answers such requests: a refused WILL or DO in state No gets DONT or WONT, and DO TIMING-MARK gets WONT each time with no state change. |
| R-4 | 1 | 1 | blocker | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: The telnet framer gives prompts"],"At the end of each read, the telnet framer SHALL give the text with no line end as a prompt."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The prompt requirement gives the partial text as a prompt at the end of each read, but the design suppresses the prompt when the text is the same as at the last prompt, and task 4.2 tests that suppression; no requirement holds the exception. |
| R-5 | 1 | 1 | blocker | in-change | ["artifact",["specs/telnet-protocol/spec.md","Scenario: A stream that starts inside a read"],"**GIVEN** an engine that accepts MCCP2"] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The scenario 'A stream that starts inside a read' gives only a policy that accepts MCCP2 and no IAC WILL MCCP2, but the design never says what starts compression; the spike starts it only when MCCP2 is on for the game, which would give a subnegotiation instead. |
| R-6 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| The codes of telnet live only in `src/telnet.rs` \| The test `source_of_mu_core_keeps_its_rules`. \|"] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The design says the source test enforces that every telnet code lives only in src/telnet.rs, but the test finds only 251 to 255 and 6; the new codes SB 250, GA 249, SE 240, EOR 239, NOP 241 and the option codes such as GMCP 201 pass it unseen. |
| R-7 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The answers of a MU client"],"The review of pull request #32 found that the spike stripped the prefix with no space. This change fixes the defect and tests it."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The design says this change fixes the TTABLE defect of the spike, but at head the spike already strips '[TTABLE ]' with its space, and the decision page records the fixing commit. |
| R-8 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","4. The telnet framer"],"Make sure that `mise run docs:build` passes, or the docs task that `mise tasks` lists."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | app-8: Task 4.3 names the mise task docs:build, which does not exist; mise.toml defines docs:check for the documentation build.; spe-1: Task 4.3 names the build target `mise run docs:build`, which mise.toml does not define. |
| R-9 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The line buffer is shared by both framers"],"The partial line, the LF split, the drop of CR and the cut at `MAX_LINE` move into one crate-private type, `LineBuffer`, in `framer.rs`."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | After the change, mu-core reads IAC IAC and the mark answer in two parsers: the Held state machine of LineFramer and the new telnet engine. The design shares only the line split and does not name the second parser or a reason to keep it. |
| R-10 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","2. The engine"],"Add the property tests `engine_matches_the_oracle` and `engine_ignores_read_splits`, for the scenario \\"Random streams in random reads\\""] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | Three new test files each need a helper that feeds a byte stream in reads that end at given points. The plan names no shared helper, so each file will hold a copy of feed_split from framer.rs. |
| R-11 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The mark is outside the option table"],"`TelnetFramer::mark` gives `IAC DO TIMING-MARK` each time, with no call to the engine."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | TelnetFramer::mark builds the bytes IAC DO TIMING-MARK again. LineFramer::mark already builds them, and the fake game tests hold a third copy as a constant. |
| R-12 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Test data and the oracle"],"The six transcripts go to `crates/mu-core/tests/data/telnet/`."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The transcripts go to a new tests/data directory. The only test data directory in the repository today is tests/fixtures, and the design names neither that location nor a reason to differ from it. |
| R-13 | 1 | 2 | should-fix | in-change | ["artifact",["design.md","The engine lives under `telnet.rs`, and the codes stay in it"],"It keeps every code of telnet: the commands IAC, DONT, DO, WONT, WILL, SB, GA, EOR, NOP, SE and the other two-byte commands"] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The design puts every telnet code in src/telnet.rs, but the source test only finds 251 to 255 and 6. SB 250, GA 249, SE 240, EOR 239 and each option code can be written as literals in the submodules and tests and still pass. |
| R-14 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: Two-byte commands are two bytes"],"The engine SHALL read each telnet command other than SB and the four negotiation verbs as two bytes, `IAC` and the command."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The two-byte command test covers NOP through EOR. No test covers IAC SE outside a subnegotiation or IAC followed by a byte that is no command, and the spec does not say what those give. These are the edges of the rule that the byte after the command is data. |
| R-15 | 1 | 2 | blocker | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: The split of reads changes no event"],"the engine SHALL give the same events and send the same bytes for any split of the same input into reads."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The split invariant covers any input, but the oracle generator builds only valid zlib, so no test splits a bad compressed stream. Inflate can fail at a different byte for each split, and the engine's state after the failure decides whether later reads give more errors or data. |
| R-16 | 1 | 1 | should-fix | in-change | ["artifact",["tasks.md","1. Documentation and dependencies"],"Add `miniz_oxide` to the dependencies of `mu-core`, and `hex` to its dev-dependencies."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | Task 1.2 adds miniz_oxide and hex to mu-core, but no task adds them to ALLOWED_CRATES. The boundary test fails for any crate of mu-core that is not on that list, and the enforcement table names only cargo deny for a new crate. |
| R-17 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"Review covers a run-time read of a spike file."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The rule that no crate file reads a spike file relies on review alone for a read at run time. The transcript test reads its files at run time, and the boundary test checks only include macros and path attributes. |
| R-18 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: A subnegotiation reaches the caller"],"It SHALL drop a payload longer than 1 MiB and give a protocol error."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The 1 MiB payload bound is tested only one byte past the limit, in one read. No test sends a payload of exactly 1 MiB, which must pass, and no test splits an oversized payload across reads. The spec also does not say whether an escaped IAC IAC counts as one byte or two. |
| R-19 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: Options are negotiated with the Q method"],"The engine SHALL refuse each other option, and SHALL NOT answer a request that does not change the state of the option."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | For a refused option, the spec says the engine never answers a request that leaves the state unchanged, but the design and task 2.2 answer a refused DO TIMING-MARK each time. No test sends a refused WILL twice for an ordinary option, so the two statements are never compared. |
| R-20 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","Prompts"],"If the partial line is empty or the same as at the last prompt, it gives no prompt."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The prompt rule skips a partial line that equals the last prompt, and the design does not reset that memory at LF, GA, EOR or a cut. So a MUSH prompt such as '> ' sent again after a line would never show. No planned test sends the same prompt twice with a line between them. |
| R-21 | 1 | 1 | should-fix | in-change | ["artifact",["design.md","The line buffer is shared by both framers"],"`TelnetFramer` also drops a NUL right after a CR, because RFC 854 sends a bare CR as `CR NUL`."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | inv-7: The framer drops a NUL right after a CR, but CR is also dropped, so the framer has to remember a CR from the end of one read. The only planned case is cr_nul_is_a_bare_cr, and nothing names a CR at the end of a read with the NUL at the start of the next.; app-9: The decision to drop a NUL right after a CR has no requirement or scenario; the line requirement names only the drop of CR, yet task 4.2 adds the test cr_nul_is_a_bare_cr. |
| R-22 | 1 | 1 | should-fix | in-change | ["path","crates/mu-core/src/framer.rs",146] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The line cut fires when the partial line reaches MAX_LINE, so a line of exactly MAX_LINE bytes and then LF gives that line and then an extra empty line. The LineBuffer move and the telnet framer scenario test only MAX_LINE plus three, not the equal case. |
| R-23 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: The telnet framer sends the window size"],"The size is two 16-bit numbers in network order."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | NAWS sends the window size through send_subnegotiation, which doubles each byte 255. The only planned test uses 120 by 40, so no test covers a size with a byte 255, such as 255 columns or 65535 rows. |
| R-24 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Scenario: A stream that is not zlib"],"the bytes after the start of compression are not a valid zlib stream"] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The WHEN of the scenario 'A stream that is not zlib' names no concrete bytes, only a class of input. |
| R-25 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Scenario: A bad compressed stream"],"the telnet framer reads the start of MCCP2, bytes that are not zlib, and then more bytes"] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The WHEN of the scenario 'A bad compressed stream' names no concrete bytes after the MCCP2 start. |
| R-26 | 1 | 1 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Scenario: The RFC 1143 cases"],"the replies and the final state of the option match the table, and no reply loops"] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The THEN of 'The RFC 1143 cases' ends in 'no reply loops', a quality with no stated bound, and points at a table the change does not state. |
| R-27 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","The answers of a MU client"],"`TelnetFramer::new` takes the character set to accept and the window size."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | TelnetFramer::new takes the character set and the window size as separate parameters. The proposal says world-sessions (#54) passes the server-family quirks of world-config (#53) to the framer, so that change must widen this signature. |
| R-28 | 1 | 1 | suggestion | in-change | ["artifact",["design.md","The engine API"],"`Engine::send_subnegotiation` queues a subnegotiation, with each byte 255 of the payload doubled."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | Data and subnegotiation payloads go through an unbounded codec round trip: send_subnegotiation and encode double 255, and the engine reads IAC IAC back as 255. No property test sends the engine's output to a second engine. |
| R-29 | 1 | 1 | suggestion | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: The telnet framer answers a CHARSET request"],"It SHALL read an optional `[TTABLE ]` prefix and its version byte before the separator."] | fixed | 01e2022fb5a1ff472f8c5b0f7c858836e9b64173 |  | The framer parses game payloads for CHARSET (prefix, version byte, separator) and TTYPE. No planned test feeds the framer arbitrary subnegotiation payloads to show that it never panics, for example on a truncated [TTABLE ] with no version byte or an empty list. |
| R-30 | 2 | 2 | blocker | in-change | ["artifact",["design.md","Prompts"],"Take a read that added a data byte to the partial line. At the end of its `feed`, `TelnetFramer` gives the partial line as a prompt."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The prompt rule gives the partial line as a prompt at the end of any read that added a data byte, with no exception for an empty partial line. A read that ends at LF, such as 'lo\\n', then gives an empty prompt, and the session records every prompt as an event, so the THEN 'the prompt hel and then the line hello' gets an extra empty prompt. |
| R-31 | 2 | 2 | blocker | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: A subnegotiation reaches the caller"],"It SHALL drop a payload longer than 1 MiB and give a protocol error."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | No decision produces the 1 MiB bound of a subnegotiation: design.md never names the bound, how an escaped IAC IAC counts toward it, that the engine reads on to IAC SE and then resumes data, or a protocol error that names the option. The spike holds such a bound, but the design cites only its API shape. |
| R-32 | 2 | 2 | should-fix | deviates-design | ["artifact",["design.md","The engine lives under `telnet.rs`, and the codes stay in it"],"The source test adds the commands from 239 to 250 to its list `CODES`."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The design says the source test only gains the codes 239 to 250. At head, the test bytes_that_are_not_codes_pass asserts that 'let x = 250;' gives no hit, so adding 250 to CODES fails that test, and task 2.1's check of source_rules cannot pass without another edit to the test. |
| R-33 | 2 | 2 | should-fix | deviates-design | ["artifact",["design.md","The engine API"],"`Engine::send_data` queues data, and `Engine::send_subnegotiation` queues a subnegotiation. Both double each byte 255 with `telnet::escape`."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The design has two mechanisms for data to the game: Engine::send_data and TelnetFramer::encode, which escapes with telnet::escape outside the engine. Framer::encode takes &self, so the framer can never call send_data; send_data has no caller in the product, only the tests byte_255_in_data_to_the_game and engine_round_trip. The spike sent its lines through send_data. |
| R-34 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","2. The engine"],"Add `src/telnet/engine.rs` with `Engine`, `Policy`, `Side`, `Event` and `ProtocolError`, as the design describes, with no MCCP2 yet."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | Task 2.2 adds a public telnet::engine::Side for the two ends of the connection, while framer.rs already holds a private Side with Client and Game for the same two ends. TelnetFramer goes into framer.rs and must read engine option events by side, so one file sees two different Side types. The design names neither the existing enum nor a reason for a second one. |
| R-35 | 2 | 4 | should-fix | deviates-design | ["artifact",["design.md","Test data and test helpers"],"The test `transcript_files_live_in_the_crate` makes sure that the canonical path of each file is under `crates/mu-core/tests/fixtures/`."] | fixed | a10c67c9b78eecd20a03908064b53552fd62354e |  | Every rule that keeps crates away from spikes/ lives in the boundary test of mu-cli, as a Violation variant that covers each crate. The new spike rule, a link into spikes/, goes into a mu-core test that checks only the six transcript files. The design names neither the boundary test as the home of the rule nor a reason to differ, and the Contributing page still lists a symbolic link as a case for review. |
| R-36 | 2 | 2 | should-fix | beyond-scope | ["artifact",["tasks.md","2. The engine"],"Add the commands from 239 to 250 to `CODES` in `crates/mu-cli/tests/source_rules.rs`, and add a case for one of them to `telnet_code_is_found_in_each_form`."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The existing test bytes_that_are_not_codes_pass asserts that the literal 250 is no code. Task 2.1 adds 250 (SB) to CODES, so that edge case now sits inside the range and the test fails; no task moves it. |
| R-37 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","2. The engine"],"add a case for one of them to `telnet_code_is_found_in_each_form`"] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The range of command codes grows to 239..255, but task 2.1 adds a found-case for only one of the new codes. No case passes the lowest code 239 as a hit, and none passes 238 as a non-hit. |
| R-38 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","The line buffer is shared by both framers"],"In `LineBuffer`, a full partial line followed by LF is one line. A full partial line followed by any other byte is cut."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The new cut rule cuts a full partial line on 'any other byte'. If CR counts, a line of exactly MAX_LINE bytes ended by CR LF, the usual telnet line end, still gives the extra empty line. The only scenario of the bound ends with a bare LF. |
| R-39 | 2 | 2 | should-fix | in-change | ["artifact",["tasks.md","2. The engine"],"It generates random streams from parts, with the events that each part must give. An optional last part is a stream with a bad zlib header."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The split invariant covers any input, but the split property test draws only well-formed parts plus a bad zlib tail. No test splits malformed framing such as IAC SB IAC SE, IAC followed by WILL inside a subnegotiation, or IAC as the last byte of the stream. |
| R-40 | 2 | 2 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: MCCP2 is inflated inside the engine"],"When MCCP2 is on for the game, `IAC SB MCCP2 IAC SE` SHALL start compression, and the engine SHALL inflate the bytes that follow."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | Nothing states what IAC SB MCCP2 IAC SE does inside a compressed stream. The spec rule starts compression whenever MCCP2 is on, which reads as a nested inflate. The spike guarded this; the plan has no rule, scenario or oracle part for it. |
| R-41 | 2 | 2 | should-fix | in-change | ["artifact",["design.md","Test data and test helpers"],"makes sure that the canonical path of each file is under `crates/mu-core/tests/fixtures/`. So a symbolic link into `spikes/` fails it."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | transcript_files_live_in_the_crate enforces that no transcript is read from spikes/, but no test shows it fails on a symbolic link into spikes/. The repository proves its other rule scans RED with probe cases. |
| R-42 | 2 | 2 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: A bad compressed stream ends the reading"],"A compressed stream that is not valid zlib SHALL give one protocol error."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The requirement 'A bad compressed stream ends the reading' shares no two distinctive words with any task line. Task 2.3 names only its scenario 'A stream that is not zlib' by test name. |
| R-43 | 2 | 2 | suggestion | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The output pane decides how to show a prompt that a line replaced."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | The risk of a false prompt for a line split across reads leaves its display to an output pane. No task names an output pane, and no client crate handles prompt events today; only the server converts the kind. |
| R-44 | 2 | 2 | suggestion | in-change | ["artifact",["design.md","The answers of a MU client"],"World-sessions adds the quirks of each world type to this struct, so the signature of `new` does not change."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | ResolvedWorld is the resolved configuration of a world that the sessions read, and world-config (#53) merges the configuration layers into it. The design makes TelnetSettings the home of the world quirks such as the character set, so world-config and world-sessions must either add a TelnetSettings field to ResolvedWorld or keep the quirks in two structs. The design does not name ResolvedWorld. |
| R-45 | 2 | 2 | suggestion | in-change | ["artifact",["design.md","The character set of the text"],"ISO-8859-1 maps each byte to the character with the same number, which `char::from(u8)` does in the standard library."] | fixed | c0af2f89c1f1a9335276f95fcd1b6a5ed3dd7d85 |  | ISO-8859-1 decode and encode are a round trip over every byte other than CR, LF and 255 escaping, an unbounded domain. The three example tests cover a few bytes and none of 128 to 159, where windows-1252 differs. |
| R-46 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The engine lives under `telnet.rs`, and the codes stay in it"],"A literal right after the name `WILL`, `WONT`, `DO`, `DONT` or `SB` in a token list is an option code, as in `[IAC, WILL, 201]`."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | The new option-code rule flags any literal right after WILL in a token list. At head, three existing LineFramer tests in framer.rs write [IAC, WILL, 5], so source_of_mu_core_keeps_its_rules fails after task 2.1. But the design says the existing LineFramer tests keep their results, and task 4.1 asks for every line_framer_* test unchanged. |
| R-47 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The line buffer is shared by both framers"],"A CR and a NUL after a CR are dropped before the check, so they never start a cut."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | app-2: The design puts the drop of a NUL after CR in the shared LineBuffer, which LineFramer also feeds, and names no switch that keeps the NUL for LineFramer. The existing property test line_framer_marks_are_transparent expects the NUL after a CR to stay in the line. So the claim that the existing LineFramer tests keep their results fails if LineBuffer is built as written.; inv-1: The design drops a NUL after a CR inside the shared LineBuffer, so LineFramer would drop it too; the existing property test line_framer_marks_are_transparent expects LineFramer to keep that NUL, and task 4.1 says every line_framer_* test stays unchanged. |
| R-48 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The bound of a subnegotiation"],"When a payload reaches the bound, the engine keeps no more bytes and marks the payload as too long."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | app-3: The design marks a payload as too long when it reaches the bound, and it calls this the rule of the spike. The spike sets the overflow only when a byte arrives after the buffer is full. Read as written, a payload of exactly 1 MiB is marked too long, which contradicts the scenario 'A payload at the bound'.; inv-2: The design marks a payload too long when it reaches the bound, so a payload of exactly 1 MiB is dropped; the spec requires that payload to pass, and the spike marks it only when a byte arrives past the bound. |
| R-49 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Test data and test helpers"],"Each of these gives the events that the engine rules state."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | The oracle relies on 'the events that the engine rules state' for each malformed part: IAC SB IAC SE, a negotiation verb inside a subnegotiation, and IAC as the last byte. No requirement or decision states those events. The only ProtocolError variant that the design names is SubnegotiationTooLong, besides the inflate error. The spike gives EmptySubnegotiation and UnterminatedSubnegotiation errors here, but the design names neither. |
| R-50 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","The engine API"],"The engine has no step that sends data. `Framer::encode` takes `&self`, so `TelnetFramer::encode` calls `telnet::escape` itself."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | In mu-core, 'the engine' already names the server engine: the crate doc, Portal and the player-visible drop texts of the session use it. The design adds telnet::Engine and calls it 'the engine' throughout design and spec, and names neither the existing term nor a reason to reuse it. |
| R-51 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Test data and test helpers"],"So it gains one more: a new `Violation::SpikeLink` for any symbolic link under `crates/` whose target resolves into `spikes/`."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | Violation::SpikeLink walks the path crates/, while the existing spike rules of the boundary test walk the member directories from cargo metadata and compare with metadata.workspace_root.join("spikes"). The design names neither that wiring nor a reason for a hard-coded root. |
| R-52 | 3 | 3 | should-fix | in-change | ["artifact",["tasks.md","3. The transcripts"],"Add the probe test `spike_link_is_found`, which makes such a link in a `TreeCopy` of the fixture."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | Each probe test of a violation in workspace_boundary.rs is named <case>_fails, for example spike_include_fails and spike_dependency_fails. The new probe is named spike_link_is_found, with no reason given. |
| R-53 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Enforcement of each rule"],"\| No empty prompt \| The test `read_that_ends_at_a_line_end`. \|"] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | The design claims no empty prompt, but GA and EOR give the partial line as a prompt with no check that it is empty, so a line, LF and IAC GA gives an empty prompt; the only test of the rule is a read that ends at LF. |
| R-54 | 3 | 3 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: The split of reads changes no event"],"One exception holds: a compressed stream that fails after its first byte can give a different amount of data before its error."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | The split exception allows only a different amount of data before a mid-stream inflate error, but the inflated bytes go through the telnet parser, so a negotiation or command before the break can also differ by split, with different replies sent; no test puts telnet bytes before the break. |
| R-55 | 3 | 3 | should-fix | in-change | ["artifact",["design.md","Test data and test helpers"],"Malformed framing is a part too: `IAC SB IAC SE`, a negotiation verb inside a subnegotiation, and an `IAC` as the last byte of the stream."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | The oracle must give the events that the engine rules state for IAC SB IAC SE, a negotiation verb inside a subnegotiation and a trailing IAC, but no requirement or decision states those events; the spike gives EmptySubnegotiation and UnterminatedSubnegotiation errors that the plan never names. |
| R-56 | 3 | 3 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Requirement: MCCP2 is inflated inside the engine"],"When MCCP2 is off, or a stream is already open, that subnegotiation SHALL start nothing."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | A stream opens only when none is open, and the tests cover only the open side and MCCP2 off; no named test starts a second stream right after a stream ends, which checks that the end clears the open state. |
| R-57 | 3 | 3 | suggestion | in-change | ["artifact",["design.md","Risks / Trade-offs"],"The error message tells the player. World-sessions adds the disconnect."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | Every protocol error reaches the session as a Message named telnet with free text, and the session records it as text. World-sessions (#54) must disconnect on a bad MCCP2 stream but not on an oversized subnegotiation, so it must either parse that text or add the Inbound::Error that this design rejects. |
| R-58 | 3 | 3 | suggestion | in-change | ["artifact",["design.md","The answers of a MU client"],"World-config (#53) adds the character set to it, and world-sessions builds `TelnetSettings` from it for each connection."] | fixed | f24817a5ad177d3d9010f79a1b624dc6331aeae2 |  | The world-type-profiles decision makes the telnet options a quirk field of a built-in type, kept out of the connection code. The design fixes the accept policy inside TelnetFramer and builds TelnetSettings from the character set alone, so world-config (#53) and world-sessions (#54) must move the policy out of the framer into TelnetSettings. |
| R-59 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/telnet/parser.rs",434] | fixed | 72eb7eca0bf94097207e9ff13080f4818c61948b |  | app-1: The design reads the bytes after IAC WONT MCCP2 as plain telnet, but the inflate error branch keeps the parse state and the open payload of the inflated bytes. If the inflated bytes before the break end inside an IAC or a subnegotiation, the bytes after the answer become a command or payload. That state depends on how much inflate gave out, which the design says differs by split, so the events after the error can differ by split, against the requirement.; inv-1: After a bad compressed stream the parser keeps the State that the garbage inflated bytes left (Iac, Sb, SbIac). recover() calls negotiate() directly, so the bytes after IAC WONT MCCP2 are not read as plain telnet: a trailing IAC makes the first plain byte a command, and an open IAC SB swallows plain data into a payload. |
| R-60 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Goals / Non-Goals"],"This change adds codes to its list and changes nothing else in it."] | fixed | 90947402876dd80ac6794d8d49b9899902029905 |  | The non-goal says the change adds codes to the source test and changes nothing else in it. At head the source test also has a new rule, Rule::OptionCode with OPTION_VERBS, which the decision on telnet.rs asks for, so the non-goal is not so. |
| R-61 | 4 | 4 | should-fix | in-change | ["artifact",["design.md","Checked facts"],"After an inflate error, the spike reads later bytes as plain telnet at once. This design drops them until `WONT MCCP2`."] | fixed | 90947402876dd80ac6794d8d49b9899902029905 |  | The checked fact says the spike reads later bytes as plain telnet at once after an inflate error. The spike returns an empty rest and its own doc says the rest of the read is lost, so it drops the rest of that read first. |
| R-62 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/tests/telnet_parser.rs",23] | fixed | 72eb7eca0bf94097207e9ff13080f4818c61948b |  | The change writes the join of adjacent Data events three times: in drain and join of telnet_parser.rs, and in events of telnet_transcripts.rs. The spec defines the split rule on joined events, so the three copies are one definition. |
| R-63 | 4 | 5 | should-fix | in-change | ["path","crates/mu-core/tests/telnet_parser.rs",202] |  | regressed from fixed 72eb7eca0bf94097207e9ff13080f4818c61948b |  | The tests build the bytes IAC SB option, the escaped payload and IAC SE three times, in two files; the parser test copy fixes the option to GMCP and the oracle part writes it inline. |
| R-64 | 4 | 4 | should-fix | in-change | ["path","docs/src/content/docs/contributing/workspace.md",35] | fixed | a10c67c9b78eecd20a03908064b53552fd62354e |  | The boundary test section of the workspace Contributing page lists each case that the test fails, with the spike dependency and the spike include, but not the new symbolic link into spikes/. |
| R-65 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/framer.rs",389] | fixed | 4c747651088a26e327aeb12a9eff9795b46108df |  | The framer sets `added` for any Data event, including one whose only bytes are a dropped CR or a dropped NUL after CR. Such a read adds no byte to the unfinished line, yet the framer gives the same prompt again: `Name: ` then a read of `\\r` gives the prompt `Name: ` twice. |
| R-66 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/telnet/parser.rs",380] | fixed | 72eb7eca0bf94097207e9ff13080f4818c61948b |  | The rule that the parser refuses DO TIMING-MARK each time holds only because the default policy does not accept TIMING-MARK on the client side. The policy is a public field of TelnetSettings that world-type profiles are meant to change; a policy that accepts it makes the parser answer WILL TIMING-MARK once and then never again. |
| R-67 | 4 | 4 | should-fix | in-change | ["path","crates/mu-cli/tests/workspace_boundary.rs",692] | fixed | a10c67c9b78eecd20a03908064b53552fd62354e |  | Violation::SpikeLink is probed only with a link to one file. The likely real case, a link to a whole directory such as tests/fixtures/telnet pointing at spikes/telnet-engine/transcripts, takes another path through files_under (a symlinked directory is not descended) and no probe test covers it. |
| R-68 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/src/framer.rs",359] | fixed | 4c747651088a26e327aeb12a9eff9795b46108df |  | The design states that an empty CHARSET list gives REJECTED, but no test sends a REQUEST with no list or `[TTABLE ]`, a version byte and no list. framer_survives_any_payload draws random bytes and almost never forms the TTABLE prefix, and it asserts only that the framer returns. |
| R-69 | 4 | 4 | should-fix | in-change | ["path","crates/mu-core/tests/telnet_framer.rs",186] | fixed | 4c747651088a26e327aeb12a9eff9795b46108df |  | two_marks_in_a_row compares TelnetFramer::mark() with the production constant MARK that mark() returns, so a wrong MARK would still pass. |
| R-70 | 4 | 4 | suggestion | in-change | ["path","crates/mu-core/src/telnet/parser.rs",115] | refuted | A generic Sides<T> would drop the const fn of Policy::accepts and add a type to replace four two-arm matches over fixed arrays; the copies cannot drift apart in a way a test would miss. |  | Policy and Parser each keep a local and a remote table of 256 entries and pick one with a match on Side in four methods: accept, accepts, q and set_q. The blocks vary only in the element type. |
| R-71 | 4 | 4 | suggestion | in-change | ["path","crates/mu-core/src/framer.rs",307] | fixed | 4c747651088a26e327aeb12a9eff9795b46108df |  | The design lets world-sessions add each world quirk to TelnetSettings so that new keeps its signature, but TelnetFramer copies charset, columns and rows out of the settings into its own fields. Each quirk that world-sessions adds then needs a new field and a new line in new as well. |
| R-72 | 4 | 4 | note | in-change | ["path","crates/mu-core/tests/telnet_parser.rs",175] | fixed | 90947402876dd80ac6794d8d49b9899902029905 |  | Two tests match no scenario and no task line: unrequested_answer_is_an_error and compressed_lines. |
| R-73 | 4 | 5 | should-fix | in-change | ["artifact",["review","not-run"],"Random streams in random reads"] | refuted | The merge matches a run only to the whole covered_by text, and these rows name two tests each; every named test ran once at head and passed, as verification-runs.txt shows. |  | covered row 'Random streams in random reads' has no verification run |
| R-74 | 4 | 4 | should-fix | in-change | ["artifact",["review","not-run"],"The recorded transcripts in other reads"] | refuted | The merge matches a run only to the whole covered_by text, and these rows name two tests each; every named test ran once at head and passed, as verification-runs.txt shows. |  | covered row 'The recorded transcripts in other reads' has no verification run |
| R-75 | 4 | 5 | should-fix | in-change | ["artifact",["review","not-run"],"A line of exactly the bound"] | refuted | The merge matches a run only to the whole covered_by text, and these rows name two tests each; every named test ran once at head and passed, as verification-runs.txt shows. |  | covered row 'A line of exactly the bound' has no verification run |
| R-76 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/framer.rs",425] |  |  |  | app-1: TelnetFramer::encode now sends each CR of a line as CR NUL, as the spec asks, but both design decisions on encode still say it gives the escaped line in the character set and then CR LF, with no CR NUL step and no recorded deviation.; inv-2: Each CR of a line to the game goes as CR NUL, but the only test puts the CR in the middle of the line. No test has a CR as the last byte before the CR LF end, two adjacent CRs, or a CR next to an escaped 255.; inv-3: TelnetFramer::encode now combines two escapes, IAC doubling and CR NUL, over an unbounded domain of lines. No property test feeds encode's output back to a TelnetFramer; parser_round_trip uses telnet::escape alone and latin1_round_trip skips CR. |
| R-77 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/telnet/parser.rs",383] |  |  |  | app-2: The parser now answers IAC DO TIMING-MARK with WONT before the Q table, whatever the policy, but the decision still says DO TIMING-MARK goes through the table like a refused option; read as written, a policy that accepts TIMING-MARK would answer WILL, against the scenario with such a policy.; inv-4: The new TIMING-MARK branch of negotiate drops IAC DONT TIMING-MARK in its catch-all arm, where before it went through the Q method. WILL, WONT and DO each have a test; no test sends DONT TIMING-MARK to show it gives no reply, no event and no state change. |
| R-78 | 5 | 5 | should-fix | in-change | ["artifact",["design.md","Checked facts"],"After a decompression error, the MCCP specification lets the client send `DONT MCCP2` and go on as plain telnet. This design does that."] |  |  |  | The checked fact says that after a decompression error the client may send DONT MCCP2 and go on as plain telnet, and that this design does that. The design and the parser drop every byte until IAC WONT MCCP2 instead, so the design does not go on as plain telnet. |
| R-79 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/src/framer.rs",317] |  |  |  | TelnetFramer::data now drops each CR itself before it feeds LineBuffer, so the drop of CR has two homes: LineBuffer::push and TelnetFramer::data. The design names LineBuffer as the one home of the CR drop. |
| R-80 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/tests/telnet_transcripts.rs",16] |  |  |  | events() of the transcript test feeds each read to a parser, collects the events and joins them, which run_reads of the parser test already does. The fix of the join moved only join_data to support, so the read loop is still written twice. |
| R-81 | 5 | 5 | should-fix | in-change | ["path","crates/mu-core/tests/telnet_parser.rs",400] |  |  |  | The new test bad_stream_that_breaks_inside_a_command spells the answer IAC WONT MCCP2 inline, as plain_bytes_after_the_answer_to_dont does, while the same file holds the constant WONT_MCCP2 for these bytes. |
| R-82 | 5 | 5 | should-fix | in-change | ["artifact",["specs/telnet-protocol/spec.md","Scenario: A request while TTYPE is off"],"the game sends `IAC SB TTYPE SEND IAC SE` before `IAC DO TTYPE`"] |  |  |  | The TTYPE gate reads the option state after the parser has read the whole read, not at the request. A read holding the request and then IAC DO TTYPE gets an IS answer (the scenario's own case), and a request then IAC DONT TTYPE in one read gets none, so the reply also differs by read split. The only test sends the request alone. |
| R-83 | 5 | 5 | suggestion | in-change | ["path","crates/mu-core/tests/telnet_parser.rs",423] |  |  |  | mark_refused_whatever_the_policy repeats the body of mark_from_the_game: the same two DO TIMING-MARK and the same drained answer. Only the policy differs. |
| R-84 | 5 | 5 | suggestion | in-change | ["path","crates/mu-core/tests/telnet_parser.rs",405] |  |  |  | bad_stream_that_breaks_inside_a_command repeats the split loop of plain_bytes_after_the_answer_to_dont: each split point, two reads, the error text removed, the same compare. Only the bytes and the expected events differ. |
| R-85 | 5 | 5 | suggestion | in-change | ["path","crates/mu-cli/tests/workspace_boundary.rs",710] |  |  |  | spike_directory_link_fails repeats spike_link_fails line for line; only the target in spikes/ and the link name differ. The same file runs its other spike cases from one list, as spike_include_fails does. |

## Reviewed files

| path | content |
|---|---|
| Cargo.lock | 0162cf8e90df185a27791cb764c70ee3e4a83c8d |
| Cargo.toml | 74fbd1dd4a82cd1f7584f6615cc5916411533c89 |
| crates/mu-cli/tests/source_rules.rs | 007732a0bb6983f4c618e0a4ce95acfabdb99a3e |
| crates/mu-cli/tests/workspace_boundary.rs | c71559d5d4046ec318da33c952328e25403a030f |
| crates/mu-core/Cargo.toml | 30bbfcbc33810f003217395fd4f915d929091b0c |
| crates/mu-core/src/fake.rs | 89b05d706b669893bc05fd6d9bdf34d3279c6f96 |
| crates/mu-core/src/framer.rs | 55f14c10ac86aa915a15dc92e76a3677a747fa07 |
| crates/mu-core/src/lib.rs | 454db19132b1a75ba30144f15968729bed30e491 |
| crates/mu-core/src/telnet.rs | 976f5e5a7cbd5e9d03cf4705236b29f952f94b18 |
| crates/mu-core/src/telnet/parser.rs | da45b29e2cc890170a66d760591a36e4276c7280 |
| crates/mu-core/src/telnet/q.rs | 263cb16c3e9b697cdbb290cb613e8e072003e11a |
| crates/mu-core/tests/fixtures/telnet/aresmush.txt | 964175e4ae3b1fd9a5e4b30db98ca4db81158de0 |
| crates/mu-core/tests/fixtures/telnet/evennia.txt | c5adb927f65a63be4b19e69119654402f426e717 |
| crates/mu-core/tests/fixtures/telnet/pennmush.txt | 4f2af18a7f1eba313c109d25654f261b8ba17065 |
| crates/mu-core/tests/fixtures/telnet/rhostmush.txt | c08a46026f8cc102dab1ddf3add1020755ebd362 |
| crates/mu-core/tests/fixtures/telnet/tinymux-latin1.txt | 69832fbb9dc3fc02076a0fa507b32ff045ff6534 |
| crates/mu-core/tests/fixtures/telnet/tinymux-utf8.txt | 6dfb3ba11818d2432bc37da78252fb9183585eea |
| crates/mu-core/tests/support/mod.rs | c919980906431d651ea99a91685edbc52eff1156 |
| crates/mu-core/tests/telnet_framer.rs | 3245724464829c032e97a8e5f6caacaa170c4958 |
| crates/mu-core/tests/telnet_parser.rs | 0fad793669b8ad0020b7b741fa0a745562ecf41b |
| crates/mu-core/tests/telnet_transcripts.rs | 0c3fc5cd998ae22a70648118fd069766c3f27022 |
| docs/src/content/docs/architecture/server-and-clients.mdx | b5897a34d643b3797bf46467b7bbb1b80fd4375c |
| docs/src/content/docs/contributing/spikes.md | 694be560c8e0316d961546c89ee7846f2762c63d |
| docs/src/content/docs/contributing/workspace.md | a6e10785e0fb9fde5e96b7140543065d461ae5c3 |
| openspec/changes/telnet-engine/.openspec.yaml | cfbed6e8fdb007b75eb7f423f77f14d16ff7b4b4 |
| openspec/changes/telnet-engine/design.md | 8408f71d3724ef68946767ff5bff22fbbb395c33 |
| openspec/changes/telnet-engine/proposal.md | 28290906fc22826e08f117f6cc2d0e5fe808097a |
| openspec/changes/telnet-engine/specs/telnet-protocol/spec.md | 731f24ff02ce748b8fb821d2b0d659f225300f76 |
| openspec/changes/telnet-engine/tasks.md | sha256:df16cd4d2ac29ba66d3f32ced5241825e9767a6fa3742abcfc1f9ac33c721c41 |

## Plan review (2026-11-17)

Verdict approve-with-notes at head e75dc3405a6b87176b209c3f52cd660fef8f7073.

### Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge |
|---|---|---|---|---|---|---|---|---|
| R-46 | approach | should-fix | in-change | The new option-code rule flags any literal right after WILL in a token list. At head, three existing LineFramer tests in framer.rs write [IAC, WILL, 5], so source_of_mu_core_keeps_its_rules fails after task 2.1. But the design says the existing LineFramer tests keep their results, and task 4.1 asks for every line_framer_* test unchanged. | design.md > The engine lives under `telnet.rs`, and the codes stay in it — "A literal right after the name `WILL`, `WONT`, `DO`, `DONT` or `SB` in a token list is an option code, as in `[IAC, WILL" | In task 2.1, add an option constant (STATUS, 5) to telnet::opt and change the three literals in framer.rs to it. Then say in the design and in task 4.1 that these two tests change only that literal. |  |  |
| R-47 | approach,invariants | should-fix | in-change | app-2: The design puts the drop of a NUL after CR in the shared LineBuffer, which LineFramer also feeds, and names no switch that keeps the NUL for LineFramer. The existing property test line_framer_marks_are_transparent expects the NUL after a CR to stay in the line. So the claim that the existing LineFramer tests keep their results fails if LineBuffer is built as written.; inv-1: The design drops a NUL after a CR inside the shared LineBuffer, so LineFramer would drop it too; the existing property test line_framer_marks_are_transparent expects LineFramer to keep that NUL, and task 4.1 says every line_framer_* test stays unchanged. | design.md > The line buffer is shared by both framers — "A CR and a NUL after a CR are dropped before the check, so they never start a cut." | app-2: State that the NUL drop is a setting of LineBuffer that only TelnetFramer turns on. For LineFramer, LineBuffer keeps a NUL after a CR as data.; inv-1: Say in the design where the NUL drop lives: either a flag of LineBuffer that only TelnetFramer sets, or a step in TelnetFramer before LineBuffer. Add a LineFramer unit test that `a\\r\\0b\\n` keeps the NUL, so the shared buffer cannot change LineFramer. |  | severity? blocker 0.88 |
| R-48 | approach,invariants | should-fix | in-change | app-3: The design marks a payload as too long when it reaches the bound, and it calls this the rule of the spike. The spike sets the overflow only when a byte arrives after the buffer is full. Read as written, a payload of exactly 1 MiB is marked too long, which contradicts the scenario 'A payload at the bound'.; inv-2: The design marks a payload too long when it reaches the bound, so a payload of exactly 1 MiB is dropped; the spec requires that payload to pass, and the spike marks it only when a byte arrives past the bound. | design.md > The bound of a subnegotiation — "When a payload reaches the bound, the engine keeps no more bytes and marks the payload as too long." | app-3: Reword the decision: a payload byte that arrives when the payload already holds MAX_SUBNEGOTIATION bytes is dropped and marks the payload too long.; inv-2: Reword the decision: the engine marks the payload too long when a byte arrives while the payload already holds MAX_SUBNEGOTIATION bytes. Keep payload_at_the_bound as the test of that edge. |  |  |
| R-49 | approach | should-fix | in-change | The oracle relies on 'the events that the engine rules state' for each malformed part: IAC SB IAC SE, a negotiation verb inside a subnegotiation, and IAC as the last byte. No requirement or decision states those events. The only ProtocolError variant that the design names is SubnegotiationTooLong, besides the inflate error. The spike gives EmptySubnegotiation and UnterminatedSubnegotiation errors here, but the design names neither. | design.md > Test data and test helpers — "Each of these gives the events that the engine rules state." | Add a decision under 'The engine API' that names the events of each malformed part. Examples: the error for IAC SB IAC, the error and then the command for IAC and a byte other than SE or IAC inside a subnegotiation, and that a trailing IAC gives nothing until the next read. |  |  |
| R-50 | fit | should-fix | in-change | In mu-core, 'the engine' already names the server engine: the crate doc, Portal and the player-visible drop texts of the session use it. The design adds telnet::Engine and calls it 'the engine' throughout design and spec, and names neither the existing term nor a reason to reuse it. | design.md > The engine API — "The engine has no step that sends data. `Framer::encode` takes `&self`, so `TelnetFramer::encode` calls `telnet::escape`" | Name the clash in the design and say 'the telnet engine' in design, spec and docs, or give the type a name that does not reuse the crate's term for the server engine. |  |  |
| R-51 | fit | should-fix | in-change | Violation::SpikeLink walks the path crates/, while the existing spike rules of the boundary test walk the member directories from cargo metadata and compare with metadata.workspace_root.join("spikes"). The design names neither that wiring nor a reason for a hard-coded root. | design.md > Test data and test helpers — "So it gains one more: a new `Violation::SpikeLink` for any symbolic link under `crates/` whose target resolves into `spi" | State that SpikeLink walks the same member directories with support::files_under, as include_violations does, and compares the canonical target with the same spikes path. |  |  |
| R-52 | fit | should-fix | in-change | Each probe test of a violation in workspace_boundary.rs is named <case>_fails, for example spike_include_fails and spike_dependency_fails. The new probe is named spike_link_is_found, with no reason given. | tasks.md > 3. The transcripts — "Add the probe test `spike_link_is_found`, which makes such a link in a `TreeCopy` of the fixture." | Name the probe spike_link_fails in tasks.md and in the enforcement table of design.md. |  |  |
| R-53 | invariants | should-fix | in-change | The design claims no empty prompt, but GA and EOR give the partial line as a prompt with no check that it is empty, so a line, LF and IAC GA gives an empty prompt; the only test of the rule is a read that ends at LF. | design.md > Enforcement of each rule — "\| No empty prompt \| The test `read_that_ends_at_a_line_end`. \|" | Add 'if it is not empty' to the GA and EOR rule in the spec and the design, and a scenario and test `ga_after_a_line_end`: `ok\\n IAC GA` gives the line `ok` and no prompt, and `IAC GA IAC GA` alone gives nothing. |  |  |
| R-54 | invariants | should-fix | in-change | The split exception allows only a different amount of data before a mid-stream inflate error, but the inflated bytes go through the telnet parser, so a negotiation or command before the break can also differ by split, with different replies sent; no test puts telnet bytes before the break. | specs/telnet-protocol/spec.md > Requirement: The split of reads changes no event — "One exception holds: a compressed stream that fails after its first byte can give a different amount of data before its " | Widen the exception to any event and sent byte before the error of a stream that breaks after its header. Put an `IAC WILL GMCP` in the inflated bytes before the break in bad_stream_after_data, and assert only what holds for every split: one error, then silence. |  |  |
| R-55 | invariants | should-fix | in-change | The oracle must give the events that the engine rules state for IAC SB IAC SE, a negotiation verb inside a subnegotiation and a trailing IAC, but no requirement or decision states those events; the spike gives EmptySubnegotiation and UnterminatedSubnegotiation errors that the plan never names. | design.md > Test data and test helpers — "Malformed framing is a part too: `IAC SB IAC SE`, a negotiation verb inside a subnegotiation, and an `IAC` as the last b" | Add a requirement with one scenario for each malformed part, for example: `IAC SB IAC SE` gives a protocol error and then the command SE; `IAC SB GMCP a IAC WILL ECHO` gives a protocol error that names GMCP, no subnegotiation, and the negotiation. List the ProtocolError variants in 'The engine API', and add a unit test for each scenario in task 2.2. |  |  |
| R-56 | invariants | should-fix | in-change | A stream opens only when none is open, and the tests cover only the open side and MCCP2 off; no named test starts a second stream right after a stream ends, which checks that the end clears the open state. | specs/telnet-protocol/spec.md > Requirement: MCCP2 is inflated inside the engine — "When MCCP2 is off, or a stream is already open, that subnegotiation SHALL start nothing." | Add a scenario and test `second_stream_after_the_end`: one read with the start, a complete zlib stream of `a`, the start again and a stream of `b` gives start, `a`, end, start and `b`. Add it to the enforcement row. |  |  |
| R-57 | fit | suggestion | in-change | Every protocol error reaches the session as a Message named telnet with free text, and the session records it as text. World-sessions (#54) must disconnect on a bad MCCP2 stream but not on an oversized subnegotiation, so it must either parse that text or add the Inbound::Error that this design rejects. | design.md > Risks / Trade-offs — "The error message tells the player. World-sessions adds the disconnect." | In 'Messages and errors', state how world-sessions tells the fatal bad-stream error from SubnegotiationTooLong without parsing text, for example a distinct message name for each ProtocolError variant, so #54 neither parses text nor changes Inbound. |  | scope? deviates-design 0.67; severity? should-fix 0.63 |
| R-58 | fit | suggestion | in-change | The world-type-profiles decision makes the telnet options a quirk field of a built-in type, kept out of the connection code. The design fixes the accept policy inside TelnetFramer and builds TelnetSettings from the character set alone, so world-config (#53) and world-sessions (#54) must move the policy out of the framer into TelnetSettings. | design.md > The answers of a MU client — "World-config (#53) adds the character set to it, and world-sessions builds `TelnetSettings` from it for each connection." | Make the accept policy a field of TelnetSettings, with TelnetSettings::default() holding the transcript policy, and cite the world-type-profiles decision in 'The answers of a MU client'. |  | severity? should-fix 0.79 |

### Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| An escaped byte in data | 2.2 | covered |  |
| The byte 255 in a subnegotiation to the game | 2.2 | covered |  |
| A line to the game | 4.2 | covered |  |
| AYT before data | 2.2 | covered |  |
| Each two-byte command | 2.2 | covered |  |
| SE outside a subnegotiation | 2.2 | covered |  |
| A byte that is no command | 2.2 | covered |  |
| An accepted option | 2.2 | covered |  |
| A refused option, twice | 2.2 | covered |  |
| A repeated request | 2.2 | covered |  |
| A request of the client that crosses a request of the game | 2.2 | covered |  |
| A refusal of a request of the client | 2.2 | covered |  |
| A payload with an escaped byte | 2.2 | covered |  |
| A payload at the bound | 2.2 | covered |  |
| A payload over the bound | 2.2 | covered |  |
| A stream that starts inside a read | 2.3 | covered |  |
| Plain bytes after the stream | 2.3 | covered |  |
| A start inside a stream | 2.3 | covered |  |
| A start while MCCP2 is off | 2.3 | covered |  |
| A stream that is not zlib | 2.3 | covered |  |
| Random streams in random reads | 2.4 | covered |  |
| The recorded transcripts in other reads | 3.1 | covered |  |
| Line ends | 4.2 | covered |  |
| CR NUL across reads | 4.2 | covered |  |
| A long run with no line end | 4.2 | covered |  |
| A line of exactly the bound | 4.1, 4.2 | covered |  |
| A Latin-1 line from the game | 4.2 | covered |  |
| A UTF-8 line from the game | 4.2 | covered |  |
| Every Latin-1 byte | 4.2 | covered |  |
| A Latin-1 line to the game | 4.2 | covered |  |
| A prompt with no GA | 4.2 | covered |  |
| A prompt with GA | 4.2 | covered |  |
| A line split across reads | 4.2 | covered |  |
| A read that ends at a line end | 4.2 | covered |  |
| Negotiation alone | 4.2 | covered |  |
| The same prompt after a line | 4.2 | covered |  |
| Lines in any split | 4.2 | covered |  |
| Two marks in a row | 4.2 | covered |  |
| The answer to a mark | 4.2 | covered |  |
| A mark from the game | 2.2 | covered |  |
| A GMCP message | 4.2 | covered |  |
| Four requests | 4.2 | covered |  |
| NAWS turns on | 4.2 | covered |  |
| A size with the byte 255 | 4.2 | covered |  |
| The character set is offered | 4.2 | covered |  |
| A request with the TTABLE prefix | 4.2 | covered |  |
| The character set is not offered | 4.2 | covered |  |
| A TTABLE prefix with no version | 4.2 | covered |  |
| Any payload | 4.2 | covered |  |
| A subnegotiation over the bound | 4.2 | covered |  |
| A bad compressed stream | 4.2 | covered |  |

### Not run

Plan phase: no verification runs.
