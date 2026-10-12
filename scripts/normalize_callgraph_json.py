#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path


def normalize(value, root, home):
    if isinstance(value, str):
        value = value.replace(root + "/", "")
        value = value.replace(home + "/", "~/")
        value = re.sub(r"~/.rustup/toolchains/[^/]+/", "~/.rustup/toolchains/<toolchain>/", value)
        return re.sub(r"~/.cargo/registry/src/[^/]+/", "~/.cargo/registry/src/<registry>/", value)
    if isinstance(value, list):
        return [normalize(item, root, home) for item in value]
    if isinstance(value, dict):
        return {key: normalize(item, root, home) for key, item in value.items()}
    return value


def main():
    if len(sys.argv) != 3:
        print("usage: normalize_callgraph_json.py INPUT OUTPUT", file=sys.stderr)
        return 2

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    root = str(Path.cwd().resolve())
    home = str(Path.home().resolve())

    with input_path.open() as input_file:
        data = json.load(input_file)

    normalized = normalize(data, root, home)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w") as output_file:
        json.dump(normalized, output_file, indent=2)
        output_file.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
