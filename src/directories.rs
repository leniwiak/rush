use std::env;

// Change working directory
pub fn gt(buf: &[String]) -> Result<(), String> {
    // Check if there is just ONE argument
    if buf.len() == 1 {
        Err(("Give me a directory path to go!").to_string())
    }
    // We can't go to more than one directory at the same time
    else if buf.len() > 2 {
        Err(("Cannot go to multiple directories simultaneously!").to_string())
    } else {
        return match env::set_current_dir(&buf[1]) {
            Err(e) => Err(format!(
                "{}: Cannot go into this directory because of an error: {}",
                buf[1],
                e.kind()
            )),
            Ok(_) => Ok(()),
        };
    }
}
