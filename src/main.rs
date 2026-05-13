use ledctl::cli::{parse_args, Command, LedAction};
use ledctl::hid::{HidSession, LedError};
use ledctl::ui;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        ui::run();
        return;
    }
    if let Err(e) = run_cli(&args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_args(args).map_err(|e| e.to_string())?;
    let session = HidSession::new()?;

    match parsed.command {
        Command::List => {
            let keyboards = session.keyboards();
            let to_show: Vec<_> = match &parsed.device_filter {
                None => keyboards.iter().collect(),
                Some(filter) => {
                    let f = filter.to_lowercase();
                    keyboards.iter().filter(|kb| kb.name.to_lowercase().contains(&f)).collect()
                }
            };
            for kb in to_show {
                println!("[{:#010x}] {}", kb.location_id, kb.name);
            }
        }
        Command::Control { led, action } => {
            let all = session.keyboards();

            let targets: Vec<usize> = match &parsed.device_filter {
                None => (0..all.len()).collect(),
                Some(filter) => {
                    let f = filter.to_lowercase();
                    let matches: Vec<usize> = all
                        .iter()
                        .enumerate()
                        .filter(|(_, kb)| kb.name.to_lowercase().contains(&f))
                        .map(|(i, _)| i)
                        .collect();
                    if matches.is_empty() {
                        return Err(LedError::DeviceNotFound.to_string().into());
                    }
                    if matches.len() > 1 {
                        let names: Vec<&str> =
                            matches.iter().map(|&i| all[i].name.as_str()).collect();
                        return Err(format!(
                            "ambiguous --device filter matches multiple keyboards: {}",
                            names.join(", ")
                        )
                        .into());
                    }
                    matches
                }
            };

            for idx in targets {
                let kb = &all[idx];
                match action {
                    LedAction::On => session.set_led(kb, led, true)?,
                    LedAction::Off => session.set_led(kb, led, false)?,
                    LedAction::Toggle => session.toggle_led(kb, led, parsed.count)?,
                }
            }
        }
    }
    Ok(())
}
