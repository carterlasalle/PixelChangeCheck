//! The interactive front door: `pcc` with no arguments, or `pcc menu`.
//!
//! Every path through this module ends in an argv that goes through the
//! same clap parser and the same dispatch as a hand-typed command. The menu
//! is a front end for the flags, not a second implementation of them, so it
//! cannot drift from what `pcc <command>` actually does.
//!
//! It also closes a gap the flags alone leave open: a `pcc://view?...`
//! invite can be pasted whole, because both long opaque strings in it (the
//! token, and a pin that is the *relay's* in one form and the *sharer's* in
//! the other) are easy to transcribe wrongly and impossible to tell apart
//! once you have.
//!
//! Presentation is opt-out, not opt-in: [`Style`] emits escape codes only
//! when the stream is a terminal, `NO_COLOR` is unset and the terminal is
//! not `dumb`. Piping the menu therefore produces clean, greppable text —
//! which the smoke suite relies on.

use crate::network::SessionToken;
use anyhow::Result;
use std::io::{BufRead, IsTerminal, Write};

/// What the caller should do once the interview finishes.
pub enum Outcome {
    /// Nothing to run: the user chose to quit, or the request needs no
    /// process (a browser link, for instance).
    Quit,
    /// Arguments for the CLI, without the program name.
    Run(Vec<String>),
}

/// Input ended before the interview did.
///
/// `pcc menu < /dev/null` must not fall through to the first option and
/// start sharing the screen, so every prompt propagates this instead.
#[derive(Debug)]
struct Eof;

impl std::fmt::Display for Eof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "input ended before the menu finished")
    }
}

impl std::error::Error for Eof {}

fn eof<T>() -> Result<T> {
    Err(Eof.into())
}

// ------------------------------------------------------------------ style

/// Whether to emit ANSI escape codes.
///
/// A one-field struct rather than a bool so the call sites read as
/// `s.bold()` and cannot be silently transposed with any other flag.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    color: bool,
}

impl Style {
    /// Never emit escape codes. What tests and any non-terminal caller use.
    pub const PLAIN: Self = Self { color: false };

    /// Colour when this is a real terminal and the environment has not
    /// asked for plain output. `NO_COLOR` is the widely-implemented opt-out
    /// (https://no-color.org); `TERM=dumb` is the older convention for a
    /// terminal that cannot interpret escapes.
    pub fn auto() -> Self {
        let opted_out = std::env::var_os("NO_COLOR").is_some()
            || std::env::var("TERM").is_ok_and(|t| t == "dumb");
        Self {
            color: std::io::stdout().is_terminal() && !opted_out,
        }
    }

    fn code(self, code: &'static str) -> &'static str {
        if self.color {
            code
        } else {
            ""
        }
    }

    fn bold(self) -> &'static str {
        self.code("\x1b[1m")
    }

    fn dim(self) -> &'static str {
        self.code("\x1b[2m")
    }

    fn reset(self) -> &'static str {
        self.code("\x1b[0m")
    }

    /// The brand colour: bold bright cyan.
    fn brand(self) -> &'static str {
        self.code("\x1b[1;96m")
    }

    /// A heading or a question.
    fn head(self) -> &'static str {
        self.code("\x1b[1;97m")
    }

    /// The thing the user is about to run.
    fn good(self) -> &'static str {
        self.code("\x1b[1;32m")
    }

    /// A caveat worth reading.
    fn warn(self) -> &'static str {
        self.code("\x1b[1;33m")
    }

    /// A refusal.
    fn bad(self) -> &'static str {
        self.code("\x1b[1;31m")
    }

    /// Wrap `text` in `code`, resetting after. No-op when colour is off.
    fn paint(self, code: &str, text: &str) -> String {
        if self.color {
            format!("{code}{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

/// The printed width of `s`, ignoring ANSI escape sequences.
///
/// Box drawing pads to the widest line, and a styled line is longer than it
/// looks; measuring the raw bytes would make every panel ragged.
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip to the end of the sequence. All we emit are SGR
            // sequences, which terminate with 'm'.
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            len += 1;
        }
    }
    len
}

/// A framed panel: a title rule, body lines, and a closing rule.
///
/// Body lines may carry styling; `visible_len` keeps the right edge
/// straight regardless.
fn panel<W: Write>(out: &mut W, s: Style, title: &str, body: &[String]) {
    let title_len = visible_len(title) + 4;
    let inner = body
        .iter()
        .map(|l| visible_len(l) + 2)
        .chain(std::iter::once(title_len))
        .max()
        .unwrap_or(0)
        .max(40);

    let frame = |c: &str, n: usize| s.paint(s.dim(), &c.repeat(n));
    let _ = writeln!(
        out,
        "\n  {}{}─ {}{} {}",
        frame("╭", 1),
        frame("─", 1),
        s.paint(s.brand(), title),
        frame("─", inner.saturating_sub(title_len)),
        frame("╮", 1)
    );
    for l in body {
        let pad = inner.saturating_sub(visible_len(l) + 2);
        let _ = writeln!(
            out,
            "  {} {}{} {}",
            frame("│", 1),
            l,
            " ".repeat(pad),
            frame("│", 1)
        );
    }
    let _ = writeln!(out, "  {}{}", frame("╰", 1), frame("─", inner + 1));
}

