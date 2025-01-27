
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
- IF: Fixed bug that allowed user to write empty comparison statements

### Added on 4.07.2024

NOTE: Import system of the Rust languge is heavly broken!
I can't import rush::detect_commands() nowhere, because it is a binary, so when I try to move detect_commands() to "helpful" library,
compiler screams at me because functions used in detect_commands() can't be found (obviously, I have to "mod" them first)
But using "mod" break EVERYTHING. I can't use "mod" in helpful, because helpful itself is "mod"ded in Rush which is a binary.
What is the purpose of doing it like that? Can anyone from the Rust community explain it to me?

For now, I've moved everything that detect_commands() depends on from other files to "helpful".
Now I have very large file that is VERY uncomfortable to read...
Thanks Rust.

- RUSH: Fixed bug, that allowed user to use empty variables in commands
- *: Drop usage of "CommandsStatus" struct

# Release 0.3.5:

### Added on 9.07.2024

- RUSH: Fixed a bug that prevented user from writing a line with multiple commands that set and check some variable, like "set IDX=1 next echo $IDX"

# Release 0.3.6:

### Added on 10.07.2024

- LOOP: Breaking out of loop under special conditions **should** work
- Add "~" shortcut to access user's home directory path

# Release 0.3.7:

### Added on 11.07.2024

- RUSH: Added $varname++ and $varname-- syntax to quickly increase/decrease a variable's value
- RUSH: Code responsible for removing quotes from enquoted sentences was moved to the detect_commands() function. User does not need to see the quoted on the output - but the shell needs it. If word is enquoted, special characters like ~, $, ++ or -- won't be resolved.

# Release 0.3.8:

### Added on 12.07.2024

- CMP: Use BigInt instead of usize
- CMP: Support for negative numbers
- GLUE: Ability to join arguments

### Added on 5.12.2024

- UNSET: Remove a variable

### Added on 7.12.2024

- CD: Check current working directory

### Waiting features:

- MATH: Do the math stuff
- FOR: More logic operations

# Release 0.4.0:

- SAY: Print text to the terminal (NEW PROGRAM!)
- NSAY: Print text to the terminal (without new line character at the end) (NEW PROGRAM!)
- RUSH: Aliases
- RUSH: History storing

# Release 0.5.0:

### Added on 14.12.2024

- Prepare for a massive code rework
- Rename CD to DIR
- Drop alias and history support (temporarily!)
- LOCK: Endless loops (NEW!)
- Theoretical comments support

### Added on 20.12.2024

- Bugfixes to lock
- Working FREE and CONTINUE keywords inside locks
- Drop variable names resolution (temporarily!)
- Use comma to separate commands
- Use semicolon to end blocks

### Added on 21.12.2024

- Change DEC and INC command names to "--" and "++"

### Added on 22.12.2024

- Bugfixes for nested locks

### Added on 23.12.2024

- Interrupts (SIGINT from CTRL+C) are working as they should (probably)

# Release 0.5.1:

### Added on 19.01.2025

- Group words inside quotationmarks

### Added on 20.01.2025

- Support for unescaped quotemarks
- With support for quotes, I can now add commit comments on git!
- Remove unneeded code
- Add very barebone syntax checker

### Added on 21.01.2025
- Disallow running empty commands like this: say hello, , say bye

# Release 0.5.2

### Added 27.01.2025

- Slowly working on IF implementation: It can sort arguments by keyword types

### Waiting features:

- Implement IF logic
- History browsing on CLI
- Variable names resolution

# Release unknown:

- RUSH: Restricted shell mode
- RUSH: Functions
- RUSH: Pipes
- RUSH: Ability to redirect command's output to a file
- RUSH: Arrays, dictionaries
