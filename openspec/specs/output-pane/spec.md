# output-pane Specification

## Purpose
The output pane is the widget of the terminal client that shows the lines of one world. Each pane and each later spawn is one instance of it, with its own lines, scroll position and selection.

## Requirements

### Requirement: The pane wraps lines to its width
The pane SHALL wrap each line to the width of its area, at a space where it can. It SHALL break a word only where the word is wider than the area. A row SHALL hold no cell past the right edge of the area. A wide grapheme that does not fit at the end of a row SHALL start the next row.

#### Scenario: A long line wraps at a space
- **GIVEN** a pane 20 columns wide that holds the line `the quick brown fox jumps over the lazy dog`
- **WHEN** the pane renders
- **THEN** the rows show `the quick brown fox`, `jumps over the lazy` and `dog`

#### Scenario: A word wider than the pane
- **GIVEN** a pane 10 columns wide that holds the line `abcdefghijklmnop`
- **WHEN** the pane renders
- **THEN** the rows show `abcdefghij` and `klmnop`

#### Scenario: A wide grapheme at the end of a row
- **GIVEN** a pane 5 columns wide that holds the line `abcd日本`
- **WHEN** the pane renders
- **THEN** the first row shows `abcd` and the second row shows `日本`
- **AND** no cell of the first row is past column 5

### Requirement: A line always has a row
Each line SHALL take at least one row, an empty line too. A grapheme wider than the area SHALL take a row of its own and show as U+FFFD in one cell. An area 0 columns wide SHALL show no row.

#### Scenario: An empty line
- **GIVEN** a pane that holds the lines `a`, an empty line and `b`
- **WHEN** the pane renders
- **THEN** the rows show `a`, an empty row and `b`

#### Scenario: A grapheme wider than the pane
- **GIVEN** a pane 1 column wide that holds the line `x日y`
- **WHEN** the pane renders
- **THEN** the rows show `x`, U+FFFD and `y`

### Requirement: The pane follows new lines at the bottom
If the last row of the newest line is on the screen, the pane is at the bottom. At the bottom, a new line SHALL scroll the pane, so the newest line stays on the last row. Away from the bottom, a new line SHALL NOT move the rows on the screen.

#### Scenario: At the bottom
- **GIVEN** a pane 3 rows high that shows the lines `1`, `2` and `3`
- **WHEN** the line `4` arrives
- **THEN** the rows show `2`, `3` and `4`

#### Scenario: Scrolled back
- **GIVEN** a pane 3 rows high that holds the lines `1` to `10` and shows `4`, `5` and `6`
- **WHEN** the line `11` arrives
- **THEN** the rows still show `4`, `5` and `6`

#### Scenario: Scrolled back by one row
- **GIVEN** a pane 3 rows high that holds the lines `1` to `10` and shows `7`, `8` and `9`
- **WHEN** the line `11` arrives
- **THEN** the rows still show `7`, `8` and `9`

#### Scenario: Back at the bottom
- **GIVEN** a pane 3 rows high that holds the lines `1` to `10`
- **AND** the user scrolled it back, then forward until it shows `8`, `9` and `10`
- **WHEN** the line `11` arrives
- **THEN** the rows show `9`, `10` and `11`

### Requirement: A resize keeps the top line
The pane SHALL keep its scroll position as a line and a byte in that line. After a resize, the top row SHALL be the row that holds that byte, in the new wrap. A pane at the bottom SHALL stay at the bottom.

#### Scenario: A narrower pane
- **GIVEN** a pane 40 columns wide, scrolled back so that its top row starts the line `line 50`
- **WHEN** the pane becomes 20 columns wide
- **THEN** the top row starts the line `line 50`

#### Scenario: A resize at the bottom
- **GIVEN** a pane 40 columns wide at the bottom, whose newest line is `the quick brown fox jumps over the lazy dog`
- **WHEN** the pane becomes 20 columns wide
- **THEN** the last row shows `dog`

### Requirement: The pane drops its oldest lines
The pane SHALL keep at most its capacity of lines. Past that capacity, it SHALL drop the oldest lines. A scroll position on a dropped line SHALL move to the oldest line that the pane keeps. A pane with no lines SHALL render an empty area, and a drop of zero lines SHALL change nothing.

#### Scenario: The oldest line goes
- **GIVEN** a pane with a capacity of 1000 lines that holds the lines `1` to `1000`
- **WHEN** the line `1001` arrives
- **THEN** the pane holds the lines `2` to `1001`