// ------------------------------------------------------------------- run

/// Run the interview. `input`/`output` are injected so this is testable
/// without a terminal; `style` decides whether escape codes are emitted.
pub fn run<R: BufRead, W: Write>(input: &mut R, out: &mut W, style: Style) -> Result<Outcome> {
    let result = interview(input, out, style);
    match result {
        Err(e) if e.downcast_ref::<Eof>().is_some() => Ok(Outcome::Quit),
        other => other,
    }
}

fn interview<R: BufRead, W: Write>(input: &mut R, out: &mut W, s: Style) -> Result<Outcome> {
    banner(out, s);
    let choice = choose(
        input,
        out,
        s,
        "What would you like to do?",
        &[
            ("Share this screen", "you are the host"),
            ("View someone else's screen", "you are the viewer"),
            ("Run a relay", "for two people who cannot reach each other"),
            (
                "Check this machine",
                "what it can capture, whether audio is available",
            ),
            ("Show every command", ""),
        ],
        1,
        true,
    )?;

    let Some(choice) = choice else {
        return Ok(Outcome::Quit);
    };

    match choice {
        0 => share(input, out, s),
        1 => view(input, out, s),
        2 => relay(input, out, s),
        3 => diagnose(input, out, s),
        _ => Ok(Outcome::Run(vec!["help".to_string()])),
    }
}

fn banner<W: Write>(out: &mut W, s: Style) {
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {}▀▀█ █▀▀ █▀▀{}   {}PixelChangeCheck{}",
        s.brand(),
        s.reset(),
        s.bold(),
        s.reset()
    );
    let _ = writeln!(
        out,
        "  {}  █ █▀▀ █▀▀{}   {}exact, lossless screen sharing{}",
        s.brand(),
        s.reset(),
        s.dim(),
        s.reset()
    );
    let _ = writeln!(out, "  {}{}{}", s.dim(), "─".repeat(58), s.reset());
    let _ = writeln!(
        out,
        "  {}Answer the questions and this prints — and can run — the exact{}",
        s.dim(),
        s.reset()
    );
    let _ = writeln!(
        out,
        "{}command. Press Enter to accept a [default].{}",
        s.dim(),
        s.reset()
    );
    let _ = writeln!(out);
}

// ---------------------------------------------------------------- prompts

/// Read one line. `None` means input ended.
fn line<R: BufRead>(input: &mut R) -> Option<String> {
    let mut buf = String::new();
    match input.read_line(&mut buf) {
        Ok(0) => None,
        Ok(_) => Some(buf.trim().to_string()),
        Err(_) => None,
    }
}

fn ask<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    s: Style,
    prompt: &str,
    default: &str,
) -> Result<String> {
    // The caret marks where typing happens; without it the prompt and the
    // hint run together on a busy line.
    let caret = s.paint(s.brand(), "›");
    if default.is_empty() {
        let _ = write!(out, "  {}{}{} {caret} ", s.bold(), prompt, s.reset());
    } else {
        let _ = write!(
            out,
            "  {}{}{} {}[{default}]{} {caret} ",
            s.bold(),
            prompt,
            s.reset(),
            s.dim(),
            s.reset()
        );
    }
    let _ = out.flush();
    match line(input) {
        None => eof(),
        Some(v) if v.is_empty() => Ok(default.to_string()),
        Some(v) => Ok(v),
    }
}

/// Like [`ask`], but an empty answer is meaningful (`None`), not a default.
fn ask_optional<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    s: Style,
    prompt: &str,
) -> Result<Option<String>> {
    let _ = write!(
        out,
        "  {}{}{} {}— blank to skip —{} {} ",
        s.bold(),
        prompt,
        s.reset(),
        s.dim(),
        s.reset(),
        s.paint(s.brand(), "›")
    );
    let _ = out.flush();
    match line(input) {
        None => eof(),
        Some(v) if v.is_empty() => Ok(None),
        Some(v) => Ok(Some(v)),
    }
}

