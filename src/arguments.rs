enum TextEditorArguments {
    NoColor = 1 << 0,
    ReadOnly = 1 << 1,
}

#[derive(Debug)]
pub struct ArgsConfig {
    pub filename: Option<String>,
    flags: u32,
}

impl ArgsConfig {
    pub fn new(args: &[String]) -> Result<ArgsConfig, String> {
        let mut config = ArgsConfig {
            filename: None,
            flags: 0,
        };

        for idx in 1..args.len() {
            match args[idx].as_str() {
                "--no-color" => config.flags |= TextEditorArguments::NoColor as u32,
                "--read-only" => config.flags |= TextEditorArguments::ReadOnly as u32,
                arg => {
                    if !arg.starts_with('-') {
                        if config.filename.is_none() {
                            config.filename = Some(arg.to_string());
                        } else {
                            return Err(format!("Multiple filenames specified: '{}'", arg));
                        }
                    } else {
                        return Err(format!("Unknown argument: '{}'", arg));
                    }
                }
            }
        }

        Ok(config)
    }
}
