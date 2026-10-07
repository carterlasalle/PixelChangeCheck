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

use crate::network::SessionToken;
use anyhow::Result;
use std::io::{BufRead, Write};

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

/// Run the interview. `input`/`output` are injected so this is testable
/// without a terminal.
pub fn run<R: BufRead, W: Write>(input: &mut R, out: &mut W) -> Result<Outcome> {
    let result = interview(input, out);
    match result {
        Err(e) if e.downcast_ref::<Eof>().is_some() => Ok(Outcome::Quit),
        other => other,
    }
}

fn interview<R: BufRead, W: Write>(input: &mut R, out: &mut W) -> Result<Outcome> {
    banner(out);
    let choice = choose(
        input,
        out,
        "What would you like to do?",
        &[
            "Share this screen            (you are the host)",
            "View someone else's screen   (you are the viewer)",
            "Run a relay                  (for two people who cannot reach each other)",
            "Check this machine           (what can it capture, is audio available)",
            "Show every command",
        ],
        1,
        true,
    )?;

    let Some(choice) = choice else {
        return Ok(Outcome::Quit);
    };

    match choice {
        0 => share(input, out),
        1 => view(input, out),
        2 => relay(input, out),
        3 => diagnose(input, out),
        _ => Ok(Outcome::Run(vec!["help".to_string()])),
    }
}

fn banner<W: Write>(out: &mut W) {
    let _ = writeln!(
        out,
        "\n  PixelChangeCheck — exact, lossless screen sharing\n  \
         ────────────────────────────────────────────────────\n  \
         Answer the questions and this prints (and optionally runs) the\n  \
         exact command. Press Enter to accept the [default].\n"
    );
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
    prompt: &str,
    default: &str,
) -> Result<String> {
    if default.is_empty() {
        let _ = write!(out, "{prompt}: ");
    } else {
        let _ = write!(out, "{prompt} [{default}]: ");
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
    prompt: &str,
) -> Result<Option<String>> {
    let _ = write!(out, "{prompt} (blank to skip): ");
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
    question: &str,
    options: &[&str],
    default: usize,
    allow_quit: bool,
) -> Result<Option<usize>> {
    let _ = writeln!(out, "\n{question}");
    for (i, opt) in options.iter().enumerate() {
        let _ = writeln!(out, "  {}) {opt}", i + 1);
    }
    if allow_quit {
        let _ = writeln!(out, "  q) Quit");
    }
    let _ = write!(out, "Choose [{default}]: ");
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
                    "  (that is not one of the numbers above — starting over)\n"
                );
                choose(input, out, question, options, default, allow_quit)
            }
        },
    }
}

fn confirm<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    question: &str,
    default_yes: bool,
) -> Result<bool> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    let _ = write!(out, "{question} {suffix}: ");
    let _ = out.flush();
    match line(input) {
        None => eof(),
        Some(v) if v.is_empty() => Ok(default_yes),
        Some(v) => Ok(matches!(v.to_ascii_lowercase().as_str(), "y" | "yes")),
    }
}

// ------------------------------------------------------------------ share

