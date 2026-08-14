use std::{
    env, fs,
    io::{self, Read as _, Write as _},
    path::PathBuf,
    process::ExitCode,
};

use aedicule::{
    ControlPhase, DEFAULT_PLUGIN_ENV, Event, FALLBACK_WAT, Frontplane, Limits, PluginInit,
    PluginSource, TextPhase, TouchContactPhase, TouchContactTracker,
    display_refresh_rate_from_environment, initialize_frontplane, render_svg, text_value_events,
};

const DEFAULT_WIDTH: f32 = 1024.0;
const DEFAULT_HEIGHT: f32 = 768.0;
const DEFAULT_SEED: u64 = 0x5eed_cafe;
const MAX_TICKS_PER_FUEL_SLICE: u64 = 60;

const HELP: &str = "\
Render a deterministic WAT application frame without opening a window.

Usage:
  aedicule-render [PLUGIN.wat|APPLICATION_DIRECTORY|APPLICATION.aed|-|@stdin] [options]

Options:
  --ticks N          Advance N fixed simulation ticks before rendering
  --control ID=VALUE Apply an exact declared integer control; repeatable
  --text ID=VALUE    Commit a declared text field's complete value; repeatable
  --touch PHASE,ID,X,Y
                     Deliver an ordered raw touch; repeatable
  --advance N        Advance N ticks at this point in the touch timeline
  --activate-action ID
                     Activate an exact currently placed guest action; repeatable
  --activate-link ID Validate/report a current external-link request; repeatable
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
    texts: Vec<TextArgument>,
    timeline: Vec<TimelineStep>,
    activate_actions: Vec<u32>,
    activate_links: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ControlArgument {
    id: u32,
    value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TouchArgument {
    phase: TouchContactPhase,
    id: u32,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimelineStep {
    Touch(TouchArgument),
    Advance(u64),
}

/// One complete headless text-field value. It is retained as text only so the
/// adapter-shared expansion can turn it into the exact scalar sequence a native
/// or browser text field emits; no bytes cross the guest boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TextArgument {
    id: u32,
    value: String,
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
            texts: Vec::new(),
            timeline: Vec::new(),
            activate_actions: Vec::new(),
            activate_links: Vec::new(),
        }
    }
}

enum Action {
    Help,
    About,
    Render(RenderOptions),
}

enum RenderInput {
    Text(String),
    Application(PluginSource),
}

