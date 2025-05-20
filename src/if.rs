use crate::global;
use crate::global::print_err;
use std::collections::HashMap;
use std::env;
use std::process;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
enum DataType {
    Ok,
    Fail,
    Code,
    Out,
    Err,
    Logic,
    Comparator,
    Var,
    Numval,
    Txtval,
    Okval,
}

pub fn logic(mut buf: Vec<String>) -> Result<bool, String> {
    /*
    if FAIL:thing1 -with-arg -with-another-arg and OK:thing2 or OUT:"thing3 and a word that should not be treated as a keyword" and $variable == 10;
        say "I'm a bunch of words on your screen"
    endif

    Possible keyword types are:
    OK - Execute a command name with 0 it's exit code is 0 or 1 if anything else
    FAIL - Reversed version of OK
    CODE - Replace command name
    OUT - Replace command with it's stdout output
    ERR - Replace command with it's stderr output
    LOGIC - AND/OR
    COMPARATOR - ==, <, >, >=, etc.
    VAR - Replace variable with it's contents
    NUMVAL - Raw number
    TXTVAL - Raw text
    OKVAL - Raw boolean

    1. Make a hash map of words from IF/ELSEIF/ELSE until ";""
       Key is a type of the thing and value is the... value.
        > wordlist = [FAIL:thing1 -with-arg -with-another-arg, LOGIC:AND, OK:thing2, LOGIC:OR, OUT:thing3 and a word that should not be treated as a keyword, LOGIC:AND, VAR:variable, COMPARATOR:==, NUMVAR:10]

    2. Iterate through wordlist

    3. Look up for the first word in a list. It may be anything but LOGIC and COMPARATOR.
       We do not accept LOGICs nor COMPARATORs at this moment because we want to prevent the user from typing something
       like "if and thing1 ... ;" or "if == thing1 and thing2 ... ;"

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
            TODO: BUT if it's set to "TRUE" or "FALSE", it will became "[TXTVAL:TRUE]" or "[TXTVAL:FALSE]"
       Leave every OKVAL, NUMVAL and TXTVAL as is.
    5. After exactly one CODE/OUT/ERR/VAL was translated to OKVAL, NUMVAL or TXTVAL,
       a COMPARATOR or LOGIC is required.
       If you reach the end of an IF comparison statement ";":
        > Check if previous block is of type OKVAL
        > There must be just one OKVAL block
        > Add to a global script iteration index a value which is an index of this IF. (Do smth like INDEX += position-of-if NOT INDEX = position_of_if).
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
          !=, =! and !    DIFFERENT (acceptable for both types)
          <               LESS (only for NUMVALs)
          =< or <=        LESS_OR_EQUAL (only for NUMVALs)
          >               GREATER (only for NUMVALs)
          >= or =>        GREATER_OR_EQUAL (only for NUMVALs)
          ~~, =~, ~=, ~   CONTAINS (only for TXTVALs)
          *-              STARTS_WITH (only for TXTVALs)
          -*              ENDS_WITH (only for TXTVALs)
    6. After exactly one COMPARATOR or LOGIC, another OK, FAIL, CODE, OUT, ERR, VAL, NUMVAL or TXTVAL is needed
    7. Repeat step 4 to parse right comparable object of type OK, FAIL, CODE, OUT, ERR, VAL, NUMVAL or TXTVAL.

    8. This was a right comparable object in an LEFT_CMD LOGIC/COMPARATOR RIGHT_CMD block.
       Now, check type of this and this 2-word list.
       and allow comparing them depending on if_operation_mode value.
       > if_operation_mode is AND / OR - Both elements must be of type OKVAL
       > if_operation_mode is EQUAL / DIFFERENT -  Both elements must be of type TXTVAL or NUMVAL
       > if_operation_mode is LESS, LESS_OR..., GREATER, GREATER_OR... - Both elements must be of type NUMVAL
       > if_operation_mode is CONTAINS - Both elements must be of type TXTVAL
       In other case, just skip comparison.
    10. Replace those three elements with OKVAL:1 if...
       > if_operation_mode = AND and both comparable objects are set to OKVAL:1
       > if_operation_mode = OR and at least one comparable object is set to OKVAL:1
       > if_operation_mode is EQUAL/DIFFERENT/LESS/etc. and they match in type and value
       Otherwise, replace them with OKVAL:0
    11. Iterate through wordlist again
    */

    /*
    If statement accepts arguments with more than one word.
    But if this is the case, we have to understand where some words belong.
    For example, if you aproach a word like "door". What you should do with it?
    It isn't a command. It's a plain text value. But can we be sure about it?
    A word "door" can be a part of a command, like in this example:
    if OK:command_that_accepts_args door do ... ;

    The best way to achieve this, is to join all of the words together.
    For example, word "door" after "OK:command_that_accepts_args" must be "glued up" with a command
    To avoid confusion, TXTVALs also have to be joined into one thing.
     */
    let mut normalized_buf = Vec::new();

    let mut append_to_last_word_instead_buf = false;
    // Iterate through words except the first one which is just "IF"

    // Remove first word in a buffer.
    // Starting IF/ELSEIF/ELSE is useless here
    buf.remove(0);

    /*
    Join any words after CODE:program_name.
    This allows the user to just type the command they want to execute like this: CODE:something blah blah
    instead of wraping the command in quotation marks like this: CODE:"funny command here"
    */
    for word in &buf {
        if word.to_uppercase().starts_with("OK:")
            || word.to_uppercase().starts_with("FAIL:")
            || word.to_uppercase().starts_with("CODE:")
            || word.to_uppercase().starts_with("OUT:")
            || word.to_uppercase().starts_with("ERR:")
        {
            // Push first word to the buffer
            normalized_buf.push(word.to_string());
            // Allow appending unresolved keywords to the word instead of the buffer itself
            append_to_last_word_instead_buf = true;
        }
        // Just append known keywords to the buffer of commands
        else if word.to_uppercase() == "AND"
            || word.to_uppercase() == "OR"
            || word == "=="
            || word == "="
            || word == "!"
            || word == "!="
            || word == "=!"
            || word == "=<"
            || word == "=>"
            || word == "<="
            || word == ">="
            || word == "<"
            || word == ">"
            || word == "~"
            || word == "~~"
            || word == "~="
            || word == "=~"
            || word == "*-"
            || word == "-*"
            // Allow reffering to variables
            || word.starts_with('$')
            // Allow numbers too!
            || word.parse::<usize>().is_ok()
            // Allow words starting with single/double quotation marks
            || word.starts_with('\'')
            || word.starts_with('"')
        {
            normalized_buf.push(word.to_string());
            append_to_last_word_instead_buf = false;
        }
        // If you approach unknown word
        else {
            // Append it to the last word in the buffer instead of the buffer itself
            // if it is a part of CODE:, OUT: or ERR: statement
            if append_to_last_word_instead_buf {
                normalized_buf
                    .last_mut()
                    .unwrap()
                    .push_str(format!(" {}", word).as_str());
            } else {
                return Err(format!("Unknown keyword: {word}"));
            }
        }

        // Find common errors
        if word.to_uppercase().trim() == "OK:"
            || word.to_uppercase().trim() == "FAIL:"
            || word.to_uppercase().trim() == "CODE:"
            || word.to_uppercase().trim() == "OUT:"
            || word.to_uppercase().trim() == "ERR:"
        {
            return Err(format!(
                "Used command referer \"{}\" without specifying a command to run",
                word
            ));
        }
    }

    // Make a list of all known IF arguments that is easier to understand from the
    // program's maintainer perspective :DDD
    let mut big_mommy = Vec::new();
    for (i, w) in buf.clone().into_iter().enumerate() {
        if w.to_uppercase().starts_with("OK:") {
            big_mommy.push((
                DataType::Ok,
                buf[i].strip_prefix("OK:").unwrap().to_string(),
            ));
        } else if w.to_uppercase().starts_with("FAIL:") {
            big_mommy.push((
                DataType::Fail,
                buf[i].strip_prefix("FAIL:").unwrap().to_string(),
            ));
        } else if w.to_uppercase().starts_with("CODE:") {
            big_mommy.push((
                DataType::Code,
                buf[i].strip_prefix("CODE:").unwrap().to_string(),
            ));
        } else if w.to_uppercase().starts_with("OUT:") {
            big_mommy.push((
                DataType::Out,
                buf[i].strip_prefix("OUT:").unwrap().to_string(),
            ));
        } else if w.to_uppercase().starts_with("ERR:") {
            big_mommy.push((
                DataType::Err,
                buf[i].strip_prefix("ERR:").unwrap().to_string(),
            ));
        } else if w.starts_with('$') {
            big_mommy.push((DataType::Var, w.strip_prefix('$').unwrap().to_string()));
        } else if w.to_uppercase() == "AND" {
            big_mommy.push((DataType::Logic, String::from("AND")));
        } else if w.to_uppercase() == "OR" {
            big_mommy.push((DataType::Logic, String::from("OR")));
        } else if w == "==" || w == "=" {
            big_mommy.push((DataType::Comparator, String::from("EQUAL")));
        } else if w == "!=" || w == "=!" || w == "!" {
            big_mommy.push((DataType::Comparator, String::from("DIFFERENT")));
        } else if w == "<" {
            big_mommy.push((DataType::Comparator, String::from("LESS")));
        } else if w == "=<" || w == "<=" {
            big_mommy.push((DataType::Comparator, String::from("LESS_OR_EQUAL")));
        } else if w == ">" {
            big_mommy.push((DataType::Comparator, String::from("GREATER")));
        } else if w == ">=" || w == "=>" {
            big_mommy.push((DataType::Comparator, String::from("GREATER_OR_EQUAL")));
        } else if w == "~~" || w == "~" || w == "~=" || w == "=~" {
            big_mommy.push((DataType::Comparator, String::from("CONTAINS")));
        } else if w == "*-" {
            big_mommy.push((DataType::Comparator, String::from("STARTS_WITH")));
        } else if w == "-*" {
            big_mommy.push((DataType::Comparator, String::from("ENDS_WITH")));
        } else {
            let is_num = w.parse::<usize>().is_ok();
            if is_num {
                big_mommy.push((DataType::Numval, w));
            } else {
                big_mommy.push((DataType::Txtval, w));
            }
        }
    }

    // Remove unescaped quotation marks
    let mut idx = 0;
    while idx < big_mommy.len() {
        big_mommy[idx].1 = global::remove_quotationmarks(&big_mommy[idx].1);
        idx += 1;
    }

    // Basically, after every thing to compare, there must be a comparator.
    // Check for the syntax before we do anything fancy.
    let mut idx = 0;
    while idx < big_mommy.len() {
        let dataunit = big_mommy[idx].clone();

        if idx % 2 == 0
            && (matches!(dataunit.0, DataType::Logic) | matches!(dataunit.0, DataType::Comparator))
        {
            return Err(format!(
                "Expected a thing to compare in place of \"{}\"",
                dataunit.1
            ));
        } else if idx % 2 != 0
            && !matches!(dataunit.0, DataType::Logic)
            && !matches!(dataunit.0, DataType::Comparator)
        {
            return Err(format!(
                "Expected a comparator in place of \"{}\"",
                dataunit.1
            ));
        } else {
            //println!("{}", dataunit.1);
        }
        idx += 1;
    }

    // Detect missing arguments after last comparator
    // Example erroneous code: if 'poland' *- 'pol' and 1 = ;
    if big_mommy.len() % 2 != 1 {
        return Err("Expected a thing to compare at the end".to_string());
    }

    /*
    Here comes the fancy stuff!
    Run commands inside "if comparison" and collect their exit codes.
    Also, resolve variables to numvals or txtvals.
    
    That should be it for now.
    */
    loop {
        /*
        Always resolve commands from first and third elements in comparison statement.
        If some of these elements are of type OUT or ERR - convert them to TXTVALs first.
        If some of these are of type CODE - convert them to NUMVALs first.
        If there is any environment variable reference - convert them to TXT/NUM vals too.

        And also, do nothing if it's just a usual NUMVAL, TXTVAL or an OKVAL.
        */

        // Start with the first one as we are sure, that it is there.
        match ref_to_value(big_mommy[0].clone()) {
            Ok(res) => {
                big_mommy.remove(0);
                big_mommy.insert(0, res);
            },
            Err(e) => return Err(e),
        };
        
        // If there are no other comparison statements, just quit.
        if big_mommy.len() < 2 {
            break
        };
        // Any code below is going to assume that the third element in big_mommy
        // (which should be a second thing to compare. For instance: A number, text or a command)
        // exists.

        // Start with the first one as we are sure, that it is there.
        match ref_to_value(big_mommy[2].clone()) {
            Ok(res) => {
                big_mommy.remove(2);
                big_mommy.insert(2, res);
            },
            Err(e) => return Err(e),
        };

        // Collect all the required info for future work
        let first_value = big_mommy[0].clone();
        let first_value_type = first_value.0;
        let first_value_content = first_value.1;
        let first_value_is_true = first_value_content == "1";

        let this_comparator = big_mommy[1].clone();
        let this_comparator_content = this_comparator.1.to_uppercase();

        let second_value = big_mommy[2].clone();
        let second_value_type = second_value.0;
        let second_value_content = second_value.1;
        let second_value_is_true = big_mommy[2].1 == "1";

        // If types does not match - throw an error
        if first_value_type != second_value_type {
            return Err("Types of values to compare do not match!".to_string());
        }

        let replace_group_with;
        
        // Compare two OKVALs separated by some AND/OR logic keyword
        if matches!(first_value_type, DataType::Okval)
            && (this_comparator_content == "AND" || this_comparator_content == "OR")
        {
            let this_comparison_is_true = this_comparator_content == "AND";

            replace_group_with = match (
                first_value_is_true,
                this_comparison_is_true,
                second_value_is_true,
            ) {
                // AND
                (true, true, true) => 1,
                // OR
                (true, false, true) | (false, false, true) | (true, false, false) => 1,
                _ => 0,
            }
        }

        // Compare two NUMVALs
        else if matches!(first_value_type, DataType::Numval) {
            let first_value_content = first_value_content.parse::<usize>().unwrap();
            let second_value_content = second_value_content.parse::<usize>().unwrap();

            replace_group_with = match this_comparator_content.as_str() {
                "EQUAL" => (first_value_content == second_value_content).into(),
                "DIFFERENT" => (first_value_content != second_value_content).into(),
                "LESS" => (first_value_content < second_value_content).into(),
                "LESS_OR_EQUAL" => (first_value_content <= second_value_content).into(),
                "GREATER" => (first_value_content > second_value_content).into(),
                "GREATER_OR_EQUAL" => (first_value_content >= second_value_content).into(),
                _ => {
                    return Err(format!(
                        "Untolerable comparator for number values: \"{}\"",
                        this_comparator_content
                    ))
                }
            }
        }

        // Compare two TXTVALs
        else if matches!(first_value_type, DataType::Txtval) {
            replace_group_with = match this_comparator_content.as_str() {
                "EQUAL" => (first_value_content == second_value_content).into(),
                "DIFFERENT" => (first_value_content != second_value_content).into(),
                "CONTAINS" => (first_value_content.contains(&second_value_content)).into(),
                "STARTS_WITH" => (first_value_content.starts_with(&second_value_content)).into(),
                "ENDS_WITH" => (first_value_content.ends_with(&second_value_content)).into(),
                _ => {
                    return Err(format!(
                        "Untolerable comparator for text values: \"{}\"",
                        this_comparator_content
                    ))
                }
            }
        } else {
            dbg!(
                &big_mommy,
                idx,
                &big_mommy[0],
                &big_mommy[1],
                &big_mommy[2]
            );
            unreachable!("Program's logic contradics itself! Please, report this error to maintainers!\nDon't forget to share all of the debugging information above.");
        };

        // Remove three first elements in IF logic memory
        big_mommy.remove(2);
        big_mommy.remove(1);
        big_mommy.remove(0);
        // Insert whatever you got from previous match operation
        big_mommy.insert(0, (DataType::Okval, replace_group_with.to_string()));
    }

    // After we're done with the loop
    // check for proper type of the only one value inside of our if comparison statement
    if big_mommy.len() == 1 && !matches!(big_mommy[0].0, DataType::Okval) {
        dbg!(&big_mommy);
        unreachable!("Program's logic contradicts itself! Please, report this error to maintainers!\nDon't forget to share all of the debugging information above.")
    }

    // And check if it is set to 1
    let result_is_true = big_mommy[0].1 == "1";
    dbg!(&big_mommy);

    Ok(result_is_true)
}


