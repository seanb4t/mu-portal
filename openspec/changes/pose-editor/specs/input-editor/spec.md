# Spec Delta

## Purpose

The editor widget of the terminal client: a text of several lines that the player writes, wraps, pastes and sends. The pose editor is one instance of it.

## ADDED Requirements

### Requirement: The editor wraps a long line
The editor SHALL wrap a line that is wider than its area at a space. It SHALL break inside a word only where the word is wider than the area. A wrap SHALL NOT change the text of the editor.

#### Scenario: A line wider than the editor
- **GIVEN** the client runs in a terminal of 40 columns
- **WHEN** the user types 10 words of 5 letters each, with a space between each two, and presses Enter
- **THEN** before the Enter, the editor shows the words on 2 rows, and no row starts or ends inside a word
- **AND** the server records one player event with the 59 characters as typed

#### Scenario: A word wider than the editor
- **GIVEN** the client runs in a terminal of 20 columns
- **WHEN** the user types 30 letters `x` with no space
- **THEN** the editor shows 20 letters on its first row and 10 on its second

### Requirement: The editor grows with its text
The editor SHALL take one row for each wrapped row of its text, and at least one row. It SHALL take at most 8 rows, and at most half the rows of the terminal, rounded down, but at least one. If the text has more rows, the editor SHALL scroll so that the row of the cursor shows.

#### Scenario: Three lines
- **GIVEN** the client runs in a terminal of 80 columns and 24 rows
- **WHEN** the user types `one`, Ctrl+J, `two`, Ctrl+J and `three`
- **THEN** the editor takes the 3 bottom rows and shows `one`, `two` and `three`
- **AND** the output pane ends on the row above the editor

#### Scenario: More rows than the limit
- **GIVEN** the client runs in a terminal of 80 columns and 24 rows
- **WHEN** the user types the lines `1` to `12`, with Ctrl+J between each two
- **THEN** the editor takes the 8 bottom rows and shows the lines `5` to `12`

#### Scenario: A short terminal
- **GIVEN** the client runs in a terminal of 80 columns and 10 rows
- **WHEN** the user types the lines `1` to `12`, with Ctrl+J between each two
- **THEN** the editor takes the 5 bottom rows and shows the lines `8` to `12`

#### Scenario: A terminal of 17 rows
- **GIVEN** the client runs in a terminal of 80 columns and 17 rows
- **WHEN** the user types the lines `1` to `12`, with Ctrl+J between each two
- **THEN** the editor takes the 8 bottom rows and shows the lines `5` to `12`

#### Scenario: A terminal of 3 rows
- **GIVEN** the client runs in a terminal of 80 columns and 3 rows
- **WHEN** the user types `one`, Ctrl+J, `two`, Ctrl+J and `three`
- **THEN** the editor takes 1 row and shows `three`

#### Scenario: A terminal of 1 row
- **GIVEN** the client runs in a terminal of 80 columns and 1 row
- **WHEN** the user types `one`, Ctrl+J, `two`, Ctrl+J and `three`
- **THEN** the client draws the frame, and the editor takes at most 1 row

#### Scenario: Back to one row
- **GIVEN** the editor shows 3 lines
- **WHEN** the user presses Enter
- **THEN** the editor takes 1 row, and the output pane takes the rows that the editor gave back

### Requirement: The cursor moves by grapheme
Left and Right SHALL move the cursor over one whole grapheme. Backspace SHALL delete the whole grapheme before the cursor, and Delete the whole grapheme after it. After each other move, such as Up and Down, the cursor SHALL be at a grapheme boundary. A grapheme is one character as the reader sees it, such as an emoji with joiners, a flag or a letter with a combining accent.

#### Scenario: Backspace after an emoji with joiners
- **WHEN** the user types `a👨‍👩‍👧` and presses Backspace once
- **THEN** the editor holds `a`

#### Scenario: Left over a flag
- **WHEN** the user types `a🇺🇸`, presses Left once and types `x`
- **THEN** the editor holds `ax🇺🇸`

#### Scenario: Right over a combining accent
- **WHEN** the user types `e` U+0301 `z`, presses Home, presses Right once and types `x`
- **THEN** the editor holds `e` U+0301 `xz`