#### Scenario: The scroll position was on a dropped line
- **GIVEN** a pane with a capacity of 1000 lines that holds the lines `1` to `1000`
- **AND** its top row starts the line `1`
- **WHEN** the lines `1001` to `1005` arrive
- **THEN** the top row starts the line `6`

#### Scenario: A selection on a dropped line
- **GIVEN** a pane with a capacity of 1000 lines that holds the lines `1` to `1000`
- **AND** a selection from the start of `1` to the end of `2`
- **WHEN** the line `1001` arrives
- **THEN** the copy text is `2`

#### Scenario: A selection with both ends on dropped lines
- **GIVEN** a pane with a capacity of 1000 lines that holds the lines `1` to `1000`
- **AND** a selection from the start of `1` to the end of `2`
- **WHEN** the lines `1001` and `1002` arrive
- **THEN** the pane has no selection

#### Scenario: An empty pane
- **GIVEN** a pane with no lines
- **WHEN** the pane drops its oldest lines, and then drops zero lines
- **THEN** the pane renders an empty area

### Requirement: The scroll bar shows the position
If the pane holds more rows than its area shows, its last column SHALL show a scroll bar. The position of the thumb SHALL be the fraction of the lines above the top row, not the fraction of the rows. If all rows fit, the pane SHALL show no scroll bar.

#### Scenario: Halfway
- **GIVEN** a pane 10 rows high that holds 100 one-row lines, scrolled so that line 50 is the top row
- **WHEN** the pane renders
- **THEN** the thumb of the scroll bar is in the middle rows of the last column

#### Scenario: All rows fit
- **GIVEN** a pane 10 rows high that holds 3 lines
- **WHEN** the pane renders
- **THEN** the last column shows no scroll bar

#### Scenario: Exactly as many rows as the pane
- **GIVEN** a pane 10 rows high that holds 10 one-row lines
- **WHEN** the pane renders
- **THEN** the last column shows no scroll bar

#### Scenario: The scroll bar takes a column
- **GIVEN** a pane 10 columns wide and 2 rows high that holds the lines `aaaaaaaaa` and `bbbbbbbbbbb`
- **WHEN** the pane renders
- **THEN** the last column shows a scroll bar
- **AND** the rows wrap at 9 columns

#### Scenario: Rows that fit only at the full width
- **GIVEN** a pane 10 columns wide and 2 rows high that holds the lines `aaaaaaaaaa` and `b`
- **WHEN** the pane renders twice
- **THEN** both renders show no scroll bar
- **AND** the rows show `aaaaaaaaaa` and `b`

### Requirement: Styles show on their text
The pane SHALL draw each part of a line with the style of that part: the foreground color, the background color, bold, dim, italic, underline, reverse and strikethrough. A style SHALL apply to its text only, in each row that the text wraps onto.

#### Scenario: A red word that wraps
- **GIVEN** a pane 10 columns wide that holds a line whose text `aaaaaaaa redword` has `redword` in red
- **WHEN** the pane renders
- **THEN** the cells of `redword` on the second row are red
- **AND** the cells of `aaaaaaaa` are in the default color

### Requirement: The pane shows one unfinished line
The pane SHALL show an unfinished line as its last row, after the newest line. A new unfinished line SHALL replace it. A new finished line SHALL remove it. The unfinished line SHALL NOT enter the stored lines.

#### Scenario: Replaced by the finished line
- **GIVEN** a pane that shows the unfinished line `hel` as its last row
- **WHEN** the line `hello` arrives
- **THEN** the last row shows `hello`
- **AND** no row shows `hel` on its own

#### Scenario: Replaced by a new unfinished line
- **GIVEN** a pane that shows the unfinished line `Name: `
- **WHEN** the unfinished line `Password: ` arrives
- **THEN** the last row shows `Password: `
- **AND** no row shows `Name: `

### Requirement: A selection copies game lines
A selection SHALL run from one position to another, where a position is a line and a byte in that line. The pane SHALL draw the selected cells in reverse. The copy text of a selection SHALL be the selected text of each line, with one LF between lines, not the wrapped rows. The copy text SHALL hold no style.

#### Scenario: A selection across a wrapped line
- **GIVEN** a pane 10 columns wide that holds the lines `first line here` and `second`
- **WHEN** the user selects from the start of `line` to the end of `second`
- **THEN** the copy text is `line here` LF `second`

#### Scenario: The selection shows
- **GIVEN** a pane with a selection of the word `here`
- **WHEN** the pane renders
- **THEN** the four cells of `here` are in reverse

### Requirement: Each cell maps to a position
Each cell of a text row SHALL map to a position. Each cell of a wide grapheme SHALL map to the start of that grapheme. A cell past the end of the text of a row SHALL map to the end of that row. On the last row of a line, that is the end of the line. A cell of the scroll bar or of the unfinished line SHALL map to no position.

