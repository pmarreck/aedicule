use std::{
    env, fs,
    io::{self, Read as _, Write as _},
    process::ExitCode,
};

use gpui_wasm::{
    ControlPhase, DEFAULT_PLUGIN_ENV, Event, FALLBACK_WAT, Frontplane, Limits, PluginInit,
    display_refresh_rate_from_environment, initialize_frontplane, render_svg,
};

const DEFAULT_WIDTH: f32 = 1024.0;
const DEFAULT_HEIGHT: f32 = 768.0;
const DEFAULT_SEED: u64 = 0x5eed_cafe;
const MAX_TICKS_PER_FUEL_SLICE: u64 = 60;

const HELP: &str = "\
Render a deterministic WAT application frame without opening a window.

Usage:
  gpui-wasm-render [PLUGIN.wat|-|@stdin] [options]

Options:
  --ticks N          Advance N fixed simulation ticks before rendering
  --control ID=VALUE Apply an exact declared integer control; repeatable
  --seed N           Initialize the plugin with deterministic seed N
  --width N          Logical viewport width (default: 1024)
  --height N         Logical viewport height (default: 768)
  -o, --output PATH  Write SVG to PATH, -/@stdout, or @stderr (default: -)
  -h, --help         Show this help
  --about            Show version and build platform

Environment:
  AE_DISPLAY_REFRESH_RATE  Exact initial display rate, e.g. 60000/1001 or 59.94
";

#[derive(Debug, PartialEq)]
struct RenderOptions {
    plugin: Option<String>,
    output: String,
    ticks: u64,
    seed: u64,
    width: f32,
    height: f32,
    controls: Vec<ControlArgument>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ControlArgument {
    id: u32,
    value: i32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            plugin: None,
            output: "-".into(),
            ticks: 0,
            seed: DEFAULT_SEED,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            controls: Vec::new(),
        }
    }
}

enum Action {
    Help,
    About,
    Render(RenderOptions),
}

fn main() -> ExitCode {
    if cfg!(debug_assertions) && env::var_os("MUTE_DEBUG_STATUS").is_none() {
        eprintln!("\x1b[33mDEBUG BUILD!\x1b[0m");
    }

    let action = match parse_arguments(env::args().skip(1)) {
        Ok(action) => action,
        Err(error) => {
            eprintln!("gpui-wasm-render: {error}");
            return ExitCode::from(2);
        }
    };
    match action {
        Action::Help => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Action::About => {
            println!(
                "gpui-wasm-render {} — deterministic WAT frame renderer for {} {}",
                env!("CARGO_PKG_VERSION"),
                env::consts::OS,
                env::consts::ARCH,
            );
            ExitCode::SUCCESS
        }
        Action::Render(options) => match run(options) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("gpui-wasm-render: {error}");
                ExitCode::FAILURE
            }
        },
    }
}

/// Parses order-independent switches with later values overriding earlier
/// values, keeping the CLI useful for both human shells and generated commands.
fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Action, String> {
    let mut options = RenderOptions::default();
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => return Ok(Action::Help),
            "--about" => return Ok(Action::About),
            "--ticks" => {
                let value = next_value(&mut arguments, "--ticks")?;
                options.ticks = value
                    .parse()
                    .map_err(|_| format!("invalid --ticks value: {value}"))?;
            }
            "--control" => {
                let value = next_value(&mut arguments, "--control")?;
                options.controls.push(parse_control(&value)?);
            }
            "--seed" => {
                let value = next_value(&mut arguments, "--seed")?;
                options.seed = value
                    .parse()
                    .map_err(|_| format!("invalid --seed value: {value}"))?;
            }
            "--width" => {
                let value = next_value(&mut arguments, "--width")?;
                options.width = positive_number("--width", &value)?;
            }
            "--height" => {
                let value = next_value(&mut arguments, "--height")?;
                options.height = positive_number("--height", &value)?;
            }
            "-o" | "--output" => options.output = next_value(&mut arguments, "--output")?,
            "--" => {
                while let Some(path) = arguments.next() {
                    set_plugin(&mut options, path)?;
                }
            }
            _ if argument.starts_with('-') && argument != "-" => {
                return Err(format!("unknown option: {argument}"));
            }
            _ => set_plugin(&mut options, argument)?,
        }
    }
    Ok(Action::Render(options))
}