/// A numbered menu. Returns the 0-based index, or `None` for quit when
/// `allow_quit` is set.
fn choose<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    s: Style,
    question: &str,
    options: &[(&str, &str)],
    default: usize,
    allow_quit: bool,
) -> Result<Option<usize>> {
    let _ = writeln!(out);
    let _ = writeln!(out, "  {}{}{}", s.head(), question, s.reset());
    let width = options
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let bar = s.paint(s.dim(), "│");
    for (i, (label, hint)) in options.iter().enumerate() {
        let chosen = i + 1 == default;
        let marker = if chosen {
            s.paint(s.brand(), "❯")
        } else {
            " ".to_string()
        };
        let number = s.paint(
            if chosen { s.brand() } else { s.dim() },
            &format!("{}", i + 1),
        );
        let text = if chosen {
            s.paint(s.bold(), &format!("{label:<width$}"))
        } else {
            format!("{label:<width$}")
        };
        let tail = if hint.is_empty() {
            String::new()
        } else {
            format!("  {}", s.paint(s.dim(), hint))
        };
        let _ = writeln!(out, "   {marker} {number} {bar} {text}{tail}");
    }
    if allow_quit {
        let _ = writeln!(
            out,
            "     {} {} {}",
            s.paint(s.dim(), "q"),
            bar,
            s.paint(s.dim(), "quit")
        );
    }
    let _ = write!(
        out,
        "\n  {}Choose{} {}{default}{} {} ",
        s.dim(),
        s.reset(),
        s.bold(),
        s.reset(),
        s.paint(s.brand(), "›")
    );
    let _ = out.flush();

    match line(input) {
        None => eof(),
        Some(v) if v.is_empty() => Ok(Some(default - 1)),
        Some(v) if allow_quit && (v == "q" || v == "quit") => Ok(None),
        Some(v) => match v.parse::<usize>() {
            Ok(n) if n >= 1 && n <= options.len() => Ok(Some(n - 1)),
            _ => {
                let _ = writeln!(
                    out,
                    "\n  {}That is not one of the numbers above — starting over.{}",
                    s.warn(),
                    s.reset()
                );
                choose(input, out, s, question, options, default, allow_quit)
            }
        },
    }
}

fn confirm<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    s: Style,
    question: &str,
    default_yes: bool,
) -> Result<bool> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    let _ = write!(
        out,
        "  {}{}{} {}{}{} {} ",
        s.bold(),
        question,
        s.reset(),
        s.dim(),
        suffix,
        s.reset(),
        s.paint(s.brand(), "›")
    );
    let _ = out.flush();
    match line(input) {
        None => eof(),
        Some(v) if v.is_empty() => Ok(default_yes),
        Some(v) => Ok(matches!(v.to_ascii_lowercase().as_str(), "y" | "yes")),
    }
}

/// A caveat that has cost people a round trip before.
fn note<W: Write>(out: &mut W, s: Style, text: &str) {
    let _ = writeln!(out);
    for (i, l) in text.lines().enumerate() {
        let lead = if i == 0 {
            s.paint(s.warn(), "!")
        } else {
            " ".to_string()
        };
        let _ = writeln!(out, "  {lead} {}", s.paint(s.warn(), l));
    }
}

fn hint<W: Write>(out: &mut W, s: Style, text: &str) {
    for l in text.lines() {
        let _ = writeln!(out, "  {}{}{}", s.dim(), l, s.reset());
    }
}

// ------------------------------------------------------------------ share

