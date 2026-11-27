# Spec Delta

## Purpose

Defines the line rules of a world: a match part that finds lines from the game, and the actions that apply to a line that matches. The first action is `stream`, which sends a line to named streams. The spec covers the checks of a rule, the order of the rules, the names from captures, the captures of several lines, and `mu world match`.

## ADDED Requirements

### Requirement: A rule is a match part and actions
A rule SHALL have a match part, with `pattern`, `until` and `lines`, and actions. The only action of this change SHALL be `stream`. A rule with no action SHALL do nothing, and the server SHALL NOT check it beyond its `id`. A rule with an action SHALL hold `pattern`. Another field in a rule with an action SHALL be an error that names the file, the id and the field.

#### Scenario: A rule with a stream loads
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\[(\w+)\] '` and `stream = "chan-$1"`
- **WHEN** the user runs `mu world show alpha/aria --config <dir>`
- **THEN** stdout lists the rule `chan`
- **AND** the exit status is 0

#### Scenario: An unknown field
- **GIVEN** the game `alpha` has the rule `chan` with `pattern = "x"`, `stream = "chat"` and `colour = "red"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `chan` and `colour`
- **AND** the exit status is 1

#### Scenario: A rule with no action
- **GIVEN** `global.toml` has the rule `hl-page` with `pattern = "pages"` and `colour = "red"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `Bob pages: hi`
- **THEN** stdout is the line `main` TAB `Bob pages: hi`
- **AND** the exit status is 0

#### Scenario: An action with no pattern
- **GIVEN** the game `alpha` has the rule `chan` with `stream = "chat"` and no `pattern`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `chan` and `pattern`
- **AND** the exit status is 1

### Requirement: The fields of a rule are checked at the load
In a rule with an action, `pattern` and `until` SHALL be strings that compile as regular expressions. `lines` SHALL be an integer from 1 to 1000, in a rule that has `until`. `stream` SHALL be a string or a list of 1 to 8 strings. The server SHALL turn each entry of `stream` with no `$` to lower case. Then each such entry SHALL be a valid stream name other than `notices`, because only the client writes that stream. Each error SHALL name the file, the id and the field.

#### Scenario: A pattern that does not compile
- **GIVEN** the character `alpha/aria` has the rule `chan` with `pattern = "(["` and `stream = "chat"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `characters/alpha/aria.toml`, `chan` and `pattern`
- **AND** the exit status is 1

#### Scenario: A fixed stream that is not a name
- **GIVEN** the game `alpha` has the rule `chan` with `pattern = "x"` and `stream = ["chat", "1chat"]`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `chan` and `1chat`
- **AND** the exit status is 1

#### Scenario: A fixed stream in upper case
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\['` and `stream = "Chat"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[Public] hi`
- **THEN** stdout is the line `chat` TAB `[Public] hi`

#### Scenario: An empty list of streams
- **GIVEN** the game `alpha` has the rule `chan` with `pattern = "x"` and `stream = []`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `chan` and `stream`
- **AND** the exit status is 1

#### Scenario: Too many streams
- **GIVEN** the game `alpha` has the rule `chan` with `pattern = "x"` and a `stream` list of 9 names
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `chan` and `stream`
- **AND** the exit status is 1

#### Scenario: Lines out of range
- **GIVEN** the game `alpha` has the rule `who` with `pattern = "x"`, `stream = "who"`, `until = "y"` and `lines = 1001`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `who` and `lines`
- **AND** the exit status is 1

#### Scenario: Lines of zero
- **GIVEN** the game `alpha` has the rule `who` with `pattern = "x"`, `stream = "who"`, `until = "y"` and `lines = 0`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `who` and `lines`
- **AND** the exit status is 1

#### Scenario: Lines without until
- **GIVEN** the game `alpha` has the rule `who` with `pattern = "x"`, `stream = "who"` and `lines = 5`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `who` and `lines`
- **AND** the exit status is 1

#### Scenario: A fixed stream names the notices
- **GIVEN** the game `alpha` has the rule `chan` with `pattern = "x"` and `stream = ["NOTICES", "main"]`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `chan` and `NOTICES`
- **AND** the exit status is 1