fn share<R: BufRead, W: Write>(input: &mut R, out: &mut W) -> Result<Outcome> {
    let target = choose(
        input,
        out,
        "What do you want to share?",
        &[
            "The whole screen      (the primary display)",
            "A specific display",
            "A region of the screen",
            "A window              (matched by its title)",
            "An application        (matched by its name)",
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
                "Display number (see `pcc diagnose --displays`)",
                "0",
            )?;
            args.push("--display".into());
            args.push(n);
        }
        2 => {
            let r = ask(input, out, "Region as x,y,w,h (e.g. 100,200,1280,720)", "")?;
            if r.is_empty() {
                let _ = writeln!(out, "  (a region is required — stopping)");
                return Ok(Outcome::Quit);
            }
            args.push("--region".into());
            args.push(r);
        }
        3 => {
            let t = ask(input, out, "Part of the window title", "")?;
            if t.is_empty() {
                let _ = writeln!(out, "  (a title is required — stopping)");
                return Ok(Outcome::Quit);
            }
            args.push("--window".into());
            args.push(t);
        }
        _ => {
            let a = ask(input, out, "Application name", "")?;
            if a.is_empty() {
                let _ = writeln!(out, "  (a name is required — stopping)");
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
        "How will viewers connect?",
        &[
            "Directly             (same network or VPN — nothing to set up)",
            "Through a relay      (works over the internet, no port forwarding)",
            "Both                 (direct when possible, relay as the fallback)",
        ],
        1,
        true,
    )?;
    let Some(route) = route else {
        return Ok(Outcome::Quit);
    };

    let mut relay_addr = None;
    let mut relay_pin = None;
    let mut session = String::new();
    if route >= 1 {
        let addr = ask(input, out, "Relay address as host:port", "")?;
        if addr.is_empty() {
            let _ = writeln!(out, "  (a relay address is required — stopping)");
            return Ok(Outcome::Quit);
        }
        let pin = match ask_optional(
            input,
            out,
            "Relay pin (64 hex characters; blank tries to fetch it from the relay)",
        )? {
            Some(p) => p,
            None => match fetch_relay_pin(&addr) {
                Ok(p) => {
                    let _ = writeln!(out, "  fetched: {p}");
                    let _ = writeln!(
                        out,
                        "  compare this against whatever the relay operator published"
                    );
                    p
                }
                Err(e) => {
                    let _ = writeln!(out, "  could not fetch it: {e}");
                    let _ = writeln!(
                        out,
                        "  run `pcc pair --relay {addr} --session <code> --token <token>` later, \
                         or paste the pin from the relay's own output"
                    );
                    return Ok(Outcome::Quit);
                }
            },
        };
        let code = ask(
            input,
            out,
            "Session code (viewers need it; blank generates one)",
            "",
        )?;
        session = if code.is_empty() {
            crate::relay::generate_session_code()
        } else {
            code
        };
        relay_addr = Some(addr);
        relay_pin = Some(pin);
    }

    // ---- audio ----
    let audio = choose(
        input,
        out,
        "Audio?",
        &[
            "No audio",
            "Microphone",
            "System sound     (needs a loopback device; `pcc diagnose --audio` lists them)",
            "Both microphone and system sound",
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
        "Serve the browser viewer too (a phone can open a link)?",
        true,
    )?;
    let approve = confirm(input, out, "Ask before admitting each viewer?", false)?;

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
    // A token the menu chose, so the viewer instructions below are complete
    // rather than "read it off the sharer's output".
    let token = SessionToken::generate();
    args.push("--token".into());
    args.push(token.as_str().to_string());

    show(out, "Command", &args);
    let _ = writeln!(
        out,
        "\n  This machine is the host. When it starts it prints the exact\n  \
         viewer command and, for the browser, a link — send one of those.\n"
    );
    if let Some(addr) = &relay_addr {
        if let Some(pin) = &relay_pin {
            let _ = writeln!(out, "  Viewer (another machine, nothing else needed):");
            let _ = writeln!(
                out,
                "    pcc view --relay {addr} --pin {pin} --session {session} --token {}",
                token.as_str()
            );
            if browser {
                let _ = writeln!(
                    out,
                    "\n  Viewer (any browser — needs `pcc relay --web` on the relay):"
                );
                let _ = writeln!(
                    out,
                    "    https://{addr}/v/{session}/#token={}",
                    token.as_str()
                );
            }
        }
    } else {
        let _ = writeln!(
            out,
            "  Viewers run: pcc view --connect <this-machine-ip>:5800 --token {} --pin <pin>",
            token.as_str()
        );
        let _ = writeln!(
            out,
            "  The pin is created when the sharer starts, so copy the line it\n  \
             prints — it fills the pin in for you."
        );
    }

    run_now(input, out, args)
}

// ------------------------------------------------------------------- view

fn view<R: BufRead, W: Write>(input: &mut R, out: &mut W) -> Result<Outcome> {
    let how = choose(
        input,
        out,
        "How do you have the connection details?",
        &[
            "A link            (pcc://view?... — paste it whole)",
            "Direct address    (host:port + pin + token)",
            "Through a relay   (relay address + session + pin + token)",
            "An iroh ticket",
            "A WebRTC offer blob",
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
            let link = ask(input, out, "Paste the link", "")?;
            if link.starts_with("http://") || link.starts_with("https://") {
                let _ = writeln!(
                    out,
                    "\n  That is a *browser* link. Just open it — no command needed,\n  \
                     and nothing has to be installed."
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
                    let _ = writeln!(out, "  (parsed a {via} invite)");
                }
                Err(e) => {
                    let _ = writeln!(out, "\n  That link did not parse: {e}");
                    return Ok(Outcome::Quit);
                }
            }
        }
        1 => {
            let addr = ask(input, out, "Sharer address as host:port", "")?;
            if addr.is_empty() {
                return Ok(Outcome::Quit);
            }
            let pin = ask(input, out, "Sharer's certificate pin (64 hex)", "")?;
            let token = ask(input, out, "Token", "")?;
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
            let addr = ask(input, out, "Relay address as host:port", "")?;
            let code = ask(input, out, "Session code", "")?;
            let pin = ask(input, out, "Relay's certificate pin (64 hex)", "")?;
            let token = ask(input, out, "Token", "")?;
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
            let ticket = ask(input, out, "Ticket", "")?;
            let token = ask(input, out, "Token", "")?;
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
            let offer = ask(input, out, "Offer blob", "")?;
            let token = ask(input, out, "Token", "")?;
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

    if !confirm(input, out, "Open a window?", true)? {
        args.push("--no-window".into());
    }
    if confirm(
        input,
        out,
        "Reconnect automatically if the connection drops?",
        true,
    )? {
        args.push("--reconnect".into());
    }

    show(out, "Command", &args);
    run_now(input, out, args)
}

// ------------------------------------------------------------------ relay

fn relay<R: BufRead, W: Write>(input: &mut R, out: &mut W) -> Result<Outcome> {
    let _ = writeln!(
        out,
        "\n  A relay is a small server both sides dial out to. You need one when\n  \
         neither machine can accept an incoming connection. Run it anywhere both\n  \
         sides can reach: a VPS, a home server, or this machine for a test.\n"
    );
    let listen = ask(input, out, "Listen address", "0.0.0.0:5900")?;
    let web = confirm(
        input,
        out,
        "Serve the browser viewer on the same port (a phone can then watch with only a link)?",
        true,
    )?;
    let cert = ask_optional(input, out, "PEM certificate path (for a stable identity)")?;
    let key = ask_optional(input, out, "PEM key path")?;

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

    show(out, "Command", &args);
    let _ = writeln!(
        out,
        "\n  It prints its own token and certificate pin. Keep it running.\n\n  \
         On the machine being shared, run:\n    \
         pcc share --relay <this-host>:{port} --relay-pin <the pin it printed> --token <its token>\n\n  \
         On the other machine, run:\n    \
         pcc view --relay <this-host>:{port} --pin <the same pin> --session <code> --token <the same token>\n\n  \
         The token is one value shared by all three: the relay's --token must\n  \
         equal the sharer's --token, and the viewer's must match too. Getting\n  \
         that wrong is the most common failure, and it reports itself as\n  \
         `bad credential`.",
        port = listen.rsplit_once(':').map(|(_, p)| p).unwrap_or("5900")
    );
    if !web {
        let _ = writeln!(
            out,
            "\n  You chose not to serve the browser viewer, so viewers need the\n  \
             `pcc` binary installed."
        );
    }
    run_now(input, out, args)
}

// --------------------------------------------------------------- diagnose

fn diagnose<R: BufRead, W: Write>(input: &mut R, out: &mut W) -> Result<Outcome> {
    let what = choose(
        input,
        out,
        "What should it report?",
        &[
            "Everything          (reachability, audio, and the path it would take)",
            "Displays            (which screens can be shared, and their sizes)",
            "Audio devices       (marking the loopback taps system audio needs)",
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
    show(out, "Command", &args);
    run_now(input, out, args)
}

// ----------------------------------------------------------------- shared

/// Show the command exactly as it would be typed.
fn show<W: Write>(out: &mut W, label: &str, args: &[String]) {
    let _ = writeln!(out, "\n  {label}:\n");
    let _ = writeln!(out, "    pcc {}", render_args(args));
    let _ = writeln!(out);
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

fn run_now<R: BufRead, W: Write>(input: &mut R, out: &mut W, args: Vec<String>) -> Result<Outcome> {
    if confirm(input, out, "Run it now?", true)? {
        Ok(Outcome::Run(args))
    } else {
        let _ = writeln!(
            out,
            "\n  Not started. Copy the line above whenever you are ready."
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
        let outcome = run(&mut cursor, &mut out).expect("menu should not error");
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
        assert!(text.contains("Just open it"));
    }

    /// Empty input must not fall through to the first option.
    #[test]
    fn empty_input_quits_rather_than_sharing() {
        let (outcome, _) = drive("");
        assert!(matches!(outcome, Outcome::Quit));
    }

    /// Sharing through a relay proves the viewer line is complete.
    #[test]
    fn relay_share_prints_a_complete_viewer_command() {
        let (outcome, text) = drive("1\n1\n2\nrelay.example:5900\ndeadbeef\nSESS\n1\ny\nn\ny\n");
        match outcome {
            Outcome::Run(args) => {
                assert_eq!(args[0], "share");
                assert!(args.contains(&"--relay".to_string()));
                assert!(args.contains(&"--relay-pin".to_string()));
                let token = args[args.iter().position(|a| a == "--token").unwrap() + 1].clone();
                assert!(text.contains(&format!("--session SESS --token {token}")));
            }
            _ => panic!("expected a share command"),
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
}
