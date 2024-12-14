use crate::global::{index, set_index};

pub fn logic(mut buf: Vec<String>) -> Result<bool, String> {
    /*
    if FAIL:thing1 -with-arg -with-another-arg and OK:thing2 or OUT:thing3 and $variable == 10 do
        say "I'm a bunch of words on your screen"
    end

    Possible keyword types are:
    OK - Execute a command name with 0 it's exit code is 0 or 1 if anything else
    FAIL - Reversed version of OK
    CODE - Replace command name
    OUT - Replace command with it's stdout output
    ERR - Replace command with it's stderr output
    LOGIC - AND/OR/DO
    COMPARATOR - ==, <, >, >=, etc.
    VAR - Replace variable with it's contents
    NUMVAL - Raw number
    TXTVAL - Raw text
    OKVAL - Raw boolean
    
    1. Make a hash map of words from IF/ELSEIF/ELSE until DO.
       Key is a type of the thing and value is the... value.
        > wordlist = [FAIL:thing1 -with-arg -with-another-arg, LOGIC:AND, OK:thing2, LOGIC:OR, OUT:thing3, LOGIC:AND, VAR:variable, COMPARATOR:==, NUMVAR:10, LOGIC:DO]
    
    2. Iterate through wordlist

    3. Look up for the first word in a list. It may be anything but LOGIC and COMPARATOR.
       We do not accept LOGICs nor COMPARATORs at this moment because we want to prevent the user from typing something
       like "if and thing1 do ..." or "if == thing1 and thing2 do ..."

    4. At this point, we need a left comparable object.
       This has to be a key of type OK, FAIL, CODE, OUT, ERR, VAL, OKVAL, NUMVAL or TXTVAL.
       No LOGIC nor COMPARATOR is allowed.
       If you find a key of type OK:
        > Execute the command with all of it's arguments. Quit from IF block if it does not exist.
        > Replace it's name depending on it's exit code.
            If command succeeded (returned exit code 0), set it to [OKVAL:1]
            If command failed (returned any other exit code), save it as [OKVAL:0]
       If you find a key of type FAIL:
        > Do everything like in the case of OK but in reverse.
       If you find a key of type CODE, OUT or ERR:
        > Execute the command with all of it's arguments. Quit from IF block if it does not exist.
        > Replace it's name with the value returned from stdout, stderr or an exit code.
        > Check if it's a number or a text
        > Replace it.
            So now something like "[CODE:thing1 -with-arg -with-another-arg]" may change to "[NUMVAL:0]"
            and something like "[OUT:thing1 -with-arg -with-another-arg]" may change to "[TXTVAL:Output from thing1]"
       If you find a key of type VAR:
        > Replace it's value with variable contents
        > Check if it's a boolean, number or a text
        > Replace it like in example below.
            If variable is set to 10, "[VAR:variable]" will became "[NUMVAL:10]"
            if variable is set to "hello", "[VAR:variable]" will became "[TXTVAL:hello]"
            if variable is set to TRUE, "[VAR:variable]" will became "[OKVAL:1]"
            if variable is set to FALSE, "[VAR:variable]" will became "[OKVAL:0]"
            BUT if it's set to "TRUE" or "FALSE", it will became "[TXTVAL:TRUE]" or "[TXTVAL:FALSE]"
       Leave every OKVAL, NUMVAL and TXTVAL as is.
    5. After exactly one CODE/OUT/ERR/VAL was translated to OKVAL, NUMVAL or TXTVAL,
       a COMPARATOR or LOGIC is required.
       If you find a [LOGIC:DO]:
        > Check if previous block is of type OKVAL
        > Add to a global script iteration index a value which is an index of DO. (Do smth like INDEX += position-of-do NOT INDEX = position_of_do).
        > Set shell_mode as CmpSuccess or CmpFailure based on the value from OKVAL 
        > End this IF statement
       If you find a [LOGIC:AND] or [LOGIC:OR]:
        > IF statement can't be finished yet. Report the need for another comparable thing.
        > Check if previous block is of type OKVAL
        > At this point, just set if_operation_mode to AND or OR
       If you find a COMPARATOR:
        > IF statement can't be finished yet. Report the need for another comparable thing.
        > Check if previous block is of type NUMVAL or TXTVAL
        > At this point, just set if_operation_mode to...
          == and =        EQUAL (acceptable for both types)
          !=, != and !    DIFFERENT (acceptable for both types)
          <               LESS (only for NUMVALs)
          =< or <=        LESS_OR_EQUAL (only for NUMVALs)
          >               GREATER (only for NUMVALs)
          >= or =>        GREATER_OR_EQUAL (only for NUMVALs)
          ~~, =~, ~=, ~   CONTAINS (only for TXTVALs)
    6. After exactly one COMPARATOR or LOGIC (other than DO), another OK, FAIL, CODE, OUT, ERR, VAL, NUMVAL or TXTVAL is needed
    7. Repeat step 4 to parse right comparable object of type OK, FAIL, CODE, OUT, ERR, VAL, NUMVAL or TXTVAL.
    
    8. This was a right comparable object in an LEFT_CMD LOGIC/COMPARATOR RIGHT_CMD block.
       Now, check type of this and this-2 wordlist element
       and allow comparing them depending on if_operation_mode value.
       > if_operation_mode is AND / OR: Both elements must be of type OKVAL
       > if_operation_mode is EQUAL / DIFFERENT:  Both elements must be of type TXTVAL or NUMVAL
       > if_operation_mode is LESS, LESS_OR..., GREATER, GREATER_OR...: Both elements must be of type NUMVAL
       > if_operation_mode is CONTAINS: Both elements must be of type TXTVAL
       In other case, just skip comparison.
    10. Replace those three elements with OKVAL:1 if...
       > if_operation_mode = AND and both comparable objects are set to OKVAL:1
       > if_operation_mode = OR and at least one comparable object is set to OKVAL:1
       > if_operation_mode is EQUAL/DIFFERENT/LESS/etc. and they match in type and value
       Otherwise, replace them with OKVAL:0
    11. Iterate through wordlist again
    */
    //let a = index();
    for w in buf {
        println!("{w}");
    }
    Ok(true)
}