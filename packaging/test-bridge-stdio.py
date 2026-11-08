"""Exercise the built bridge over real stdio, without a vault, pipe or AI client."""
import json
import os
from pathlib import Path
import subprocess
import time


def main():
    home = Path(os.environ.get("NOTAYA_HOME", ""))
    if str(home) != r"D:\Notaya Tests\v1-6-D":
        raise RuntimeError("Set NOTAYA_HOME to the lane D throwaway folder first.")
    bridge = Path(os.environ["CARGO_TARGET_DIR"]) / "debug" / "notaya-bridge.exe"
    isolated = home / f"stdio-{os.getpid()}-{time.time_ns()}"
    isolated.mkdir()
    # This directory deliberately has no data folder, so the bridge cannot open any pipe.
    env = dict(os.environ, NOTAYA_HOME=str(isolated))

    def run(data=b"", args=()):
        return subprocess.run(
            [str(bridge), *args], input=data, capture_output=True, env=env,
            timeout=10, check=False,
        )

    def request(identifier, method, params=None):
        value = {"jsonrpc": "2.0", "id": identifier, "method": method}
        if params is not None:
            value["params"] = params
        return value

    messages = [
        request("init", "initialize", {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "stdio fixture", "version": "1"}}),
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        request(2, "tools/list"),
        request("read", "tools/call", {"name": "notaya_page", "arguments": {"action": "page.get", "params": {}}}),
        request(4, "tools/call", {"name": "notaya_session", "arguments": {"action": "ai_confirm_answer", "params": {"private": "ERROR MUST NOT ECHO THIS"}}}),
        request(5, "unknown"),
        request(6, "ping"),
        {"jsonrpc": "2.0", "method": "unknown_notification"},
    ]
    data = b"".join((json.dumps(m) + "\n").encode() for m in messages) + b"invalid json\n"
    result = run(data)
    assert result.returncode == 0, result.stderr
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    assert len(replies) == 7, replies
    by_id = {reply["id"]: reply for reply in replies}
    assert by_id["init"]["result"]["protocolVersion"] == "2025-11-25"
    assert by_id[2]["result"]["tools"] == []
    assert by_id["read"]["result"]["isError"] is True
    assert "not_running" in by_id["read"]["result"]["content"][0]["text"]
    assert by_id[4]["error"]["code"] == -32602
    assert by_id[5]["error"]["code"] == -32601
    assert by_id[6]["result"] == {}
    assert by_id[None]["error"]["code"] == -32700
    assert b"ERROR MUST NOT ECHO THIS" not in result.stdout + result.stderr

    help_result = run(args=("--help",))
    assert help_result.returncode == 0 and help_result.stdout == b""
    invalid = run(args=("--unknown",))
    assert invalid.returncode == 1 and invalid.stdout == b""
    oversized = run(b"x" * (8 * 1024 * 1024 + 1) + b"\n")
    assert oversized.returncode == 1 and oversized.stdout == b""
    assert list(isolated.iterdir()) == [], "The bridge wrote outside a pairing operation."
    print("PASS: real bridge stdio, lifecycle, tools, fixed errors, malformed/oversized input, stdout isolation, EOF and zero disk writes (no pipe or vault).")


if __name__ == "__main__":
    main()
