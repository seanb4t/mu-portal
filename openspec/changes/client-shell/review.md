---
verdict: changes-requested
phase: plan
reviewed_at: 2026-11-22T22:04:51+00:00
head: c41b481932f355567bb67947a1f2746b0c929502
round: 2
next_id: 62
lenses:
  spec-lint: ok
  approach: ok
  fit: ok
  test-coverage: ok
  test-honesty: not-applicable
  invariants: ok
  verify: ok
---
# Review

## Findings

| id | lens | severity | scope | claim | evidence | action | mark | judge | consequence |
|---|---|---|---|---|---|---|---|---|---|
| R-1 | verify | blocker | in-change | PARTIAL: the fix a9555c243143a1800eea6daef564f374719bf20f of R-1 did not hold. app-1: /quit cannot confirm a quit while texts wait: the keys that type a second /quit reset quit_armed, and no decision says how a command confirms.; inv-1: /quit can never leave while a send waits: each typed key of a second /quit clears the quit arm, so the double quit of Ctrl+C never completes by command. | design.md > D4. Commands are parsed in `update` at a send — "A send, any other command, or a key that reaches another entry clears it." | fix it again | regression |  | unmet-requirement |
| R-44 | test-coverage | should-fix | in-change | delta scenario 'An argument to an entry that takes none' has no matrix row | review > absent-row — "An argument to an entry that takes none" | add the row with its covering task or test |  |  | unmet-requirement |
| R-45 | test-coverage | should-fix | in-change | delta scenario 'Connect with no world in focus' has no matrix row | review > absent-row — "Connect with no world in focus" | add the row with its covering task or test |  |  | unmet-requirement |
| R-46 | test-coverage | should-fix | in-change | delta scenario 'A trailing space is no argument' has no matrix row | review > absent-row — "A trailing space is no argument" | add the row with its covering task or test |  |  | unmet-requirement |
| R-47 | test-coverage | should-fix | in-change | delta scenario 'Disconnect an unknown world' has no matrix row | review > absent-row — "Disconnect an unknown world" | add the row with its covering task or test |  |  | unmet-requirement |
| R-48 | test-coverage | should-fix | in-change | delta scenario 'Quit twice while a text waits' has no matrix row | review > absent-row — "Quit twice while a text waits" | add the row with its covering task or test |  |  | unmet-requirement |
| R-49 | test-coverage | should-fix | in-change | delta scenario 'A command with no world in focus' has no matrix row | review > absent-row — "A command with no world in focus" | add the row with its covering task or test |  |  | unmet-requirement |
| R-50 | test-coverage | should-fix | in-change | delta scenario 'A file that cannot be read' has no matrix row | review > absent-row — "A file that cannot be read" | add the row with its covering task or test |  |  | unmet-requirement |
| R-51 | test-coverage | should-fix | in-change | delta scenario 'A binding replaces a default' has no matrix row | review > absent-row — "A binding replaces a default" | add the row with its covering task or test |  |  | unmet-requirement |
| R-52 | test-coverage | should-fix | in-change | delta scenario 'None on a key with a modifier' has no matrix row | review > absent-row — "None on a key with a modifier" | add the row with its covering task or test |  |  | unmet-requirement |
| R-53 | test-coverage | should-fix | in-change | delta scenario 'Two names of one key' has no matrix row | review > absent-row — "Two names of one key" | add the row with its covering task or test |  |  | unmet-requirement |
| R-54 | test-coverage | should-fix | in-change | delta scenario 'An uppercase key' has no matrix row | review > absent-row — "An uppercase key" | add the row with its covering task or test |  |  | unmet-requirement |
| R-55 | test-coverage | should-fix | in-change | delta scenario 'A refusal during a walk of the history' has no matrix row | review > absent-row — "A refusal during a walk of the history" | add the row with its covering task or test |  |  | unmet-requirement |
| R-56 | test-coverage | should-fix | in-change | delta scenario 'No empty entry' has no matrix row | review > absent-row — "No empty entry" | add the row with its covering task or test |  |  | unmet-requirement |
| R-57 | test-coverage | should-fix | in-change | delta scenario 'A focus change ends a walk' has no matrix row | review > absent-row — "A focus change ends a walk" | add the row with its covering task or test |  |  | unmet-requirement |
| R-58 | test-coverage | should-fix | in-change | delta scenario 'Back to the first world' has no matrix row | review > absent-row — "Back to the first world" | add the row with its covering task or test |  |  | unmet-requirement |
| R-59 | test-coverage | should-fix | in-change | delta scenario 'The echo of a pose of two lines' has no matrix row | review > absent-row — "The echo of a pose of two lines" | add the row with its covering task or test |  |  | unmet-requirement |
| R-60 | test-coverage | should-fix | in-change | delta scenario 'A send before the first list' has no matrix row | review > absent-row — "A send before the first list" | add the row with its covering task or test |  |  | unmet-requirement |
| R-61 | test-coverage | should-fix | in-change | delta scenario 'The first list fails' has no matrix row | review > absent-row — "The first list fails" | add the row with its covering task or test |  |  | unmet-requirement |
| R-27 | verify | should-fix | in-change | REGRESSED: the fix a9555c243143a1800eea6daef564f374719bf20f of R-27 did not hold. The key-name grammar has `+` both as the joiner and as a legal one-character key, so `ctrl++` is ambiguous; no task tests `+`, an uppercase character or a repeated modifier. | design.md > D3. Key names — "A key is one lowercase character, `plus` for the `+` key, or a name of a special key." | fix it again | regression |  | wrong-behaviour |