### Requirement: A stream name is an id
A stream name SHALL be lower case. Before mu checks a name, it SHALL turn each upper-case letter to lower case. A stream name SHALL be 1 to 32 characters from `a` to `z`, `0` to `9` and `-`. It SHALL start with a letter, and SHALL NOT be a word of the layout sentences. These are the rules of a block id, so a stream name can also be the id of an `output`.

#### Scenario: A name that is a word of the sentences
- **GIVEN** the game `alpha` has the rule `chan` with `pattern = "x"` and `stream = "showing"`
- **WHEN** the user runs `mu world list --config <dir>`
- **THEN** stderr names `games/alpha.toml`, `chan` and `showing`
- **AND** the exit status is 1

### Requirement: The first rule that matches applies
The server SHALL run the rules of a world that have an action on each line from the game. The rules SHALL run after the hooks, in their merged order. The first rule whose `pattern` matches SHALL apply its actions. No later rule SHALL run on that line. A line that no rule matches SHALL have the one stream `main`.

#### Scenario: The earlier rule wins
- **GIVEN** `global.toml` has the rule `pub` with `pattern = '^\[Public\]'` and `stream = "public"`, and then the rule `chan` with `pattern = '^\['` and `stream = "chat"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `[Public] hi` and `[Help] yo`
- **THEN** stdout is the lines `public` TAB `[Public] hi` and `chat` TAB `[Help] yo`

#### Scenario: A character rule after a global rule
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\['` and `stream = "chat"`, and the character `alpha/aria` has the rule `pub` with `pattern = '^\[Public\]'` and `stream = "public"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[Public] hi`
- **THEN** stdout is the line `chat` TAB `[Public] hi`