fn share<R: BufRead, W: Write>(input: &mut R, out: &mut W, s: Style) -> Result<Outcome> {
    let target = choose(
        input,
        out,
        s,
        "What do you want to share?",
        &[
            ("The whole screen", "the primary display"),
            ("A specific display", ""),
            ("A region of the screen", "e.g. 100,200,1280,720"),
            ("A window", "matched by its title"),
            ("An application", "matched by its name"),
        ],
        1,
        true,
    )?;
    let Some(target) = target else {
        return Ok(Outcome::Quit);
    };

    let mut args: Vec<String> = vec!["share".into()];
    match target {
        0 => {} // the primary display is the default; no flag needed
        1 => {
            let n = ask(
                input,
                out,
                s,
                "Display number (see `pcc diagnose --displays`)",
                "0",
            )?;
            args.push("--display".into());
            args.push(n);
        }
        2 => {
            let r = ask(input, out, s, "Region as x,y,w,h", "100,200,1280,720")?;
            if r.is_empty() {
                let _ = writeln!(
                    out,
                    "  {}A region is required — stopping.{}",
                    s.bad(),
                    s.reset()
                );
                return Ok(Outcome::Quit);
            }
            args.push("--region".into());
            args.push(r);
        }
        3 => {
            let t = ask(input, out, s, "Part of the window title", "")?;
            if t.is_empty() {
                let _ = writeln!(
                    out,
                    "  {}A title is required — stopping.{}",
                    s.bad(),
                    s.reset()
                );
                return Ok(Outcome::Quit);
            }
            args.push("--window".into());
            args.push(t);
        }
        _ => {
            let a = ask(input, out, s, "Application name", "")?;
            if a.is_empty() {
                let _ = writeln!(
                    out,
                    "  {}A name is required — stopping.{}",
                    s.bad(),
                    s.reset()
                );
                return Ok(Outcome::Quit);
            }
            args.push("--application".into());
            args.push(a);
        }
    }

    // ---- how viewers reach you ----
    let route = choose(
        input,
        out,
        s,
        "How will viewers connect?",
        &[
            ("Directly", "same network or VPN — nothing to set up"),
            (
                "Through a relay",
                "works over the internet, no port forwarding",
            ),
            ("Both", "direct when possible, relay as the fallback"),
        ],
        1,
        true,
    )?;
    let Some(route) = route else {
        return Ok(Outcome::Quit);
    };

    let mut relay_addr = None;
    let mut relay_pin = None;
    let mut relay_token = None;
    let mut session = String::new();
    if route >= 1 {
        let _ = writeln!(out);
        hint(
            out,
            s,
            "A relay prints one line when it starts. Pasting that whole line\n\
             fills in the address, the pin and the token at once — the token is\n\
             the value nobody can guess, and the usual reason a session is\n\
             refused with `bad credential`.\n\
             No relay yet? Quit, run `pcc` → \"Run a relay\" in another terminal,\n\
             then paste the line it prints into this one.",
        );
        // A remembered relay removes the paste entirely — that is the whole
        // point of the store.
        let saved = crate::relays::load().unwrap_or_default();
        if !saved.is_empty() {
            let mut opts: Vec<(&str, &str)> = saved
                .iter()
                .map(|(name, r)| (name.as_str(), r.addr.as_str()))
                .collect();
            opts.push(("some other relay", "paste or type its details"));
            match choose(input, out, s, "Which relay?", &opts, 1, true)? {
                Some(i) if i < saved.len() => {
                    let r = saved
                        .iter()
                        .nth(i)
                        .map(|(_, v)| v.clone())
                        .expect("index came from the same map");
                    let _ = writeln!(
                        out,
                        "  {}using saved relay {}{}{} — nothing to paste{}",
                        s.good(),
                        s.bold(),
                        r.addr,
                        s.reset(),
                        s.reset()
                    );
                    relay_addr = Some(r.addr);
                    relay_pin = Some(r.pin);
                    relay_token = Some(r.token);
                }
                Some(_) => {}
                None => return Ok(Outcome::Quit),
            }
        }

        let pasted = if relay_addr.is_none() {
            ask_optional(
                input,
                out,
                s,
                "Paste the relay's line (blank to type the values instead)",
            )?
        } else {
            None
        };
        if let Some(text) = pasted {
            match crate::reach::parse_relay_hint(&text) {
                Ok(h) => {
                    let _ = writeln!(
                        out,
                        "  {}got it — relay {}{}{}",
                        s.good(),
                        s.bold(),
                        h.relay,
                        s.reset()
                    );
                    relay_addr = Some(h.relay);
                    relay_pin = Some(h.pin);
                    relay_token = Some(h.token);
                }
                Err(e) => {
                    let _ = writeln!(
                        out,
                        "  {}That line did not parse: {e}{}",
                        s.warn(),
                        s.reset()
                    );
                    let _ = writeln!(
                        out,
                        "  {}Answer the questions below instead.{}",
                        s.dim(),
                        s.reset()
                    );
                }
            }
        }

        if relay_addr.is_none() {
            let addr = ask(input, out, s, "Relay address", "127.0.0.1:5900")?;
            if addr.is_empty() {
                let _ = writeln!(
                    out,
                    "  {}A relay address is required — stopping.{}",
                    s.bad(),
                    s.reset()
                );
                return Ok(Outcome::Quit);
            }
            let pin = match ask_optional(
                input,
                out,
                s,
                "Relay pin (64 hex characters; blank fetches it from the relay)",
            )? {
                Some(p) => p,
                None => match fetch_relay_pin(&addr) {
                    Ok(p) => {
                        let _ = writeln!(out, "  {}fetched {p}{}", s.good(), s.reset());
                        let _ = writeln!(
                            out,
                            "  {}Compare it against whatever the relay operator published.{}",
                            s.dim(),
                            s.reset()
                        );
                        p
                    }
                    Err(e) => {
                        let _ = writeln!(out, "  {}Could not fetch it: {e}{}", s.bad(), s.reset());
                        let _ = writeln!(
                            out,
                            "  {}Later: pcc pair --relay {addr} --session <code> --token <token>{}",
                            s.dim(),
                            s.reset()
                        );
                        return Ok(Outcome::Quit);
                    }
                },
            };
            relay_addr = Some(addr);
            relay_pin = Some(pin);
        }

        let code = ask(
            input,
            out,
            s,
            "Session code (viewers need it; blank generates one)",
            "",
        )?;
        session = if code.is_empty() {
            crate::relay::generate_session_code()
        } else {
            code
        };
    }

    // ---- audio ----
    let audio = choose(
        input,
        out,
        s,
        "Audio?",
        &[
            ("No audio", ""),
            ("Microphone", ""),
            (
                "System sound",
                "needs a loopback device; `pcc diagnose --audio` lists them",
            ),
            ("Both", "microphone and system sound together"),
        ],
        1,
        true,
    )?;
    let Some(audio) = audio else {
        return Ok(Outcome::Quit);
    };

    // ---- extras ----
    let browser = confirm(
        input,
        out,
        s,
        "Serve the browser viewer too (a phone can then watch from a link)?",
        true,
    )?;
    let approve = confirm(input, out, s, "Ask before admitting each viewer?", false)?;

    // ---- assemble ----
    if let (Some(addr), Some(pin)) = (&relay_addr, &relay_pin) {
        args.push("--relay".into());
        args.push(addr.clone());
        args.push("--relay-pin".into());
        args.push(pin.clone());
        args.push("--session".into());
        args.push(session.clone());
    }
    match audio {
        0 => args.push("--audio-source=none".into()),
        1 => args.push("--audio-source=mic".into()),
        2 => args.push("--audio-source=system".into()),
        _ => args.push("--audio-source=both".into()),
    }
    if approve {
        args.push("--approve".into());
    }
    if !browser {
        args.push("--no-web".into());
    }
    // Through a relay the token is not ours to choose. The share's --token
    // must equal the *relay's* --token or registration is refused with
    // `bad credential`, so generating one here would guarantee failure on
    // the one path this menu exists to make easy. A pasted relay line
    // already carried it; asking again would invite the very mismatch this
    // avoids. Only a direct share gets a generated token.
    let token = match (&relay_addr, relay_token) {
        (Some(_), Some(t)) => {
            let _ = writeln!(
                out,
                "  {}token taken from the pasted line{}",
                s.dim(),
                s.reset()
            );
            t
        }
        (Some(_), None) => {
            let t = ask(
                input,
                out,
                s,
                "Relay token (the `relay token:` line it printed)",
                "",
            )?;
            if t.is_empty() {
                let _ = writeln!(
                    out,
                    "  {}A relay share must use the relay's own token, and the relay\n  \
                     refuses to register without it. Stopping rather than starting a\n  \
                     session that cannot connect.{}",
                    s.bad(),
                    s.reset()
                );
                return Ok(Outcome::Quit);
            }
            t
        }
        (None, _) => SessionToken::generate().as_str().to_string(),
    };
    args.push("--token".into());
    args.push(token.clone());

    show(out, s, "Now run this for the sharing side", &args);

    let mut body = vec![format!(
        "{}This machine is the host.{} Start it and it prints the exact",
        s.bold(),
        s.reset()
    )];
    body.push(format!(
        "{}viewer command — send that to whoever is watching.{}",
        s.bold(),
        s.reset()
    ));
    if let Some(addr) = &relay_addr {
        if let Some(pin) = &relay_pin {
            body.push(String::new());
            body.push(format!("{}Viewer, any machine:{}", s.head(), s.reset()));
            body.push(format!(
                "  {}pcc view --relay {addr} --pin {pin}{}",
                s.good(),
                s.reset()
            ));
            body.push(format!("     --session {session} --token {token}"));
            if browser {
                body.push(String::new());
                body.push(format!(
                    "{}Viewer, any browser (needs `pcc relay --web`):{}",
                    s.head(),
                    s.reset()
                ));
                body.push(format!(
                    "  {}https://{addr}/v/{session}/#token={token}{}",
                    s.good(),
                    s.reset()
                ));
            }
        }
    } else {
        body.push(String::new());
        body.push("Viewers run the line it prints; it carries this machine's".to_string());
        body.push("address and the certificate pin, which is created at".to_string());
        body.push("startup. Copy that line — do not assemble your own.".to_string());
    }
    panel(out, s, "What the other side runs", &body);

    run_now(input, out, s, args)
}

