
# Release 0.2.0:

### Added on 24.03.2024

- RUSH: Migrated to standalone project tree
- IF: Kind of working IF logic

# Release 0.2.1:

### Added on 29.03.2024

- RUSH: ELSE command

### Added on 01.04.2024

- *: Break code to multiple parts
- IF: Rename to 'TEST'

### Added on 02.04.2024

- UNTIL: Wait for keypress
- SLEEP: Add new command that will stop command execution for some time
- GETENV, SETENV: Ability to view and modify simple variables
- RUSH: Deal with char boundaries (The bug was caused by dependency: carrot-libs/libinput!)

# Release 0.3.0:

### Added on 02.04.2024

- RUSH: More supported shortcuts due to the changes made in libinput in carrot-libs
- CMP: Ability to compare text and numeric values.
- RUSH: Initial support for "$varname" syntax

### Added on 04.04.2024

- RUSH: Support for keywords starting with "$"

### Added on 09.04.2024

- RUSH: Avoid word splitting when words are enclosed in "" or ''
- RUSH: Known bug - Words in quotes are always added to the command as the last
- RUSH: Known bug - Shell is not testing if quotes are properly ended or not
- CMP: Ability to compare with command's stdout, stderr or exit code. It'll also work with variables because the shell itself is responsible for replacing text starting with a dollar sign with a variable contents.

### Waiting features:

- MATH: Do the math stuff
- WHILE: LOOP, MATCH, FOR: More logic operations
- BREAK: Quit from the loop

# Release 0.4.0:

### Waiting features:

- RUSH: Aliases
- RUSH: History storing and browsing
- RANDOM: Generate random numbers
- SAY: Print text to the terminal

# Release unknown:

- RUSH: Restricted shell mode
- RUSH: Run files (scripts) passed as arguments from CLI
- RUSH: Comments
- RUSH: Functions
- RUSH: Pipes
- RUSH: Ability to redirect command's output to a file
- RUSH: Arrays, dictionaries