## Repeats

None.

## Out of bound

None.

## Accepted

| id | severity | disposition | reason | ticket |
|---|---|---|---|---|
| R-37 | suggestion | deferred | Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. | https://github.com/seanb4t/mu-portal/issues/164 |
| R-40 | suggestion | deferred | Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. | https://github.com/seanb4t/mu-portal/issues/164 |
| R-43 | suggestion | deferred | Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. | https://github.com/seanb4t/mu-portal/issues/164 |

## Rounds

| round | phase | kind | head | digest | verdict | new | repeats | regressions | resolved | out-of-bound | decision |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | plan | full | cd0f860fa27eb1117378b9c54cf28fff7f1499a8 | 0cc27524b56d703022f85cd2f165b79dc07d5bd9b3417e5a48d6adc5b4474e0e | changes-requested | 43 | 0 | 0 | 0 | 0 |  |
| 2 | plan | follow-up | c41b481932f355567bb67947a1f2746b0c929502 | 8f785eb1e63002f496fc05f4bbf2d5535e91ced2c90b207b05f699ac434a5253 | changes-requested | 18 | 0 | 2 | 38 | 0 |  |

## Judge

judge: 4 questions, 0.000 USD

## Coverage matrix

| scenario | covered by | status | verified |
|---|---|---|---|
| Help lists the entries | 3.3 | covered |  |
| A key and a command give the same focus | 3.4 | covered |  |
| An editing entry is not a command | 3.1 | covered |  |
| A command does not reach the game | 3.1 | covered |  |
| Several lines are not a command | 3.1 | covered |  |
| An escaped slash | 3.1 | covered |  |
| An unknown command | 3.1 | covered |  |
| A missing world | 3.1 | covered |  |
| An unknown world | 3.1 | covered |  |
| Connect the world in focus | 3.2 | covered |  |
| Disconnect a named world | 3.2 | covered |  |
| Focus by name | 3.1 | covered |  |
| Quit | 3.3 | covered |  |
| A file that sets the echo | 6.2 | covered |  |
| No file | 6.2 | covered |  |
| An unknown key | 6.2 | covered |  |
| A bad echo value | 6.2 | covered |  |
| Not TOML | 6.2 | covered |  |
| A new binding | 6.4 | covered |  |
| The default keys stay | 6.4 | covered |  |
| A removed binding | 6.4 | covered |  |
| An unknown entry | 6.4 | covered |  |
| An unknown key name | 6.4 | covered |  |
| An entry that needs an argument | 6.4 | covered |  |
| The default dims | 7.1 | covered |  |
| Hide | 7.1 | covered |  |
| A line goes to the world in focus | 2.1 | partial |  |
| An empty line | 2.1 | partial |  |
| Enter with no world | 2.1 | partial |  |
| A refused line | 2.1 | partial |  |
| A refused line after more text | 5.1 | covered |  |
| Ctrl+M sends | 2.1 | partial |  |
| A pose of two lines | 7.1 | partial |  |
| A line break that the world refuses | 2.1 | partial |  |
| A line that the server refuses | 5.1 | covered |  |
| The last line is refused | 5.1 | covered |  |
| A refused send of Enter with a send after it | 5.1 | covered |  |
| The sends stay apart | 5.1 | covered |  |
| The stream ends while lines wait | 5.1 | covered |  |
| Lines of two worlds wait | 5.1 | covered |  |
| The server resets while lines wait | 5.1 | covered |  |
| A refusal after the stream ended | 5.1 | covered |  |
| A send enters the history | 4.1 | covered |  |
| A pose is one entry | 4.1 | covered |  |
| Each world has its own history | 4.1 | covered |  |
| No repeat | 4.1 | covered |  |
| Back to the draft | 4.2 | covered |  |
| Up inside a pose moves the cursor | 4.2 | covered |  |
| The second world | 2.2 | covered |  |
| No world at that place | 2.2 | covered |  |
| Top and bottom | 2.2 | covered |  |
| Before the first list | 7.2 | covered |  |
| The flag wins over the environment |  | partial |  |
| The default directory |  | partial |  |
| The directory under the home directory |  | partial |  |
| An empty MU_CONFIG |  | partial |  |
| A relative XDG_CONFIG_HOME |  | partial |  |
| No home directory |  | partial |  |
| A missing directory |  | partial |  |
| A missing default directory |  | partial |  |
| The client takes the flag | 6.2 | covered |  |
| Worlds from character files |  | partial |  |
| A character with no game |  | partial |  |
| A file name outside the id rule |  | partial |  |
| A stray file | 6.3 | covered |  |
| A link to nothing |  | partial |  |
| The file of the server |  | partial |  |
| A character named server |  | partial |  |
| The file of the client | 6.3 | covered |  |

