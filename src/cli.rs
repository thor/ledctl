use crate::hid::Led;

#[derive(Debug)]
pub enum LedAction {
    On,
    Off,
    Toggle,
}

#[derive(Debug)]
pub enum Command {
    List,
    Control { led: Led, action: LedAction },
}

#[derive(Debug)]
pub struct ParsedArgs {
    pub command: Command,
    pub device_filter: Option<String>,
    pub count: u32,
}

#[derive(Debug)]
pub enum CliError {
    Usage(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Usage(s) => write!(f, "{}", s),
        }
    }
}

pub fn parse_args(args: &[String]) -> Result<ParsedArgs, CliError> {
    let mut device_filter: Option<String> = None;
    let mut count: u32 = 1;
    let mut positional: Vec<String> = vec![];
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--list" => {
                return Ok(ParsedArgs {
                    command: Command::List,
                    device_filter,
                    count,
                });
            }
            "--device" => {
                i += 1;
                device_filter = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| CliError::Usage("--device requires a value".into()))?,
                );
            }
            "--count" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| CliError::Usage("--count requires a value".into()))?;
                let n: u32 = raw
                    .parse()
                    .map_err(|_| CliError::Usage(format!("invalid count: {}", raw)))?;
                if n == 0 || n > 9999 {
                    return Err(CliError::Usage("--count must be between 1 and 9999".into()));
                }
                count = n;
            }
            other => positional.push(other.to_string()),
        }
        i += 1;
    }

    if positional.len() != 2 {
        return Err(CliError::Usage(
            "Usage: ledctl [--device <name>] <caps|num|scroll> <on|off|toggle> [--count <n>]\n       ledctl --list".into(),
        ));
    }

    let led = match positional[0].as_str() {
        "caps" => Led::Caps,
        "num" => Led::Num,
        "scroll" => Led::Scroll,
        other => {
            return Err(CliError::Usage(format!(
                "unknown LED '{}': use caps, num, or scroll",
                other
            )))
        }
    };

    let action = match positional[1].as_str() {
        "on" => LedAction::On,
        "off" => LedAction::Off,
        "toggle" => LedAction::Toggle,
        other => {
            return Err(CliError::Usage(format!(
                "unknown state '{}': use on, off, or toggle",
                other
            )))
        }
    };

    Ok(ParsedArgs {
        command: Command::Control { led, action },
        device_filter,
        count,
    })
}
