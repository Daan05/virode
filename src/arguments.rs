enum TextEditorArguments {
    NoColor = 1 << 0,
    ReadOnly = 1 << 1,
}

#[derive(Debug)]
pub struct ArgsConfig {
    pub file_name: String,
    flags: u32,
}

impl ArgsConfig {
    pub fn new(args: &[String]) -> Result<ArgsConfig, String> {
        let mut config = ArgsConfig {
            file_name: String::from(""),
            flags: 0,
        };

        for idx in 1..args.len() {
            match args[idx].as_str() {
                "--no-color" => config.flags |= TextEditorArguments::NoColor as u32,
                "--read-only" => config.flags |= TextEditorArguments::ReadOnly as u32,
                arg => {
                    if !arg.starts_with('-') {
                        if config.file_name.is_empty() {
                            config.file_name = arg.to_string();
                        } else {
                            return Err(format!("Multiple filenames specified: '{}'", arg));
                        }
                    } else {
                        return Err(format!("Unknown argument: '{}'", arg));
                    }
                }
            }
        }

        if config.file_name.is_empty() {
            return Err(String::from("No filename was passed as argument"));
        }

        Ok(config)
    }
}

