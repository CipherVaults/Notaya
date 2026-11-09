"""Offline source-linked proofs; writes only new throwaway evidence directories.

Uses the repository's existing Rust dependency declarations and cached packages.
Never starts the Notaya executable, a WebView, a pipe, or a dialog. No cleanup.
"""
from pathlib import Path
import datetime
import re
import shutil
import sys
import subprocess

REPO = Path(__file__).resolve().parents[3]
BASE = Path(r"D:\Notaya Tests\v1-6-S")
STAMP = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
if len(sys.argv) == 2:
    # Reuse only the generated build cache; data-run always gets a fresh name.
    RUN = Path(sys.argv[1]).resolve()
    assert RUN.parent == BASE.resolve() and RUN.name.startswith("area1-proof-")
    assert (RUN / "Cargo.toml").exists()
else:
    RUN = BASE / ("area1-proof-" + STAMP)
    RUN.mkdir(parents=True, exist_ok=False)
src = REPO / "src-tauri"
manifest = (src / "Cargo.toml").read_text(encoding="utf-8")
manifest = re.sub(r'^members = .*\n', '', manifest, flags=re.M)
manifest = re.sub(r'\[lib\][\s\S]*?(?=\[build-dependencies\])',
                  '[[bin]]\nname = "area1-core"\npath = "proof.rs"\n\n', manifest)
(RUN / "Cargo.toml").write_text(manifest, encoding="utf-8")
shutil.copyfile(src / "Cargo.lock", RUN / "Cargo.lock")
modules = re.findall(r'^mod (\w+);', (src / "src/lib.rs").read_text(encoding="utf-8"), re.M)
modules = [m for m in modules if m != "roundtrip_tests"]
prefix = '#![allow(dead_code)]\n'
for m in modules:
    path = src / "src" / (m + ".rs")
    if not path.exists():
        path = src / "src" / m / "mod.rs"
    prefix += '#[path = r"' + str(path) + '"] mod ' + m + ';\n'
body = Path(__file__).with_name("core-repro.rs").read_text(encoding="utf-8")
(RUN / "proof.rs").write_text(prefix + body, encoding="utf-8")
print("Proof execution directory:", RUN, flush=True)
cmd = ["cargo", "run", "--offline", "--manifest-path", str(RUN / "Cargo.toml"),
       "--target-dir", str(RUN / "target"), "--bin", "area1-core", "--", str(RUN / ("data-run-" + STAMP))]
result = subprocess.run(cmd, cwd=REPO, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
(RUN / ("result-" + STAMP + ".txt")).write_text(result.stdout, encoding="utf-8")
print(result.stdout[-16000:])
raise SystemExit(result.returncode)