fn ref_to_value(big_mommy_element: (DataType, String)) -> Result<(DataType, String), String> {
    let cmd = big_mommy_element.1.split_whitespace().collect::<Vec<&str>>();
    let cmdname = cmd[0];
    let cmdargs = cmd.iter().skip(1);

    match big_mommy_element.0 {
        // If the thing's type is OK
        DataType::Ok => {
            // Run a command and collect it's exit status
            let exit_code = process::Command::new(cmdname)
                .args(cmdargs)
                .status()
                .unwrap();
            // If the command has been ran, append a value of type OKVAL to the list of
            // IF's collection of logics
            if let Some(code) = exit_code.code() {
                // Is exit code a zero? Then append OKVAL:SUCCESS to the list
                if code == 0 {
                    Ok((DataType::Okval, 1.to_string()))
                }
                // or else, append a failure
                else {
                    Ok((DataType::Okval, 0.to_string()))
                }
            } else {
                // No command? No bitches.
                Err(format!("An error occured on command \"{}\"", cmdname))
            }
        }
        // This code is the exact same thing as the code above, but with reversed returns
        DataType::Fail => {
            let exit_code = process::Command::new(cmdname)
                .args(cmdargs)
                .status()
                .unwrap();
            
            if let Some(code) = exit_code.code() {
                if code == 0 {
                    Ok((DataType::Okval, 0.to_string()))
                } else {
                    Ok((DataType::Okval, 1.to_string()))
                }
            } else {
                Err(format!("An error occured on command \"{}\"", cmdname))
            }
        }
        DataType::Code => {
            let exit_code = process::Command::new(cmdname)
                .args(cmdargs)
                .status()
                .unwrap();
            if let Some(code) = exit_code.code() {
                Ok((DataType::Numval, code.to_string()))
            } else {
                Err(format!("An error occured on command \"{}\"", cmdname))
            }
        }
        DataType::Var => {
            let variable = env::var(&big_mommy_element.1);
            if let Ok(v) = variable {
                let num = v.parse::<usize>();
                if let Ok(result) = num {
                    Ok((DataType::Numval, result.to_string()))
                } else if v == "TRUE" {
                    Ok((DataType::Okval, 1.to_string()))
                } else if v == "FALSE" {
                    Ok((DataType::Okval, 0.to_string()))
                } else {
                    Ok((DataType::Txtval, v.to_string()))
                }
            } else {
                Err(format!("Variable \"{}\" is undefined", &big_mommy_element.1))
            }
        }
        _ => Ok(big_mommy_element),
    }
}