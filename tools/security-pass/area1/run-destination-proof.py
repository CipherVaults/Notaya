"""Execute the unchanged destination/write tail of save_file_as_person offline.

The file picker is represented by its chosen path; no live dialog is opened.
The functions extension, resolve, checked_destination and vault-extension check
are taken from the reviewed sources, not reimplemented in Python.
"""
from pathlib import Path
import datetime
import re
import subprocess

REPO = Path(__file__).resolve().parents[3]
BASE = Path(r"D:\Notaya Tests\v1-6-S")
RUN = BASE / ("area1-destination-" + datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ"))
RUN.mkdir(parents=True, exist_ok=False)

def function(path, name):
    text = path.read_text(encoding="utf-8")
    start = re.search(r'^(?:pub(?:\([^\n]*\))? )?fn ' + name + r'\(', text, re.M).start()
    end = re.search(r'^}', text[start:], re.M).end() + start
    return text[start:end]

io = REPO / "src-tauri/src/commands/database_io.rs"
export = REPO / "src-tauri/src/export/mod.rs"
save = function(io, "save_file_as_person")
tail = save[save.index("    let dest = "):]
tail = tail[:tail.rfind("}")]
# The state adapter is only the destination folder used by the real tail.
rust = r'''
#![allow(dead_code)]
use std::{fs, path::{Path, PathBuf}};
#[derive(Debug)] enum Error { Invalid(String), Other(String) }
impl Error { fn invalid(s: impl Into<String>) -> Self { Self::Invalid(s.into()) } }
type Result<T> = std::result::Result<T, Error>;
struct Paths { data: PathBuf }
struct State { paths: Paths }
'''
filenames = REPO / "src-tauri/src/filenames.rs"
rust += '#[path = r"' + str(filenames) + '"] mod filenames;\n'
rust += function(io, "extension") + '\n'
rust += function(export, "resolve") + '\n'
rust += function(export, "checked_destination") + '\n'
rust += 'fn write_tail(name: &str, chosen: PathBuf, bytes: &[u8], state: &State) -> Result<Option<String>> {\n'
rust += '    let ext = extension(&name);\n' + tail + '}\n'
rust += r'''
fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).unwrap());
    assert!(root.starts_with(PathBuf::from(r"D:\Notaya Tests\v1-6-S")) && !root.exists());
    fs::create_dir_all(root.join("data")).unwrap();
    let state = State { paths: Paths { data: root.join("data") } };
    let target = state.paths.data.join("shell-prefs.json");
    let original = b"{\"closeToTray\":true,\"startMinimized\":false}";
    let payload = b"area1-generated-throwaway-attachment";
    fs::write(&target, original).unwrap();
    assert!(write_tail("attachment.json", target.clone(), payload, &state).is_err());
    assert_eq!(fs::read(&target).unwrap(), original);
    assert!(write_tail("attachment", target.clone(), payload, &state).is_ok());
    assert_eq!(fs::read(&target).unwrap(), payload);
    println!("REPRODUCED: extensionless attachment overwrote throwaway data/shell-prefs.json; named-extension control refused");
    let outside = root.join("outside.nota");
    assert!(write_tail("attachment", outside.clone(), payload, &state).is_err());
    assert!(!outside.exists());
    println!("CONTROL: direct .nota destination remains refused");
    println!("Evidence preserved at {}", root.display());
}
'''
(RUN / "proof.rs").write_text(rust, encoding="utf-8")
exe = RUN / "destination-proof.exe"
compiled = subprocess.run(["rustc", "--edition=2024", str(RUN / "proof.rs"), "-o", str(exe)],
                          capture_output=True, text=True)
output = compiled.stdout + compiled.stderr
code = compiled.returncode
if code == 0:
    result = subprocess.run([str(exe), str(RUN / "data-run")], capture_output=True, text=True)
    output += result.stdout + result.stderr
    code = result.returncode
(RUN / "result.txt").write_text(output, encoding="utf-8")
print("Proof execution directory:", RUN)
print(output[-10000:])
raise SystemExit(code)