#### Scenario: Delete before a skin tone
- **WHEN** the user types `👍🏽b`, presses Home and presses Delete once
- **THEN** the editor holds `b`

#### Scenario: Up into an emoji with joiners
- **WHEN** the user types `a👨‍👩‍👧`, Ctrl+J and `xxx`, presses Up once and types `z`
- **THEN** the first line of the editor is `a👨‍👩‍👧z`

### Requirement: The editor uses the widths of the terminal
The editor SHALL use the same width for each grapheme as the output pane. That is the width that the startup probe learned. If the probe learned no width, it is the `unicode-width` value. The cursor, the wrap and the drawn cells SHALL all use that width.

#### Scenario: A terminal that adds code point widths
- **GIVEN** a terminal that answers the cursor position query after `👨‍👩‍👧` at column 9, so the width of that class is 8
- **WHEN** the user types `👨‍👩‍👧x`
- **THEN** the cell of `x` in the editor is at column 9
- **AND** the terminal cursor is at column 10 of the editor row

#### Scenario: A terminal that does not answer
- **GIVEN** a terminal that answers no query
- **WHEN** the user types `👨‍👩‍👧x`
- **THEN** the cell of `x` in the editor is at column 3

### Requirement: Keys insert a line break
Alt+Enter and Ctrl+J SHALL insert a line break at the cursor. If the terminal reports the Shift modifier with Enter, Shift+Enter SHALL insert one. A terminal with the kitty keyboard protocol reports it. Enter with no modifier SHALL NOT insert one.

#### Scenario: Each key of a line break
- **WHEN** the user types `a`, the client gets the key event of one of Shift+Enter, Alt+Enter and Ctrl+J, and the user types `b`
- **THEN** the editor holds `a`, a line break and `b`, on 2 rows

### Requirement: A paste is one edit
A bracketed paste SHALL insert its text at the cursor as one edit. Each CR LF and each lone CR of the paste SHALL become LF. The editor SHALL drop each other control character of the paste, apart from the tab. One undo SHALL remove the whole paste. A paste SHALL NOT delete a selection.

#### Scenario: A paste of several lines
- **GIVEN** the editor holds `x`
- **WHEN** the terminal sends a bracketed paste of `one` CR LF `two` CR `three`
- **THEN** the editor holds `x` and then the 3 lines `one`, `two` and `three`
- **AND** the server records no event

#### Scenario: Undo of a paste
- **GIVEN** the editor holds `x`
- **WHEN** the terminal sends a bracketed paste of 4096 bytes in 51 lines, and the user presses Ctrl+Z once
- **THEN** the editor holds `x`

#### Scenario: Control characters in a paste
- **WHEN** the terminal sends a bracketed paste of `a` ESC `[31mb` BEL tab `c` U+009B `d`
- **THEN** the editor holds `a[31mb` tab `cd`
- **AND** the client writes no ESC that the paste held to the terminal

#### Scenario: A paste with a selection
- **GIVEN** the editor holds `xy`, and the user selected `y` with Shift+Left
- **WHEN** the terminal sends a bracketed paste of `z`, and the user presses Ctrl+Z once
- **THEN** before the Ctrl+Z, the editor holds `xzy`
- **AND** after the Ctrl+Z, the editor holds `xy`

### Requirement: The terminal cursor is at the insertion point
After each frame, the client SHALL show the terminal cursor at the cell of the insertion point of the editor. An input method opens its box at that cell. If the text fills the last row of the editor exactly, the cursor SHALL be at the last column of that row.

#### Scenario: The cursor after typing
- **GIVEN** the client runs in a terminal of 80 columns and 24 rows
- **WHEN** the user types `hello`
- **THEN** the terminal cursor shows, and it is on the last row at column 6

#### Scenario: The cursor on a wrapped row after a wide grapheme
- **GIVEN** the client runs in a terminal of 20 columns
- **WHEN** the user types 18 letters `x`, then `漢字`
- **THEN** the terminal cursor is on the second row of the editor, at column 3, after `字`

#### Scenario: The cursor after a full row
- **GIVEN** the client runs in a terminal of 20 columns
- **WHEN** the user types 20 letters `x`
- **THEN** the editor takes 1 row, and the terminal cursor is on that row at column 20