fn next_value(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn positive_number(option: &str, value: &str) -> Result<f32, String> {
    let number: f32 = value
        .parse()
        .map_err(|_| format!("invalid {option} value: {value}"))?;
    if number.is_finite() && number > 0.0 {
        Ok(number)
    } else {
        Err(format!("invalid {option} value: {value}"))
    }
}

fn parse_control(value: &str) -> Result<ControlArgument, String> {
    let (id, value_number) = value
        .split_once('=')
        .ok_or_else(|| format!("invalid --control value: {value}; expected ID=VALUE"))?;
    if id.is_empty() || value_number.is_empty() || value_number.contains('=') {
        return Err(format!(
            "invalid --control value: {value}; expected ID=VALUE"
        ));
    }
    Ok(ControlArgument {
        id: id
            .parse()
            .map_err(|_| format!("invalid --control ID: {id}"))?,
        value: value_number
            .parse()
            .map_err(|_| format!("invalid --control VALUE: {value_number}"))?,
    })
}

fn set_plugin(options: &mut RenderOptions, path: String) -> Result<(), String> {
    if options.plugin.is_some() {
        return Err("only one plugin input may be specified".into());
    }
    options.plugin = Some(path);
    Ok(())
}

/// Executes guest lifecycle calls headlessly and emits the same immutable
/// frame-command buffer consumed by the GPUI adapter.
fn run(options: RenderOptions) -> Result<(), String> {
    let mut plugin_init = PluginInit::new(options.seed, options.width, options.height);
    if let Some(display_refresh) =
        display_refresh_rate_from_environment().map_err(|error| error.to_string())?
    {
        plugin_init = plugin_init.with_display_refresh(display_refresh);
    }
    let wat = read_plugin(options.plugin.as_deref())?;
    let limits = Limits::default();
    let max_ticks_per_call = u64::from(limits.max_ticks_per_call);
    let mut frontplane = Frontplane::from_wat(&wat, limits).map_err(|error| error.to_string())?;
    initialize_frontplane(&mut frontplane, plugin_init).map_err(|error| error.to_string())?;

    for control in options.controls {
        frontplane
            .event(Event::Control {
                id: control.id,
                value: control.value,
                phase: ControlPhase::Release,
            })
            .map_err(|error| error.to_string())?;
    }

    let mut remaining = options.ticks;
    while remaining > 0 {
        let ticks = remaining
            .min(max_ticks_per_call)
            .min(MAX_TICKS_PER_FUEL_SLICE) as u32;
        frontplane.tick(ticks).map_err(|error| error.to_string())?;
        remaining -= u64::from(ticks);
    }
    let frame = frontplane.render().map_err(|error| error.to_string())?;
    let svg = render_svg(&frame, options.width, options.height);
    write_output(&options.output, svg.as_bytes())
}

fn read_plugin(path: Option<&str>) -> Result<String, String> {
    match path {
        None => match env::var_os(DEFAULT_PLUGIN_ENV) {
            Some(path) => fs::read_to_string(&path)
                .map_err(|error| format!("read {}: {error}", path.to_string_lossy())),
            None => Ok(FALLBACK_WAT.into()),
        },
        Some("-" | "@stdin") => {
            let mut wat = String::new();
            io::stdin()
                .read_to_string(&mut wat)
                .map_err(|error| format!("read stdin: {error}"))?;
            Ok(wat)
        }
        Some(path) => fs::read_to_string(path).map_err(|error| format!("read {path}: {error}")),
    }
}

fn write_output(path: &str, bytes: &[u8]) -> Result<(), String> {
    match path {
        "-" | "@stdout" => io::stdout()
            .lock()
            .write_all(bytes)
            .map_err(|error| format!("write stdout: {error}")),
        "@stderr" => io::stderr()
            .lock()
            .write_all(bytes)
            .map_err(|error| format!("write stderr: {error}")),
        path => fs::write(path, bytes).map_err(|error| format!("write {path}: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, RenderOptions, parse_arguments};

    #[test]
    fn later_switch_values_override_earlier_values() {
        let arguments = [
            "--ticks", "10", "--width", "640", "--ticks", "20", "--width", "800",
        ]
        .into_iter()
        .map(str::to_owned);

        let Action::Render(options) = parse_arguments(arguments).unwrap() else {
            panic!("expected render action");
        };
        assert_eq!(
            options,
            RenderOptions {
                ticks: 20,
                width: 800.0,
                ..RenderOptions::default()
            }
        );
    }
}