### Not run

Plan phase: no verification runs.

## Verification runs

None.

## Dropped

None.

## Lens notes

- verify: R-1: at head the default keymap binds plain Enter to Action::Submit (crates/mu-tui/src/keymap.rs:128), and on_input_key clears quit_armed for each action that is not Quit (update.rs:304). D4 still says a key that reaches another entry clears the arm, so the Enter of the second /quit clears it before the command is parsed. D4 must exempt the Submit or SubmitLines key that runs a /quit, and say whether edit entries such as Ctrl+U and Ctrl+J count as keys that only edit.
- verify: R-27: the fix of D3 and of the client-config requirement narrows a key to one lowercase character. A digit or a punctuation character is not lowercase, so a strict parser refuses alt+1, the default key of focus-1, and [keys] cannot rebind or unbind Alt+1 to Alt+9. Say: one character that is not an uppercase letter.
- verify: Out of bound: proposal.md What Changes still says that an unknown command shows a notice, while D4 now gives a bad command a hint (Model::hint). The proposal line is not in diff-hunks.txt.
- verify: D8 lists the ends of the first list as 'a reply with worlds, a failure, and the 30 s limit'. A reply with an empty list should also clear first_list. The natural reading clears the flag on each Msg::Listed, so this is a wording risk only.
- verify: D3 binds "none" to the entry ignore. With edits unset, /ignore then becomes a good command that clears the editor and enters history. This is harmless but may surprise.
- verify: R-35: the unchanged world-config scenarios are not named one by one, but each has a test in crates/mu-cli/tests/world_config.rs (flag_wins_over_the_environment, default_directory, stray_file, server_file_makes_no_world and others), and task 6.3 now runs the whole file.
- verify: R-36: the rule id appears only in design.md and an archived review. D3 now names it as an engram rule and states its content inline.

## Ledger

