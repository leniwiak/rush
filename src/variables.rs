use std::env::var_os;
use carrot_libs::system;

pub fn getenv(buf:&[String]) -> Result<String, String> {
    // Check if there is just ONE argument
    // We can't check more than one variable at the same time
    match buf.len() {
        1 => Err(("Give me a variable name to check!").to_string()),
        2 => {
            match var_os(&buf[1]) {
                Some(ret) => {
                    match ret.into_string() {
                        Ok(a) => Ok(a),
                        Err(_) => Err(format!("Error occured while setting up a variable: {}!", buf[1]))
                    }
                },
                None => Err(format!("Variable \"{}\" is not set!", buf[1]))
            }
        },
        _ => Err(("Cannot check multiple variables simultaneously!").to_string()),
    }
}

use std::env::set_var;
pub fn setenv(buf:&[String]) -> Result<(), String> {
    // Check if there is just ONE argument
    // We can't set more than one variable at the same time
    if buf.len() < 2 {
        Err(("Give me a variable name and it's contents to set!").to_string())
    }
    else {
        // Allow user to set variables with proper letters only
        if system::check_simple_characters_compliance(&buf[1]).is_err() {
            return Err(format!("Variable name contains invalid characters: {}!", buf[1]));
        }
        // Value must contain contents of arg 2+
        let mut value = String::new();
        for a in &buf[2..] {
            value.push_str(&format!("{} ", a));
        };

        // trim _end() is going to remove any trailing white characters at the end
        set_var(&buf[1], value.trim_end());
        Ok(())
    }
}

use std::env::remove_var;
pub fn remenv(buf:&[String]) -> Result<(), String> {
    // Check if there is just ONE argument
    // We can't set more than one variable at the same time
    if buf.len() < 2 {
        Err(("Give me a variable name and it's contents to set!").to_string())
    }
    else {
        // Allow user to remove variables with proper letters only
        if system::check_simple_characters_compliance(&buf[1]).is_err() {
            return Err(format!("Variable name contains invalid characters: {}!", buf[1]));
        }
        remove_var(&buf[1]);
        Ok(())
    }
}

pub fn chenv(buf:&[String], increment:bool) -> Result<(), String> {
    let mut set = 1;
    match buf.len() {
        1 => Err(("Give me a variable name to increment!").to_string()),
        2 | 3 => {
            if buf.len() == 3 {
                match buf[2].parse::<isize>() {
                    Err(e) => return Err(format!("Can't parse second argument to a number: {:?}", e.kind())),
                    Ok(a) => set = a,
                }
            }
            // Read variable
            if let Some(ret) =  var_os(&buf[1]) {
                // Convert to a string from OsString
                match ret.into_string() {
                    Err(_) => Err(format!("Error occured while checking a variable: {}!", buf[1])),
                    Ok(ret) => {
                        // Convert it to a number
                        match ret.parse::<isize>() {
                            Ok(a) => {
                                // Increment/decrement and set it up
                                if increment {
                                    set_var(&buf[1], (a+set).to_string());
                                }
                                else {
                                    set_var(&buf[1], (a-set).to_string());
                                }
                                Ok(())
                            }
                            Err(_) => Err(format!("Error occured while converting a variable to a number: {}!", buf[1])),
                        }
                    }
                } 
            }
            else {
               Err(format!("Variable \"{}\" is not set!", buf[1]))
            }
        }
        _ => Err(("Cannot understand more arguments!").to_string())
    }
}