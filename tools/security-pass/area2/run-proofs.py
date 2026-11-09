"""Compile production modules with existing offline dependencies; preserve all evidence.

No Cargo/build hooks or application entry point are run. Only a fresh directory
under the approved throwaway root is written. Cached Area 1 libraries are read.
"""
from pathlib import Path
import datetime
import json
import os
import re
import subprocess
import tomllib

REPO = Path(__file__).resolve().parents[3]
BASE = Path(r"D:\Notaya Tests\v1-6-S")
CACHE = BASE / "area1-proof-20261109T075233781764Z" / "target" / "debug"
STAMP = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
RUN = BASE / ("area2-proof-" + STAMP)
RUN.mkdir(parents=True, exist_ok=False)
src = REPO / "src-tauri"
cached_root = CACHE.parent.parent
def packages(path):
    return {(p["name"], p["version"], p.get("source"), p.get("checksum"))
            for p in tomllib.loads(path.read_text(encoding="utf-8"))["package"]}
assert packages(cached_root / "Cargo.lock") <= packages(src / "Cargo.lock"), "cached dependency identity differs"
assert tomllib.loads((cached_root / "Cargo.toml").read_text(encoding="utf-8"))["dependencies"] == tomllib.loads((src / "Cargo.toml").read_text(encoding="utf-8"))["dependencies"], "cached dependency declarations differ"
modules = re.findall(r"^mod (\w+);", (src / "src/lib.rs").read_text(encoding="utf-8"), re.M)
prefix = "#![allow(dead_code)]\n"
for name in modules:
    if name == "roundtrip_tests":
        continue
    path = src / "src" / (name + ".rs")
    if not path.exists():
        path = src / "src" / name / "mod.rs"
    prefix += '#[path = r"' + str(path) + '"] mod ' + name + ';\n'
(RUN / "proof.rs").write_text(prefix + Path(__file__).with_name("core-repro.rs").read_text(encoding="utf-8"), encoding="utf-8")
manifest = (src / "Cargo.toml").read_text(encoding="utf-8")
dependencies = manifest.split("[dependencies]", 1)[1].split("[profile.release]", 1)[0]
fingerprints = list((CACHE / ".fingerprint").glob("notaya-app-*/bin-area1-core.json"))
assert len(fingerprints) == 1, "expected the preserved Area 1 build fingerprint"
dep_hashes = {d[1]: d[3].to_bytes(8, "little").hex()
              for d in json.loads(fingerprints[0].read_text())["deps"]}
cmd = ["rustc", "--edition=2024", "--crate-name", "area2_core", str(RUN / "proof.rs"),
       "-L", "dependency=" + str(CACHE / "deps"), "-o", str(RUN / "area2-core.exe")]
for name in re.findall(r"^([a-zA-Z0-9_-]+)\s*=", dependencies, re.M):
    crate = name.replace("-", "_")
    matches = list((CACHE / "deps").glob("lib" + crate + "-*.rlib"))
    matches = [p for p in matches if
               (CACHE / ".fingerprint" / (name + p.stem[-17:]) / ("lib-" + crate)).read_text().strip()
               == dep_hashes[crate]]
    assert len(matches) == 1, (crate, "expected exactly one cached library")
    cmd += ["--extern", crate + "=" + str(matches[0])]
for output in (CACHE / "build").glob("*/output"):
    for native in re.findall(r"^cargo::?rustc-link-search=native=(.+)$", output.read_text(encoding="utf-8"), re.M):
        cmd += ["-L", "native=" + native.strip()]
env = os.environ.copy()
env["CARGO_PKG_VERSION"] = re.search(r'^version = "([^"]+)"', manifest, re.M)[1]
env["CARGO_PKG_NAME"] = "notaya-app"
env["CARGO_MANIFEST_DIR"] = str(src)
env["NOTAYA_HOME"] = str(RUN / "data")
print("Proof execution directory:", RUN, flush=True)
compiled = subprocess.run(cmd, cwd=REPO, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
(RUN / "compile.txt").write_text(compiled.stdout, encoding="utf-8")
print(compiled.stdout[-4000:], flush=True)
if compiled.returncode:
    raise SystemExit(compiled.returncode)
result = subprocess.run([str(RUN / "area2-core.exe"), str(RUN / "data")], cwd=RUN, env=env,
                        text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
(RUN / "result.txt").write_text(result.stdout, encoding="utf-8")
print(result.stdout, flush=True)
raise SystemExit(result.returncode)
