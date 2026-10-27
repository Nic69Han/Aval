# names Specification

## Purpose
Defines how names are resolved inside a module: declared items, parameters, let bindings, imports,
duplicates and reserved names.

## Requirements

### Requirement: Unknown names
Every value or function name SHALL resolve to a parameter, a `let` binding in scope, a lambda parameter, a module item, a dimension variable or a prelude function; otherwise `E101` is reported.

#### Scenario: RES-001 Undeclared variable
- **WHEN** a function `f(x: Int) -> Int` returns `y`
- **THEN** `aval check` reports `E101` with field `name` equal to `y`

#### Scenario: RES-002 Let binding is not visible outside its block
- **WHEN** a function returns `if x > 0 { let y: Int = 1` newline `y } else { y }`
- **THEN** `aval check` reports `E101` for the second `y`

#### Scenario: RES-003 Call to a function declared later
- **WHEN** function `a` calls function `b` declared after it in the same module
- **THEN** `aval check` reports no error

### Requirement: No overloading or duplicates
A module SHALL NOT declare two items with the same name, and a function SHALL NOT have two parameters with the same name.

#### Scenario: RES-004 Duplicate function
- **WHEN** a module declares `fn f() -> Int` twice
- **THEN** `aval check` reports `E102` at the second declaration

#### Scenario: RES-005 Shadowing is forbidden
- **WHEN** a function `f(x: Int) -> Int` contains `let x: Int = 1`
- **THEN** `aval check` reports `E102`

### Requirement: Unresolved imports
In the MVP every `import` SHALL be reported as `E103`, since multi-file resolution is out of scope.

#### Scenario: RES-006 Import of a module
- **WHEN** a module contains `import vision.camera { Camera }`
- **THEN** `aval check` reports `E103` with field `module` equal to `vision.camera`

### Requirement: Reserved type name O
A type, enum or resource SHALL NOT be named `O`.

#### Scenario: RES-007 Type named O
- **WHEN** a module declares `type O { v: Int }`
- **THEN** `aval check` reports `E105`

### Requirement: Prelude functions cannot be redefined
A module SHALL NOT declare a function, type or variant whose name is a prelude function (`approx`, `print`) or a prelude type or variant (`Ok`, `Err`); this is `E102`.

#### Scenario: RES-008 Redefining approx
- **WHEN** a module declares `fn approx(a: Float, b: Float) -> Bool` returning `true`
- **THEN** `aval check` reports `E102`

#### Scenario: RES-009 Type named Ok
- **WHEN** a module declares `type Ok { v: Int }`
- **THEN** `aval check` reports `E102`
