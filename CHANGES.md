
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

### Added on 09.05.2024

- RUSH: Avoid word splitting when words are enclosed in "" or ''
- RUSH: Known bug - Words in quotes are always added to the command as the last (Fixed!)
- RUSH: Fixed an issue that caused shell to panic when just "then" alone was request as a command
- RUSH: Known bug - Shell is not testing if quotes are properly ended or not
- RUSH: Killing child process with ctrl+c no longer kills parent process (Fixed!)
- CMP: Ability to compare with command's stdout, stderr or exit code. It'll also work with variables because the shell itself is responsible for replacing text starting with a dollar sign with a variable contents.


# Release 0.3.1:

### Added on 12.06.2024

- TEST: Renamed back again to 'IF'. Don't know why I named it 'TEST' before. Such a stupid name.

# Release 0.3.2:

### Added on 15.06.2024

- *: Many code improvements, refactorings and cleanup.
- *: Many (long) comments were added to make all the spaghetti less painful to read
- IF: Fixed an issue that caused IF not to detect END keyword

# Release 0.3.3:

### Added on 18.06.2024

- IF: Complete code overhaul, with some syntax changes
- IF: This program is tolerated as completely independent program from the shell
- HELPFUL: split_commands() separates commands not only after keywords requested in args, but also when '\n' char is found
- IF: Add support for ELSE and ELSEIF (again)

# Release 0.3.4:

### Added on 3.07.2024

- LOOP: Add this very simple loop

### Waiting features:

- MATH: Do the math stuff
- WHILE, LOOP, MATCH, FOR: More logic operations

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
