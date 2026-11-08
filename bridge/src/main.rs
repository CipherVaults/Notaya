mod mcp;
mod pipe;
mod tools;

use mcp::{Effect, Mcp};
use pipe::{ClientMessage, Transport};
use std::{
    collections::VecDeque,
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

fn main() {
    if run().is_err() {
        // Do not print paths, tokens, input JSON or OS diagnostics that might contain note text.
        eprintln!("Notaya bridge: the connection ended. Check Notaya's AI access settings.");
        std::process::exit(1);
    }
}

fn options(args: impl Iterator<Item = std::ffi::OsString>) -> io::Result<Option<PathBuf>> {
    let mut args = args;
    let mut home = None;
    while let Some(arg) = args.next() {
        if arg == "--home" && home.is_none() {
            let value = args
                .next()
                .filter(|value| !value.is_empty())
                .ok_or(io::ErrorKind::InvalidInput)?;
            home = Some(PathBuf::from(value));
        } else {
            return Err(io::ErrorKind::InvalidInput.into());
        }
    }
    Ok(home.or_else(|| {
        ["NOTAYA_HOME", "CIPHERLEAF_HOME"]
            .into_iter()
            .filter_map(std::env::var_os)
            .find(|v| !v.is_empty())
            .map(PathBuf::from)
    }))
}

fn output(value: &serde_json::Value) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, value)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

struct Runtime {
    mcp: Mcp,
    pipe: Option<Box<dyn Transport>>,
    data: Option<PathBuf>,
    beside: PathBuf,
    home: Option<PathBuf>,
}

impl Runtime {
    fn effects(&mut self, effects: Vec<Effect>) -> io::Result<()> {
        let mut queue: VecDeque<_> = effects.into();
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Output(value) => output(&value)?,
                Effect::Connect(client) => {
                    match connect(&self.beside, self.home.as_deref(), &client) {
                        Ok((pipe, data, paired)) => {
                            self.pipe = Some(pipe);
                            self.data = Some(data);
                            self.mcp.connected(paired);
                        }
                        Err(()) => {
                            queue.extend(self.mcp.disconnected("not_running"));
                        }
                    }
                }
                Effect::Send(message) => {
                    if self
                        .pipe
                        .as_mut()
                        .is_none_or(|pipe| pipe.send(&message).is_err())
                    {
                        self.pipe = None;
                        queue.extend(self.mcp.disconnected("not_running"));
                    }
                }
                Effect::Save(pairing) => {
                    if self
                        .data
                        .as_ref()
                        .is_some_and(|data| pipe::save_pairing(data, &pairing).is_ok())
                    {
                        queue.extend(self.mcp.saved_pairing());
                    } else {
                        self.pipe = None;
                        queue.extend(self.mcp.disconnected("not_paired"));
                    }
                }
                Effect::Close => {
                    self.pipe = None;
                    self.data = None;
                }
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
fn connect(
    beside: &Path,
    home: Option<&Path>,
    client: &str,
) -> Result<(Box<dyn Transport>, PathBuf, bool), ()> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let data = pipe::data_dir(beside, home, local.as_deref()).map_err(|_| ())?;
    let sid = pipe::native::current_sid().map_err(|_| ())?;
    let name = pipe::pipe_name(&data, &sid).map_err(|_| ())?;
    // Identity is verified inside connect before even reading the credential file.
    let mut pipe = pipe::native::Connection::connect(&name, beside).map_err(|_| ())?;
    let pairing = pipe::load_pairing(&data, client).map_err(|_| ())?;
    pipe.send(&ClientMessage::Hello {
        bridge_version: env!("CARGO_PKG_VERSION").into(),
        client: client.into(),
    })
    .map_err(|_| ())?;
    let paired = pairing.is_some();
    if let Some(pairing) = pairing {
        pipe.send(&ClientMessage::Auth {
            pairing_id: pairing.pairing_id,
            token: pairing.token,
        })
        .map_err(|_| ())?;
    }
    Ok((Box::new(pipe), data, paired))
}

#[cfg(not(windows))]
fn connect(_: &Path, _: Option<&Path>, _: &str) -> Result<(Box<dyn Transport>, PathBuf, bool), ()> {
    Err(())
}

fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" {
        eprintln!(
            "notaya-bridge [--home <folder>]\nMCP over stdio. Open the Notaya.exe beside this bridge and enable AI access.\n--home overrides NOTAYA_HOME (the folder that contains data). Diagnostics use stderr."
        );
        return Ok(());
    }
    let home = options(args.into_iter())?;
    let exe = std::env::current_exe()?;
    let beside = exe.parent().ok_or(io::ErrorKind::NotFound)?.to_path_buf();
    // Bounded input queue. EOF ends this process even if Notaya is waiting for confirmation.
    let (send, receive) = mpsc::sync_channel(16);
    std::thread::spawn(move || {
        let mut stdin = BufReader::new(io::stdin());
        loop {
            let line = mcp::read_line(&mut stdin);
            let done = !matches!(line, Ok(Some(_)));
            if send.send(line).is_err() || done {
                break;
            }
        }
    });
    let mut runtime = Runtime {
        mcp: Mcp::default(),
        pipe: None,
        data: None,
        beside,
        home,
    };
    loop {
        match receive.recv_timeout(Duration::from_millis(20)) {
            Ok(Ok(Some(bytes))) => {
                let effects = match serde_json::from_slice(&bytes) {
                    Ok(value) => runtime.mcp.input(value),
                    Err(_) => vec![mcp::parse_error()],
                };
                runtime.effects(effects)?;
            }
            Ok(Ok(None)) | Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            Ok(Err(error)) => return Err(error),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(pipe) = runtime.pipe.as_mut() {
            match pipe.poll() {
                Ok(messages) => {
                    for message in messages {
                        if runtime.pipe.is_none() {
                            break;
                        }
                        let effects = runtime.mcp.server(message);
                        runtime.effects(effects)?;
                    }
                }
                Err(_) => {
                    runtime.pipe = None;
                    let effects = runtime.mcp.disconnected("not_running");
                    runtime.effects(effects)?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_line_override_wins_and_unknown_options_are_refused() {
        assert_eq!(
            options(
                ["--home", "D:\\Notaya Tests\\v1-6-D"]
                    .into_iter()
                    .map(Into::into)
            )
            .unwrap()
            .unwrap(),
            PathBuf::from("D:\\Notaya Tests\\v1-6-D")
        );
        for args in [
            vec!["--home"],
            vec!["--home", ""],
            vec!["--pipe", "real"],
            vec!["--home", "a", "--home", "b"],
        ] {
            assert!(options(args.into_iter().map(Into::into)).is_err());
        }
    }
}