fn main() -> ExitCode {
    if cfg!(debug_assertions) && env::var_os("MUTE_DEBUG_STATUS").is_none() {
        eprintln!("\x1b[33mDEBUG BUILD!\x1b[0m");
    }

    let action = match parse_arguments(env::args().skip(1)) {
        Ok(action) => action,
        Err(error) => {
            eprintln!("aedicule-render: {error}");
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
                "aedicule-render {} — deterministic WAT frame renderer for {} {}",
                env!("CARGO_PKG_VERSION"),
                env::consts::OS,
                env::consts::ARCH,
            );
            ExitCode::SUCCESS
        }
        Action::Render(options) => match run(options) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("aedicule-render: {error}");
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
            "--text" => {
                let value = next_value(&mut arguments, "--text")?;
                options.texts.push(parse_text(&value)?);
            }
            "--touch" => {
                let value = next_value(&mut arguments, "--touch")?;
                options
                    .timeline
                    .push(TimelineStep::Touch(parse_touch(&value)?));
            }
            "--advance" => {
                let value = next_value(&mut arguments, "--advance")?;
                options.timeline.push(TimelineStep::Advance(
                    value
                        .parse()
                        .map_err(|_| format!("invalid --advance value: {value}"))?,
                ));
            }
            "--activate-action" => {
                let value = next_value(&mut arguments, "--activate-action")?;
                options.activate_actions.push(
                    value
                        .parse()
                        .map_err(|_| format!("invalid --activate-action ID: {value}"))?,
                );
            }
            "--activate-link" => {
                let value = next_value(&mut arguments, "--activate-link")?;
                options.activate_links.push(
                    value
                        .parse()
                        .map_err(|_| format!("invalid --activate-link ID: {value}"))?,
                );
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

/// Splits only at the first `=`, because a text value may legitimately contain
/// further `=` characters and may also be empty, which clears the field.
fn parse_text(value: &str) -> Result<TextArgument, String> {
    let (id, text) = value
        .split_once('=')
        .ok_or_else(|| format!("invalid --text value: {value}; expected ID=VALUE"))?;
    if id.is_empty() {
        return Err(format!("invalid --text value: {value}; expected ID=VALUE"));
    }
    Ok(TextArgument {
        id: id.parse().map_err(|_| format!("invalid --text ID: {id}"))?,
        value: text.to_owned(),
    })
}

/// Parses one identity-bearing contact edge without admitting NaN/infinity,
/// which would otherwise make guest coordinate behavior platform-dependent.
fn parse_touch(value: &str) -> Result<TouchArgument, String> {
    let invalid =
        || format!("invalid --touch value: {value}; expected start|move|end|cancel,ID,X,Y");
    let fields = value.split(',').collect::<Vec<_>>();
    let [phase, id, x, y] = fields.as_slice() else {
        return Err(invalid());
    };
    let phase = match *phase {
        "start" => TouchContactPhase::Start,
        "move" => TouchContactPhase::Move,
        "end" => TouchContactPhase::End,
        "cancel" => TouchContactPhase::Cancel,
        _ => return Err(invalid()),
    };
    let id = id.parse().map_err(|_| invalid())?;
    let x: f32 = x.parse().map_err(|_| invalid())?;
    let y: f32 = y.parse().map_err(|_| invalid())?;
    if !x.is_finite() || !y.is_finite() {
        return Err(invalid());
    }
    Ok(TouchArgument { phase, id, x, y })
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
    let input = read_plugin(options.plugin.as_deref())?;
    let limits = Limits::default();
    let mut frontplane = match input {
        RenderInput::Text(wat) => Frontplane::from_wat(&wat, limits),
        RenderInput::Application(source) => Frontplane::from_application(&source, limits),
    }
    .map_err(|error| error.to_string())?;
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

    for text in options.texts {
        for event in text_value_events(text.id, &text.value, TextPhase::Commit) {
            frontplane.event(event).map_err(|error| error.to_string())?;
        }
    }

    if !options.activate_actions.is_empty() {
        frontplane.render().map_err(|error| error.to_string())?;
        for action_id in options.activate_actions {
            let available = frontplane.ui_snapshot().is_some_and(|snapshot| {
                snapshot
                    .buttons
                    .iter()
                    .any(|button| button.action_id == action_id)
            });
            if !available {
                return Err("unsupported or unavailable capability in action activation".into());
            }
            frontplane
                .event(Event::MenuAction(action_id))
                .map_err(|error| error.to_string())?;
        }
    }

    let has_touch = options
        .timeline
        .iter()
        .any(|step| matches!(step, TimelineStep::Touch(_)));
    let mut contacts = if has_touch {
        let max_contacts = frontplane
            .metadata()
            .touch_max_contacts
            .ok_or_else(|| "unsupported or unavailable touch capability".to_owned())?;
        Some(
            TouchContactTracker::new(max_contacts)
                .ok_or_else(|| "unsupported or unavailable touch capability".to_owned())?,
        )
    } else {
        None
    };
    for step in options.timeline {
        match step {
            TimelineStep::Touch(touch) => {
                if let Some(event) = contacts
                    .as_mut()
                    .expect("touch timeline initialized its contact tracker")
                    .observe(touch.phase, touch.id, touch.x, touch.y)
                {
                    frontplane.event(event).map_err(|error| error.to_string())?;
                }
            }
            TimelineStep::Advance(ticks) => advance_ticks(&mut frontplane, ticks)?,
        }
    }

    advance_ticks(&mut frontplane, options.ticks)?;
    let frame = frontplane.render().map_err(|error| error.to_string())?;
    let revision = frontplane
        .ui_snapshot()
        .map(|snapshot| snapshot.revision)
        .unwrap_or_default();
    let external_links = options
        .activate_links
        .iter()
        .map(|id| frontplane.external_link_request(revision, *id))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let svg = render_svg(&frame, options.width, options.height);
    for request in external_links {
        eprintln!(
            "aedicule-render: external-link revision={} id={} url={}",
            request.revision, request.id, request.url
        );
    }
    write_output(&options.output, svg.as_bytes())
}

/// Advances long scripted spans in bounded fuel slices, matching the final
/// `--ticks` path while allowing contact edges between deterministic spans.
fn advance_ticks(frontplane: &mut Frontplane, mut remaining: u64) -> Result<(), String> {
    let max_ticks_per_call = u64::from(Limits::default().max_ticks_per_call);
    while remaining > 0 {
        let ticks = remaining
            .min(max_ticks_per_call)
            .min(MAX_TICKS_PER_FUEL_SLICE) as u32;
        frontplane.tick(ticks).map_err(|error| error.to_string())?;
        remaining -= u64::from(ticks);
    }
    Ok(())
}

fn read_plugin(path: Option<&str>) -> Result<RenderInput, String> {
    match path {
        None => match env::var_os(DEFAULT_PLUGIN_ENV) {
            Some(path) => Ok(RenderInput::Application(plugin_source_for_path(
                PathBuf::from(path),
            ))),
            None => Ok(RenderInput::Text(FALLBACK_WAT.into())),
        },
        Some("-" | "@stdin") => {
            let mut wat = String::new();
            io::stdin()
                .read_to_string(&mut wat)
                .map_err(|error| format!("read stdin: {error}"))?;
            Ok(RenderInput::Text(wat))
        }
        Some(path) => Ok(RenderInput::Application(plugin_source_for_path(
            PathBuf::from(path),
        ))),
    }
}

/// Classifies a renderer path through the same application-container rules as
/// the GUI CLI so immutable sibling and archived assets remain available.
fn plugin_source_for_path(path: PathBuf) -> PluginSource {
    if path.is_dir() {
        PluginSource::Directory(path)
    } else if path.extension().is_some_and(|extension| extension == "aed") {
        PluginSource::Archive(path)
    } else {
        PluginSource::File(path)
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
