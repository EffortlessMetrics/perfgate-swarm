#!/usr/bin/env python3
import json
from pathlib import Path


def escape_command_value(value):
    return (
        str(value)
        .replace("%", "%25")
        .replace("\r", "%0D")
        .replace("\n", "%0A")
    )


def escape_command_property(value):
    return escape_command_value(value).replace(":", "%3A").replace(",", "%2C")


path = Path("target/ripr/review/comments.json")
if not path.exists():
    raise SystemExit(0)

data = json.loads(path.read_text(encoding="utf-8"))

for item in data.get("comments", []):
    file = item.get("path") or item.get("file")
    line = item.get("line")
    title = item.get("title") or "RIPR"
    body = item.get("body") or item.get("message") or ""

    if not file or not line:
        continue

    file = escape_command_property(file)
    line = escape_command_property(line)
    title = escape_command_property(title)
    body = escape_command_value(body)
    print(f"::warning file={file},line={line},title={title}::{body}")