// ------------------------------------------------------------------- view

fn view<R: BufRead, W: Write>(input: &mut R, out: &mut W, s: Style) -> Result<Outcome> {
    let how = choose(
        input,
        out,
        s,
        "How do you have the connection details?",
        &[
            ("A link", "pcc://view?... — paste it whole"),
            ("Direct address", "host:port + pin + token"),
            ("Through a relay", "relay address + session + pin + token"),
            ("An iroh ticket", "no ports, no relay to run"),
            ("A WebRTC offer blob", "manual signalling"),
        ],
        1,
        true,
    )?;
    let Some(how) = how else {
        return Ok(Outcome::Quit);
    };

    let mut args: Vec<String> = vec!["view".into()];
    match how {
        0 => {
            let link = ask(input, out, s, "Paste the link", "")?;
            if link.starts_with("http://") || link.starts_with("https://") {
                panel(
                    out,
                    s,
                    "Nothing to run",
                    &[
                        format!("{}That is a *browser* link.{}", s.bold(), s.reset()),
                        String::new(),
                        format!(
                            "{}Open it in any browser — no command, and nothing{}",
                            s.dim(),
                            s.reset()
                        ),
                        format!("{}has to be installed.{}", s.dim(), s.reset()),
                    ],
                );
                return Ok(Outcome::Quit);
            }
            match crate::reach::parse_pair_url(&link) {
                Ok(invite) => {
                    args = invite.to_view_args();
                    let via = if invite.relay.is_some() {
                        "relay"
                    } else {
                        "direct"
                    };
                    let _ = writeln!(
                        out,
                        "  {}parsed a {via} invite — pin and token filled in{}",
                        s.good(),
                        s.reset()
                    );
                }
                Err(e) => {
                    let _ = writeln!(
                        out,
                        "\n  {}That link did not parse: {e}{}",
                        s.bad(),
                        s.reset()
                    );
                    return Ok(Outcome::Quit);
                }
            }
        }
        1 => {
            let addr = ask(input, out, s, "Sharer address as host:port", "")?;
            if addr.is_empty() {
                return Ok(Outcome::Quit);
            }
            let pin = ask(input, out, s, "Sharer's certificate pin (64 hex)", "")?;
            let token = ask(input, out, s, "Token", "")?;
            args.extend([
                "--connect".into(),
                addr,
                "--pin".into(),
                pin,
                "--token".into(),
                token,
            ]);
        }
        2 => {
            let addr = ask(input, out, s, "Relay address as host:port", "")?;
            let code = ask(input, out, s, "Session code", "")?;
            let pin = ask(input, out, s, "Relay's certificate pin (64 hex)", "")?;
            let token = ask(input, out, s, "Token", "")?;
            args.extend([
                "--relay".into(),
                addr,
                "--session".into(),
                code,
                "--pin".into(),
                pin,
                "--token".into(),
                token,
            ]);
        }
        3 => {
            let ticket = ask(input, out, s, "Ticket", "")?;
            let token = ask(input, out, s, "Token", "")?;
            args.extend([
                "--transport".into(),
                "iroh".into(),
                "--ticket".into(),
                ticket,
                "--token".into(),
                token,
            ]);
        }
        _ => {
            let offer = ask(input, out, s, "Offer blob", "")?;
            let token = ask(input, out, s, "Token", "")?;
            args.extend([
                "--transport".into(),
                "webrtc".into(),
                "--offer".into(),
                offer,
                "--token".into(),
                token,
            ]);
        }
    }

    if !confirm(input, out, s, "Open a window?", true)? {
        args.push("--no-window".into());
    }
    if confirm(
        input,
        out,
        s,
        "Reconnect automatically if the connection drops?",
        true,
    )? {
        args.push("--reconnect".into());
    }

    show(out, s, "Now run this on the viewing side", &args);
    run_now(input, out, s, args)
}

