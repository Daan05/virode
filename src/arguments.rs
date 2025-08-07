enum TextEditorArguments {
    NoColor = 1 << 0,
    ReadOnly = 1 << 1,
}

#[derive(Debug)]
pub struct ArgsConfig {
    pub file_name: String,
    _flags: u32,
}

impl ArgsConfig {
    pub fn new(args: &[String]) -> Result<ArgsConfig, String> {
        let mut file_name = String::from("");
        let mut flags = 0;

        for idx in 1..args.len() {
            match args[idx].as_str() {
                "--no-color" => flags |= TextEditorArguments::NoColor as u32,
                "--read-only" => flags |= TextEditorArguments::ReadOnly as u32,
                arg => {
                    if !arg.starts_with('-') {
                        if file_name.is_empty() {
                            file_name = arg.to_string();
                        } else {
                            return Err(format!("Multiple filenames specified: '{}'", arg));
                        }
                    } else {
                        return Err(format!("Unknown argument: '{}'", arg));
                    }
                }
            }
        }

        if file_name.is_empty() {
            return Err(String::from("No filename was passed as argument"));
        }

        Ok(ArgsConfig {
            file_name: file_name,
            _flags: flags,
        })
    }
}