#### Scenario: No match
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\['` and `stream = "chat"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `Bob says, "hi"`
- **THEN** stdout is the line `main` TAB `Bob says, "hi"`

#### Scenario: A rule that names main
- **GIVEN** `global.toml` has the rule `mine` with `pattern = '^\[Public\] Aria'` and `stream = "main"`, and then the rule `chan` with `pattern = '^\['` and `stream = "chat"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[Public] Aria says hi`
- **THEN** stdout is the line `main` TAB `[Public] Aria says hi`

### Requirement: A pattern has the full syntax of fancy-regex
A pattern SHALL accept named groups, lookahead, lookbehind, back-references and inline flags such as `(?i)`. Case SHALL count unless a flag says otherwise. A pattern SHALL match the text of the line without its color codes.

#### Scenario: A lookbehind and a named group
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '(?<=^\[)(?<chan>\w+)(?=\] )'` and `stream = "chan-$chan"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[Ooc] hi`
- **THEN** stdout is the line `chan-ooc` TAB `[Ooc] hi`

#### Scenario: A back-reference
- **GIVEN** `global.toml` has the rule `twice` with `pattern = '^(\w+) \1$'` and `stream = "echo"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `hey hey` and `hey you`
- **THEN** stdout is the lines `echo` TAB `hey hey` and `main` TAB `hey you`

#### Scenario: Case counts
- **GIVEN** `global.toml` has the rule `pub` with `pattern = '^\[public\]'` and `stream = "public"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[Public] hi`
- **THEN** stdout is the line `main` TAB `[Public] hi`

#### Scenario: A colored line
- **GIVEN** `global.toml` has the rule `pub` with `pattern = '^\[Public\] hi$'` and `stream = "public"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line ESC `[1m[Public]` ESC `[0m hi`
- **THEN** the first field of the line on stdout is `public`
- **AND** the second field holds the two color codes of the input

### Requirement: A stream name comes from a template
The server SHALL expand `$1`, `$name`, `${name}` and `$$` in each entry of `stream` with the captures of the match. A group that did not take part in the match SHALL give an empty text. The server SHALL turn each result to lower case, and SHALL drop a name that is twice in the list. If a result is not a valid stream name, the line SHALL have the one stream `main`.

#### Scenario: One stream for each channel
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\[([\w-]+)\] '` and `stream = "chan-$1"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `[Public] hi` and `[RP-Help] yo`
- **THEN** stdout is the lines `chan-public` TAB `[Public] hi` and `chan-rp-help` TAB `[RP-Help] yo`

#### Scenario: A name from a capture that is not valid
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\[([^\]]+)\] '` and `stream = "chan-$1"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[Role Play] hi`
- **THEN** stdout is the line `main` TAB `[Role Play] hi`

#### Scenario: A line in two streams
- **GIVEN** `global.toml` has the rule `page` with `pattern = 'pages: '` and `stream = ["pages", "main"]`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `Bob pages: hi`
- **THEN** stdout is the line `pages,main` TAB `Bob pages: hi`

### Requirement: A bad name is reported once
A name from a capture is bad if it is not valid, or if it is `notices`, because only the client writes that stream. If a name from a capture is bad, the server SHALL record a script error event that names the rule and the name. It SHALL record no other bad-name event for that rule until the world connects again.

#### Scenario: Two bad names
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^You say, "\[([^\]]+)\] '` and `stream = "chan-$1"`
- **AND** a server runs with `--config <dir>` and the fake world `alpha/aria`
- **WHEN** a client sends `[Role Play] one` and then `[Role Play] two` to `alpha/aria`, and reads the stream
- **THEN** the stream holds exactly one script error event for `alpha/aria`, and its text contains `chan` and `chan-role play`
- **AND** the game events `You say, "[Role Play] one"` and `You say, "[Role Play] two"` each have the one stream `main`

#### Scenario: A capture names the notices
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\[(\w+)\] '` and `stream = "$1"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[notices] hi`
- **THEN** stdout is the line `main` TAB `[notices] hi`
- **AND** stderr names `chan` and `notices`

### Requirement: Only lines from the game are matched
The server SHALL run the rules on the lines from the game only. Each other event SHALL have the one stream `main`. Prompts, side-protocol messages, player lines, echoes, logins, connects and script errors are such events.

#### Scenario: The line that the player sends
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '\[Public\]'` and `stream = "public"`
- **AND** a server runs with `--config <dir>` and the fake world `alpha/aria`
- **WHEN** a client sends `[Public] hi` to `alpha/aria` and reads the stream
- **THEN** the player event `[Public] hi` has the one stream `main`
- **AND** the game event `You say, "[Public] hi"` has the one stream `public`

### Requirement: A capture can span several lines
If a rule has `until`, the line that matches `pattern` SHALL start a capture. Each later line from the game SHALL get the same streams. The capture SHALL include the first later line that matches `until`, and SHALL end there. While a capture runs, no other rule SHALL run. The line that starts a capture SHALL NOT end it.

#### Scenario: A WHO list
- **GIVEN** `global.toml` has the rule `who` with `pattern = '^Player Name'`, `until = 'players are connected\.$'` and `stream = "who"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `Player Name  Idle`, `Aria  3m`, `2 players are connected.` and `Bob says, "hi"`
- **THEN** the first fields of the four lines on stdout are `who`, `who`, `who` and `main`

#### Scenario: A line that starts and could end
- **GIVEN** `global.toml` has the rule `who` with `pattern = '^WHO'`, `until = 'WHO'` and `stream = "who"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `WHO list`, `Aria` and `end of WHO`
- **THEN** the first fields of the three lines on stdout are `who`, `who` and `who`

#### Scenario: Another rule during a capture
- **GIVEN** `global.toml` has the rule `who` with `pattern = '^Player Name'`, `until = '^End'` and `stream = "who"`, and then the rule `chan` with `pattern = '^\['` and `stream = "chat"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `Player Name`, `[Public] hi`, `End` and `[Public] yo`
- **THEN** the first fields of the four lines on stdout are `who`, `who`, `who` and `chat`

### Requirement: A capture ends at its line limit
A capture SHALL hold at most `lines` lines, the first included. `lines` SHALL default to 100. When a capture reaches that count, it SHALL end. The server SHALL then match the next line again from the first rule.

#### Scenario: A capture that never sees its end
- **GIVEN** `global.toml` has the rule `who` with `pattern = '^WHO'`, `until = '^End'`, `lines = 2` and `stream = "who"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `WHO`, `Aria`, `Bob` and `WHO`
- **THEN** the first fields of the four lines on stdout are `who`, `who`, `main` and `who`

#### Scenario: A capture of one line
- **GIVEN** `global.toml` has the rule `who` with `pattern = '^WHO'`, `until = '^End'`, `lines = 1` and `stream = "who"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin lines `WHO` and `Aria`
- **THEN** the first fields of the two lines on stdout are `who` and `main`

#### Scenario: The default limit
- **GIVEN** `global.toml` has the rule `who` with `pattern = '^WHO'`, `until = '^End'` and `stream = "who"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `WHO` and then 100 lines `x`
- **THEN** the first field of the first 100 lines on stdout is `who`, and the first field of the last line is `main`

### Requirement: A capture ends at a disconnect
If its world disconnects, a capture SHALL end. The first line from the game after a connect SHALL be matched from the first rule.

#### Scenario: A drop during a capture
- **GIVEN** `global.toml` has the rule `who` with `pattern = '^You say, "WHO'`, `until = '^End'` and `stream = "who"`
- **AND** a server runs with `--config <dir>` and the fake world `alpha/aria`
- **WHEN** a client sends `WHO` and then `@drop` to `alpha/aria`, connects `alpha/aria` again, sends `hi`, and reads the stream
- **THEN** the game event `You say, "WHO"` has the one stream `who`
- **AND** the game event `You say, "hi"` has the one stream `main`

### Requirement: A slow match counts as no match
If a match passes the backtrack limit of the engine, the match SHALL stop. The line SHALL then count as no match for that rule, and the server SHALL try the next rule. The server SHALL record one script error event that names the rule. It SHALL record no other slow-match event for that rule until the world connects again. A bad-name event of the same rule SHALL NOT count against this limit.

#### Scenario: A pattern that backtracks too much
- **GIVEN** `global.toml` has the rule `slow` with `pattern = '^You say, "(a|aa)+\1b'` and `stream = "slow"`, and then the rule `all` with `pattern = '^You say, "a'` and `stream = "rest"`
- **AND** a server runs with `--config <dir>` and the fake world `alpha/aria`
- **WHEN** a client sends a line of 40 `a` characters to `alpha/aria` and reads the stream
- **THEN** the game event of that line has the one stream `rest`
- **AND** the stream holds one script error event whose text contains `slow`

### Requirement: `mu world match` shows the streams of each line
`mu world match <world> --config <dir>` SHALL read lines from stdin. It SHALL match each one as a line from the game of that world, with one capture state for the whole input. For each line it SHALL print the streams with a comma between them, then a tab, then the line. It SHALL print each script error on stderr and still exit 0. An unknown world or a bad configuration SHALL be an error with the exit status 1.

#### Scenario: An unknown world
- **WHEN** the user runs `mu world match nowhere/nobody --config <dir>` with an empty stdin
- **THEN** stderr names `nowhere/nobody`
- **AND** the exit status is 1

#### Scenario: No rules
- **GIVEN** the configuration has no rule with an action
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `hi`
- **THEN** stdout is the line `main` TAB `hi`
- **AND** the exit status is 0

#### Scenario: A bad name on stderr
- **GIVEN** `global.toml` has the rule `chan` with `pattern = '^\[([^\]]+)\] '` and `stream = "chan-$1"`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `[Role Play] hi`
- **THEN** stderr holds one line that contains `chan` and `chan-role play`
- **AND** the exit status is 0

### Requirement: `mu world match` splits stdin into lines
`mu world match` SHALL split stdin into lines with the line rule of a session. It SHALL drop each CR, and it SHALL cut a line at the longest line that a session holds. It SHALL NOT apply the telnet steps of a session: it SHALL keep a NUL after a CR, which a session drops.

#### Scenario: A carriage return in a line
- **GIVEN** the configuration has a rule with the pattern `^two$` and the stream `x`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `t`, a CR, and `wo`
- **THEN** stdout is the line `x` TAB `two`

#### Scenario: A NUL after a carriage return
- **GIVEN** the configuration has a rule with the pattern `^ab$` and the stream `x`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with the stdin line `a`, a CR, a NUL, and `b`
- **THEN** stdout is the line `main` TAB `a`, a NUL, and `b`

#### Scenario: A line longer than a session holds
- **GIVEN** the configuration has a rule with the pattern `^a` and the stream `x`
- **WHEN** the user runs `mu world match alpha/aria --config <dir>` with one stdin line of `a` characters, 3 longer than the longest line that a session holds
- **THEN** stdout has two lines, and each one starts with `x` TAB