#### Scenario: The second cell of a wide grapheme
- **GIVEN** a pane that holds the line `a日b`
- **WHEN** a test asks for the position of column 3 of that row
- **THEN** the position is byte 1 of the line, the start of `日`

#### Scenario: Past the end of a line
- **GIVEN** a pane 20 columns wide that holds the line `abc`
- **WHEN** a test asks for the position of column 10 of that row
- **THEN** the position is byte 3 of the line

#### Scenario: No position
- **GIVEN** a pane with a scroll bar and the unfinished line `Name: `
- **WHEN** a test asks for the position of a cell of the scroll bar, and of a cell of the unfinished line
- **THEN** neither cell has a position

### Requirement: Cells take the learned width
The pane SHALL give each cell of a grapheme the width that the width profile gives it, when the grapheme is in a risky width class or that width differs from the `unicode-width` value. The pane SHALL use the same width to wrap the line. A profile width of 0 SHALL count as no answer for that class.

#### Scenario: A profile that adds code point widths
- **GIVEN** a width profile that gives the family emoji `👨‍👩‍👧` a width of 8
- **AND** a pane 20 columns wide that holds the line `👨‍👩‍👧x`
- **WHEN** the pane renders
- **THEN** the cell of `x` is at column 9

#### Scenario: The default profile
- **GIVEN** the default width profile
- **AND** a pane that holds the line `👨‍👩‍👧x`
- **WHEN** the pane renders
- **THEN** the cell of `x` is at column 3

### Requirement: An image is a line of its own
An image line SHALL take a fixed number of rows at every width, and the pane SHALL NOT wrap it. The pane SHALL clip its columns at the right edge of the area. With the kitty graphics protocol, each cell of the image SHALL be a kitty unicode placeholder with its row mark and column mark. Its foreground color SHALL be the image id as a 24-bit color. An image SHALL take at least 1 and at most 297 rows and columns.

#### Scenario: An image at two widths
- **GIVEN** a pane that holds an image line of 4 rows and 10 columns
- **WHEN** the pane renders at 40 columns and then at 20 columns
- **THEN** the image takes 4 rows at each width

#### Scenario: Placeholder cells
- **GIVEN** a pane in a terminal with the kitty graphics protocol, and an image line with the id 7
- **WHEN** the pane renders
- **THEN** the cell in row 2, column 3 of the image is a placeholder
- **AND** that placeholder has the row mark of row 1 and the column mark of column 2
- **AND** its foreground color is the id 7

#### Scenario: An image id above 255
- **GIVEN** a pane in a terminal with the kitty graphics protocol, and an image line with the id 256
- **WHEN** the pane renders
- **THEN** the foreground color of each placeholder is red 0, green 1 and blue 0

#### Scenario: The size limits of an image
- **GIVEN** images of 297 rows, of 298 rows, of 400 rows, of 298 columns and of 0 rows and 0 columns
- **WHEN** the pane takes each one as a line
- **THEN** the image lines take 297, 297 and 297 rows, 297 columns, and 1 row and 1 column

#### Scenario: An image wider than the pane
- **GIVEN** a pane 5 columns wide that holds an image line of 2 rows and 10 columns
- **WHEN** the pane renders
- **THEN** each of the 2 rows holds 5 placeholder cells, the columns 0 to 4 of the image

### Requirement: An image falls back to its URL
An image line SHALL hold the URL of its image. In a terminal without the kitty graphics protocol, the pane SHALL show an image line as one text row that holds the URL. The URL SHALL show each control character as U+FFFD. The copy text of an image line SHALL be the URL in every terminal.

#### Scenario: No kitty graphics
- **GIVEN** a pane in a terminal without the kitty graphics protocol
- **AND** an image line of 4 rows and 10 columns with the URL `https://example.com/map.png`
- **WHEN** the pane renders
- **THEN** the image line takes one row that shows `https://example.com/map.png`

#### Scenario: A URL with control characters
- **GIVEN** a pane in a terminal without the kitty graphics protocol, and an image line with the URL `https://x/` ESC `[2J` BEL
- **WHEN** the pane renders
- **THEN** the row shows `https://x/` U+FFFD `[2J` U+FFFD

#### Scenario: The copy of an image
- **GIVEN** a pane in a terminal with the kitty graphics protocol, and a selection over an image line with the URL `https://example.com/map.png`
- **WHEN** the user copies the selection
- **THEN** the copy text is `https://example.com/map.png`