// ------------------------------------------------------------------ relay

fn relay<R: BufRead, W: Write>(input: &mut R, out: &mut W, s: Style) -> Result<Outcome> {
    let _ = writeln!(out);
    hint(
        out,
        s,
        "A relay is a small server both sides dial out to. You need one when\n\
         neither machine can accept an incoming connection. Run it anywhere\n\
         both sides can reach: a VPS, a home server, or this machine.",
    );

    let listen = ask(input, out, s, "Listen address", "0.0.0.0:5900")?;
    let web = confirm(
        input,
        out,
        s,
        "Serve the browser viewer on the same port (a phone can then watch from a link)?",
        true,
    )?;
    let cert = ask_optional(
        input,
        out,
        s,
        "PEM certificate path (for a stable identity)",
    )?;
    let key = ask_optional(input, out, s, "PEM key path")?;

    let mut args: Vec<String> = vec!["relay".into(), "--listen".into(), listen.clone()];
    if web {
        args.push("--web".into());
    }
    if let (Some(c), Some(k)) = (&cert, &key) {
        args.push("--cert".into());
        args.push(c.clone());
        args.push("--cert-key".into());
        args.push(k.clone());
    }

    show(out, s, "Now run this for the relay", &args);

    let mut body = vec![format!(
        "{}Leave it running.{} It holds this terminal, so open a second one",
        s.bold(),
        s.reset()
    )];
    body.push("for the sharing side.".to_string());
    body.push(String::new());
    body.push(format!(
        "{}The moment it starts it prints a complete line beginning{}",
        s.dim(),
        s.reset()
    ));
    body.push(format!(
        "{}{}pcc share --relay …{}{}",
        s.dim(),
        s.good(),
        s.reset(),
        s.reset()
    ));
    body.push(format!(
        "{}with its own token and pin already filled in. Copy that whole{}",
        s.dim(),
        s.reset()
    ));
    body.push(format!(
        "{}line to the machine being shared.{}",
        s.dim(),
        s.reset()
    ));
    if web {
        body.push(String::new());
        body.push(format!(
            "{}It prints the browser link too, so a viewer can watch with{}",
            s.dim(),
            s.reset()
        ));
        body.push(format!(
            "{}only a URL once the sharer is running.{}",
            s.dim(),
            s.reset()
        ));
    } else {
        body.push(String::new());
        body.push(format!(
            "{}You turned the browser viewer off, so viewers need the `pcc`{}",
            s.dim(),
            s.reset()
        ));
        body.push(format!("{}binary installed.{}", s.dim(), s.reset()));
    }
    panel(out, s, "After it starts", &body);

    note(
        out,
        s,
        "The token is one value across all three sides. A share whose token\n  \
         disagrees with the relay's is refused with `bad credential`.",
    );

    run_now(input, out, s, args)
}

