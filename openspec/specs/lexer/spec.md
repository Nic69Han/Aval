# lexer Specification

## Purpose
Defines how Aval source text is split into tokens: reserved words, identifiers, literals, comments and
significant newlines, and the lexical errors reported to agents.

## Requirements

### Requirement: Reserved words
The lexer SHALL recognise exactly the 46 reserved words of the reference (§2) as keywords, and no other word.

#### Scenario: LEX-001 Keywords cannot be used as identifiers
- **WHEN** a module declares `fn match(x: Int) -> Int`
- **THEN** `aval check` reports `E002` at the position of `match`

#### Scenario: LEX-002 Excluded words are ordinary identifiers
- **WHEN** a module binds `let while: Int = 1` inside a function body and returns `while`
- **THEN** `aval check` reports no error

### Requirement: Identifier case
The lexer SHALL classify `[a-z_][a-z0-9_]*` as value identifiers and `[A-Z][A-Za-z0-9]*` as type identifiers.

#### Scenario: LEX-003 PascalCase name in value position of a let
- **WHEN** a function body contains `let Total: Int = 1`
- **THEN** `aval check` reports `E002` with `expected` containing `ident`

### Requirement: Comments
The lexer SHALL treat `#` to end of line as a comment.

#### Scenario: LEX-004 Comments are ignored
- **WHEN** a valid module has a comment line between two functions and a trailing comment after a `let`
- **THEN** `aval check` reports no error

### Requirement: Literals
The lexer SHALL accept integer, float (digits on both sides of the dot, optional exponent), string (escapes `\" \\ \n \t`), and duration literals (`ns`, `us`, `ms`, `s`).

#### Scenario: LEX-005 Float literal requires digits after the dot
- **WHEN** a function returns `2.`
- **THEN** `aval check` reports `E001`

#### Scenario: LEX-006 Unterminated string
- **WHEN** a function returns `"abc` with no closing quote
- **THEN** `aval check` reports `E001` at the opening quote

#### Scenario: LEX-007 Duration literal in latency
- **WHEN** a function declares `latency < 500us`
- **THEN** `aval check` reports no error

### Requirement: Invalid characters
The lexer SHALL report `E001` for any character outside the token set.

#### Scenario: LEX-008 Stray character
- **WHEN** a function body contains `return 1 @ 2`
- **THEN** `aval check` reports `E001` at the column of `@`

### Requirement: Newline significance
The lexer SHALL ignore newlines inside parentheses and brackets and after `{`, `,`, `=>`, `->`, `=` and binary operators.

#### Scenario: LEX-009 Expression split after an operator
- **WHEN** a function returns `1 +` followed by a newline and `2`
- **THEN** `aval check` reports no error and the function returns `3`

#### Scenario: LEX-010 List literal split across lines
- **WHEN** an `examples` line contains a list literal spread over three lines inside `[ ]`
- **THEN** `aval check` reports no error