| id | first | last | severity | scope | consequence | location | disposition | basis | override | judge | claim |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: Quit by command"],"`/quit` SHALL do what Ctrl+C does, under the requirement \\"Ctrl+C quits the client only\\"."] | fixed | 7b6443877bc61629ce3453f165d7d7729ed95d19 |  |  | app-1: /quit cannot confirm a quit while texts wait: the keys that type a second /quit reset quit_armed, and no decision says how a command confirms.; inv-1: /quit can never leave while a send waits: each typed key of a second /quit clears the quit arm, so the double quit of Ctrl+C never completes by command. |
| R-2 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: A bad command keeps its text"],"It can also lack a required world, or name a world that the client does not hold."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | No decision checks the world of /disconnect: D10 checks only /focus and /connect against candidates, so /disconnect with an unknown world is not a bad command. |
| R-3 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/client-commands/spec.md","Requirement: Connect and disconnect a world"],"With no world and no world in focus, it SHALL be a bad command."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | app-3: No decision makes /connect or /disconnect with no world and no world in focus a bad command; D4 clears the editor and runs act for a good command.; inv-10: The spec makes /connect and /disconnect with no world and no world in focus a bad command, but no scenario or task test passes that null case. |
| R-4 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: A refused text stops the sends after it"],"It SHALL show the reason as a line with the prefix `-- `, and the count of the texts in the history."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  | severity? should-fix 0.60 | The refusal notice must hold the count of the texts put in history, but D6 only extends the existing count of the return_waiting notice; the refusal notice at head has no count. |
| R-5 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Scenario: The sends stay apart"],"**AND** after Up the editor holds `c` alone"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  | severity? should-fix 0.70 | In 'The sends stay apart' the editor gets the lines a and b through set_input, which puts the cursor at the end, on the last line; by D5 Up there moves the cursor and recalls nothing. |
| R-6 | 1 | 2 | blocker | in-change | unmet-requirement | ["artifact",["design.md","D2. `mu-cli` reads the file, `mu-tui` owns the type"],"It returns an error that names the key and the value, and `mu-cli` prints that error."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  | severity? should-fix 0.78 | The keymap error of D2 names only the key and the value, and mu-cli prints it as 'mu client: <error>', so stderr does not name client.toml for the three [keys] error scenarios. |
| R-7 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. The screen opens before the first list"],"That list has its own 5 s limit."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | D8 says the list that mu-cli makes before run has its own 5 s limit, but the first list_worlds call in client.rs has no limit; only the retry after starting a server does. |
| R-8 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D2. `mu-cli` reads the file, `mu-tui` owns the type"],"It parses the file with `toml` and `serde_path_to_error`, and gives the value to `mu_tui::run`."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | D2 plans a second TOML parse in mu-cli with toml and serde_path_to_error. mu_core::world::parse_toml already does that parse, with the file:line:column error format that server.toml uses. mu-cli does not depend on serde_path_to_error, and its allowlist row does not hold it. |
| R-9 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["proposal.md","Impact"],"The world resolver accepts `client.toml`."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  | scope? beyond-scope 0.73 | fit-3: The proposal puts the world resolver change in mu-cli. The resolver and its one top-level exception, is_server_file with SERVER_FILE, live in mu-core dir.rs. The design names neither that crate nor how the exception takes a second file, so a change kept to the named crates cannot make the scenario The file of the client hold.; app-10: The world resolver that refuses stray top-level files is in mu-core (world/dir.rs), but the proposal's Impact places its change under crates/mu-cli and lists no mu-core change. |
| R-10 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"`Help` adds one notice line for each entry."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  | scope? deviates-design 0.72 | D4 makes a bad command and /help add notices. The model has two kinds of -- line. A notice is kept, and shows again after each Reset and for the first world. A hint is for one moment. The help list and a bad command are of the moment, so as notices they come back after each Reset. |
| R-11 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D8. The screen opens before the first list"],"The model starts with `Listing::Asked` and a flag `first_list`, and the top row shows `listing worlds` until `Msg::Listed` arrives."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | D8 takes the first list through Msg::Listed, but does not say what happens to Model::from_listing. That constructor takes the first list now, and the tests build the model with it. The two paths handle a failed list differently: from_listing adds a notice, and Msg::Listed only sets list_failure. |
| R-12 | 1 | 2 | should-fix | in-change | drift | ["artifact",["design.md","D7. Echo"],"It shows the first part with `> ` and each other part with two spaces."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | D7 puts the split of a player line and its prefixes in update. text::line already owns the prefix rule of each source and the clean of the text, so the > prefix of a player line would have two places. |
| R-13 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","docs/src/content/docs/guides/configuration.md",36] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The configuration guide links each top-level file of the directory to its own page, and says that server.toml is the one exception to the stray-file rule. Task 6.5 adds a page for client.toml, but no task updates the configuration guide, so after the change it tells the player that client.toml is an error. |
| R-14 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-commands/spec.md","Requirement: A bad command keeps its text"],"A bad command names no entry, or an entry that does not run as a command."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | Nothing refuses an argument to an entry that takes none, so `/quit now` or `/focus-next beta/bryn` runs and drops the argument; the bad-command list omits that case. |
| R-15 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"The text after `/` is split at the first run of spaces into a name and an argument."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The D4 split leaves the empty argument undefined: `/connect ` with a trailing space may give Some("") and an unknown world instead of the world in focus; no test passes it. |
| R-16 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/world-config/spec.md","Requirement: The configuration directory"],"The flag `--config <dir>` of `mu server`, `mu world` and `mu client` SHALL name the configuration directory"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | `mu client --config <dir>` starts a server without the flag, so the client reads client.toml of <dir> while the server it starts reads worlds from MU_CONFIG or the XDG directory. |
| R-17 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["path","crates/mu-core/src/world/dir.rs",107] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The world resolver treats server.toml in any case, unreadable and hard-linked with edge tests, but task 6.3 tests client.toml only by its exact name, so `Client.toml` on a case-insensitive disk or an unreadable client.toml is untested. |
| R-18 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: A bad client.toml stops the client"],"A bad file is one that the client cannot read or parse"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | A client.toml that exists but cannot be read is a bad file, but no scenario or task test passes it; only a missing file and parse errors are tested. |
| R-19 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/terminal-client/spec.md","Scenario: No world at that place"],"**WHEN** the user presses Alt+5"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The focus-by-number test passes Alt+5 with two worlds, not Alt+3, the first place past the last world, where an off-by-one index check would fail. |
| R-20 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: The top row shows the first list"],"a key that sends SHALL show that no world is in focus."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The rule that a key that sends during the first list shows that no world is in focus has no test; before_the_first_list checks only the top row. |
| R-21 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/terminal-client/spec.md","Requirement: Each world keeps an input history"],"The client SHALL NOT add an empty text, or a text equal to the newest entry."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The rule that an empty text never enters the history has no test; no_repeat covers only equal texts, and an empty Enter is a normal send. |
| R-22 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D7. Echo"],"With `Echo::Hide` the client adds no line, but the event still counts for the unseen mark."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | D7 keeps the unseen mark for a hidden echo, but the hide scenario only checks the focused world, so the mark for a player event of a world not in focus is untested. |
| R-23 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["specs/client-config/spec.md","Requirement: The keys table binds a key to an entry"],"Each binding SHALL replace the default binding of that key, and `\\"none\\"` SHALL remove it."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | "none" on a modified chord whose plain key is bound falls back to the plain entry, so `"shift+enter" = "none"` would send instead of doing nothing; the only test unbinds ctrl+n, whose plain key has no entry. |
| R-24 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D8. The screen opens before the first list"],"the top row shows `listing worlds` until `Msg::Listed` arrives."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The first list can end in a failure, which sets only list_failure; no test checks that the top row leaves `listing worlds` then, so the flag of D8 can stay set forever. |
| R-25 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["specs/client-config/spec.md","Requirement: The keys table binds a key to an entry"],"Each binding SHALL replace the default binding of that key"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | No scenario binds a key that already has a default, so the rule that a binding replaces the default is never tested; a_new_binding uses alt+n, which has no default. |
| R-26 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Key names"],"The order of the modifiers does not matter."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | Two [keys] names of one chord (`ctrl+shift+z` and `shift+ctrl+z`) are distinct BTreeMap keys, so one binding silently wins by key order; nothing refuses the pair. |
| R-27 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D3. Key names"],"A key is one character or one name from this list:"] | fixed | 7b6443877bc61629ce3453f165d7d7729ed95d19 |  |  | The key-name grammar has `+` both as the joiner and as a legal one-character key, so `ctrl++` is ambiguous; no task tests `+`, an uppercase character or a repeated modifier. |
| R-28 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D5. History is a state of each world"],"the place of the walk, and the draft that the first Up saved."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The draft lives in the History of one world while the pose editor is shared, so a focus change during a walk leaves the draft in the old world and the new world's walk saves a recalled entry as its draft. |
| R-29 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D6. Returned sends go to history (#151)"],"The other returned texts, and then the text that the editor held, go into the history of their world."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | A refusal or a stream end during a walk pushes the recalled entry as 'the text that the editor held', and nothing pushes the saved draft, so the draft is lost; the D5 property test excludes any put_back. |
| R-30 | 1 | 2 | should-fix | in-change | wrong-behaviour | ["artifact",["design.md","D4. Commands are parsed in `update` at a send"],"A good command clears the editor, adds the text to the history of the world in focus, and runs `act`."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | A good command goes into the history of the world in focus, but focus is None with no world or before the first list ends; no test runs a command in that null case. |
| R-31 | 1 | 2 | should-fix | in-change | next-change | ["artifact",["proposal.md","What Changes"],"**BREAKING** (spec only, no release yet): after a refusal, a timeout, a stream end or a reset, only the first returned send goes back"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The proposal marks the refusal return path as BREAKING, but no delta spec carries a BREAKING marker. |
| R-32 | 1 | 2 | should-fix | in-change | false-test | ["artifact",["specs/world-config/spec.md","Scenario: The client takes the flag"],"**THEN** the client opens the screen"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The THEN step of the scenario 'The client takes the flag' names no observable: 'the client opens the screen' names no row, status or value. |
| R-33 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","2. The registry and one dispatch"],"every test of `crates/mu-tui` still passes"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | Six unchanged scenarios of the modified Enter requirement are named by no task. Only the blanket 2.1 line covers them. |
| R-34 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","7. Echo and the first list"],"`a_pose_of_two_lines` (echo) in `app.rs` pass"] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The heading 'A pose of two lines' occurs twice (Enter requirement, echo requirement). Task 7.1 names one test '(echo)', so the %r send scenario has no named test. |
| R-35 | 1 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["tasks.md","6. Configuration"],"Make sure that `the_file_of_the_client` in `world_config.rs` passes, and that `a_stray_file` still passes."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | Eleven unchanged world-config scenarios (flag, defaults, env, missing dir, character files, server file) are named by no task. Task 6.3 names only a_stray_file and the_file_of_the_client. |
| R-36 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D3. Key names"],"Repository rule 5rk42v8qs1 asks for a crate first."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | Repository rule 5rk42v8qs1 is not found anywhere in the repository outside an archived review, so the rule that D3 and D10 rely on cannot be read. |
| R-37 | 1 | 1 | suggestion | in-change |  | ["artifact",["design.md","Risks / Trade-offs"],"A player who wants the old look sets `echo = \\"show\\"`. \\u2192 The configuration page says so."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/164; reason: Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. |  |  | The risk of the new dim default is mitigated by the configuration page, but task 6.5 checks only that the page names /help, not that it states the old look is echo = "show". |
| R-38 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","Context"],"`Action` (`crates/mu-tui/src/update.rs`) is a `Copy` enum of 15 variants with no names."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | Action has 13 variants at head, not 15. |
| R-39 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","Context"],"`mu-cli` already parses TOML with `toml` and `serde_path_to_error` (`crates/mu-cli/src/config.rs`)."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | mu-cli does not parse TOML with serde_path_to_error in config.rs; it calls mu_core::world::parse_toml (server_config.rs), and serde_path_to_error is a dependency of mu-core only. |
| R-40 | 1 | 1 | suggestion | in-change |  | ["path","crates/mu-cli/src/server_config.rs",63] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/164; reason: Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. |  |  | The reader of client.toml repeats server_config::read. Both join the file name to the directory, give the default for a missing directory or file, and name the path in a read error. Only the file name, the Hidden value and the certificate fix-up of server.toml differ. |
| R-41 | 1 | 2 | suggestion | in-change |  | ["path","crates/mu-tui/src/keymap.rs",53] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | The keymap keeps an Action for each chord, not the registry entry. The command palette of client-layout lists the registry, and it must map each Action back to an entry to show its key. The D1 test that each default action comes from an entry needs the same reverse map. |
| R-42 | 1 | 2 | suggestion | in-change |  | ["artifact",["design.md","D3. Key names"],"A property test checks that it reads back each name that it writes."] | resolved | resolved in round 2 from fixed a9555c243143a1800eea6daef564f374719bf20f |  |  | D3 promises a property test that the key-name parser reads back each name it writes, but no task creates it. |
| R-43 | 1 | 1 | suggestion | in-change |  | ["artifact",["tasks.md","3. Client commands"],"Make sure that `connect_the_world_in_focus` and `disconnect_a_named_world` in `client.rs` pass."] | deferred | ticket: https://github.com/seanb4t/mu-portal/issues/164; reason: Low-cost polish of the client-shell plan: the dim-echo note in the guide, a shared reader for top-level config files, and runnable test commands in tasks; deferred to one ticket by the owner. |  |  | Tasks name test functions but only 6.1 gives a runnable command. Each test is named without a cargo nextest line. |
| R-44 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An argument to an entry that takes none"] |  |  |  |  | delta scenario 'An argument to an entry that takes none' has no matrix row |
| R-45 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Connect with no world in focus"] |  |  |  |  | delta scenario 'Connect with no world in focus' has no matrix row |
| R-46 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A trailing space is no argument"] |  |  |  |  | delta scenario 'A trailing space is no argument' has no matrix row |
| R-47 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Disconnect an unknown world"] |  |  |  |  | delta scenario 'Disconnect an unknown world' has no matrix row |
| R-48 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Quit twice while a text waits"] |  |  |  |  | delta scenario 'Quit twice while a text waits' has no matrix row |
| R-49 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A command with no world in focus"] |  |  |  |  | delta scenario 'A command with no world in focus' has no matrix row |
| R-50 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A file that cannot be read"] |  |  |  |  | delta scenario 'A file that cannot be read' has no matrix row |
| R-51 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A binding replaces a default"] |  |  |  |  | delta scenario 'A binding replaces a default' has no matrix row |
| R-52 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"None on a key with a modifier"] |  |  |  |  | delta scenario 'None on a key with a modifier' has no matrix row |
| R-53 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Two names of one key"] |  |  |  |  | delta scenario 'Two names of one key' has no matrix row |
| R-54 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"An uppercase key"] |  |  |  |  | delta scenario 'An uppercase key' has no matrix row |
| R-55 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A refusal during a walk of the history"] |  |  |  |  | delta scenario 'A refusal during a walk of the history' has no matrix row |
| R-56 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"No empty entry"] |  |  |  |  | delta scenario 'No empty entry' has no matrix row |
| R-57 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A focus change ends a walk"] |  |  |  |  | delta scenario 'A focus change ends a walk' has no matrix row |
| R-58 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"Back to the first world"] |  |  |  |  | delta scenario 'Back to the first world' has no matrix row |
| R-59 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The echo of a pose of two lines"] |  |  |  |  | delta scenario 'The echo of a pose of two lines' has no matrix row |
| R-60 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"A send before the first list"] |  |  |  |  | delta scenario 'A send before the first list' has no matrix row |
| R-61 | 2 | 2 | should-fix | in-change | unmet-requirement | ["artifact",["review","absent-row"],"The first list fails"] |  |  |  |  | delta scenario 'The first list fails' has no matrix row |