// --------------------------------------------------------------- diagnose

fn diagnose<R: BufRead, W: Write>(input: &mut R, out: &mut W, s: Style) -> Result<Outcome> {
    let what = choose(
        input,
        out,
        s,
        "What should it report?",
        &[
            (
                "Everything",
                "reachability, audio, and the path it would take",
            ),
            ("Displays", "which screens can be shared, and their sizes"),
            (
                "Audio devices",
                "marking the loopback taps system audio needs",
            ),
        ],
        1,
        true,
    )?;
    let Some(what) = what else {
        return Ok(Outcome::Quit);
    };
    let mut args: Vec<String> = vec!["diagnose".into()];
    match what {
        1 => args.push("--displays".into()),
        2 => args.push("--audio".into()),
        _ => {}
    }
    show(out, s, "Now run this", &args);
    run_now(input, out, s, args)
}

// ----------------------------------------------------------------- shared

/// Show the command exactly as it would be typed.
fn show<W: Write>(out: &mut W, s: Style, label: &str, args: &[String]) {
    panel(
        out,
        s,
        label,
        &[format!(
            "{}{}pcc {}{}",
            s.good(),
            s.bold(),
            render_args(args),
            s.reset()
        )],
    );
}

/// Render argv the way a shell would need it: quote anything that is not
/// plainly safe, so the printed line can be pasted back verbatim.
fn render_args(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            let safe = !a.is_empty()
                && a.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-.,:=/@".contains(c));
            if safe {
                a.clone()
            } else {
                format!("'{}'", a.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn run_now<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    s: Style,
    args: Vec<String>,
) -> Result<Outcome> {
    if confirm(input, out, s, "Run it now?", true)? {
        let _ = writeln!(out, "  {}Starting…{}", s.good(), s.reset());
        Ok(Outcome::Run(args))
    } else {
        let _ = writeln!(
            out,
            "  {}Not started — copy the line above whenever you are ready.{}",
            s.dim(),
            s.reset()
        );
        Ok(Outcome::Quit)
    }
}

/// Read the relay's certificate fingerprint off the wire, exactly as
/// `pcc pair --relay` does.
fn fetch_relay_pin(relay: &str) -> Result<String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let addr = crate::network::resolve(relay).await?;
        let name = relay.rsplit_once(':').map(|(h, _)| h).unwrap_or("pcc");
        crate::network::fetch_fingerprint(addr, name).await
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn drive(input: &str) -> (Outcome, String) {
        let mut cursor = Cursor::new(input.to_string());
        let mut out = Vec::new();
        let outcome = run(&mut cursor, &mut out, Style::PLAIN).expect("menu should not error");
        (outcome, String::from_utf8(out).unwrap())
    }

    /// The whole point: an invite pasted whole becomes the right argv.
    #[test]
    fn pasting_a_relay_invite_produces_the_view_command() {
        let (outcome, _) = drive(
            "2\n1\npcc://view?relay=relay.example:5900&pin=abc123&session=S1&token=TOK\nn\nn\ny\n",
        );
        match outcome {
            Outcome::Run(args) => assert_eq!(
                args,
                vec![
                    "view",
                    "--relay",
                    "relay.example:5900",
                    "--session",
                    "S1",
                    "--pin",
                    "abc123",
                    "--token",
                    "TOK",
                    "--no-window",
                ]
            ),
            _ => panic!("expected a view command"),
        }
    }

    /// A browser link is not a command; running one would be wrong.
    #[test]
    fn a_browser_link_runs_nothing() {
        let (outcome, text) = drive("2\n1\nhttps://relay.example:5900/v/S1/#token=TOK\n");
        assert!(matches!(outcome, Outcome::Quit));
        assert!(text.contains("Open it in any browser"));
    }

    /// Empty input must not fall through to the first option.
    #[test]
    fn empty_input_quits_rather_than_sharing() {
        let (outcome, _) = drive("");
        assert!(matches!(outcome, Outcome::Quit));
    }

    /// Sharing through a relay proves the viewer line is complete, and that
    /// the token is the *relay's* — generating one here would guarantee
    /// `bad credential` on the one path this menu exists to make easy.
    #[test]
    fn relay_share_uses_the_relay_token_not_a_generated_one() {
        // The blank line skips the paste shortcut so this exercises the
        // typed-answers path.
        let (outcome, text) =
            drive("1\n1\n2\n\nrelay.example:5900\ndeadbeef\nSESS\n1\ny\nn\nRELAYTOKEN\ny\n");
        match outcome {
            Outcome::Run(args) => {
                assert_eq!(args[0], "share");
                assert!(args.contains(&"--relay".to_string()));
                assert!(args.contains(&"--relay-pin".to_string()));
                let token = args[args.iter().position(|a| a == "--token").unwrap() + 1].clone();
                assert_eq!(token, "RELAYTOKEN");
                assert!(text.contains(&format!("--session SESS --token {token}")));
            }
            _ => panic!("expected a share command"),
        }
    }

    /// A relay share with no token cannot work, so it must stop rather than
    /// start a session that will be refused.
    #[test]
    fn a_relay_share_without_a_token_stops() {
        let (outcome, text) =
            drive("1\n1\n2\n\nrelay.example:5900\ndeadbeef\nSESS\n1\ny\nn\n\nn\n");
        assert!(matches!(outcome, Outcome::Quit));
        assert!(text.contains("relay's own token"));
    }

    /// A direct share has no relay token to borrow, so it must generate one
    /// (and print it in the viewer line).
    #[test]
    fn a_direct_share_generates_a_token() {
        let (outcome, text) = drive("1\n1\n1\n1\ny\nn\ny\n");
        match outcome {
            Outcome::Run(args) => {
                let token = args[args.iter().position(|a| a == "--token").unwrap() + 1].clone();
                assert!(
                    token.len() >= 8,
                    "a generated token must satisfy the parser"
                );
                assert!(!args.contains(&"--relay".to_string()));
                assert!(text.contains(&token));
            }
            _ => panic!("expected a share command"),
        }
    }

    /// One paste must replace the hunt for four separate values.
    #[test]
    fn pasting_the_relay_line_fills_in_address_pin_and_token() {
        let pin = "b".repeat(64);
        let line =
            format!("pcc share --relay 203.0.113.5:5900 --relay-pin {pin} --token RELAYTOK123");
        // share → whole screen → through a relay → the pasted line →
        // session → no audio → browser on → no approval → run.
        let (outcome, text) = drive(&format!("1\n1\n2\n{line}\nSESS\n1\ny\nn\ny\n"));
        match outcome {
            Outcome::Run(args) => {
                let value =
                    |flag: &str| args[args.iter().position(|a| a == flag).unwrap() + 1].clone();
                assert_eq!(value("--relay"), "203.0.113.5:5900");
                assert_eq!(value("--relay-pin"), pin);
                assert_eq!(value("--token"), "RELAYTOK123");
                // The token was never asked for: the pasted line had it.
                assert!(text.contains("token taken from the pasted line"));
            }
            _ => panic!("expected a share command"),
        }
    }

    /// A line that does not parse falls back to the questions rather than
    /// failing the whole flow.
    #[test]
    fn an_unparseable_relay_line_falls_back_to_the_questions() {
        let (outcome, text) =
            drive("1\n1\n2\ngarbage that is not a relay line\n127.0.0.1:5900\ndeadbeef\nSESS\n1\ny\nn\nTOK12345678\ny\n");
        match outcome {
            Outcome::Run(args) => {
                assert!(args.contains(&"--relay".to_string()));
                assert!(text.contains("did not parse"));
            }
            _ => panic!("expected the questions to still work"),
        }
    }

    /// Quoting: a window title with spaces must come back paste-able.
    #[test]
    fn args_are_quoted_for_the_shell() {
        assert_eq!(
            render_args(&["share".into(), "--window".into(), "My Editor".into()]),
            "share --window 'My Editor'"
        );
    }

    /// Panels pad to the widest line, so a styled line must measure as its
    /// visible text, not as its bytes — otherwise every box is ragged.
    #[test]
    fn styling_does_not_count_toward_the_printed_width() {
        assert_eq!(visible_len("abc"), 3);
        assert_eq!(visible_len("\x1b[1;96mabc\x1b[0m"), 3);
        assert_eq!(visible_len(""), 0);
        assert_eq!(visible_len("\x1b[1m"), 0);
    }

    /// Colour is opt-out: piping the menu must not emit escape codes, or
    /// every greppable line and every logged transcript gets polluted.
    #[test]
    fn plain_style_emits_no_escape_codes() {
        let (_, text) = drive("q\n");
        assert!(
            !text.contains('\x1b'),
            "plain output must be escape-free, got: {text:?}"
        );
    }

    /// And when it is on, the styling actually reaches the output.
    #[test]
    fn styled_output_carries_escape_codes() {
        let mut cursor = Cursor::new(b"q\n".to_vec());
        let mut out = Vec::new();
        let style = Style { color: true };
        let _ = run(&mut cursor, &mut out, style).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains('\x1b'), "styled output should carry codes");
        assert!(!text.contains("\x1b\x1b"), "no doubled codes");
    }
}
