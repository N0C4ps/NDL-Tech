#!/usr/bin/env python3
# tags: [playground, local, cargo, path-dependency]
"""Toggle playground/Cargo.toml between the git dependency on `ndl` and a
local path dependency, to test unpushed local crate changes through the
playground (see docs/playground.md and how-tos/test-local-changes-in-playground.md).

Modes:
  status  - show which dependency form is currently active (default)
  on      - switch to a local `path = "../crates/ndl"` dependency
  off     - switch back to the git dependency (the committed default)

Always run `off` before committing — playground/ is gitignored, but the
local-path form only exists for your machine and must not be relied on by
anyone else.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_PLAYGROUND_CARGO_TOML = REPO_ROOT / "playground" / "Cargo.toml"

GIT_DEP_RE = re.compile(
    r'ndl\s*=\s*\{\s*git\s*=\s*"[^"]+".*\}',
)
LOCAL_DEP_RE = re.compile(
    r'ndl\s*=\s*\{\s*path\s*=\s*"[^"]+".*\}',
)

# The exact git dependency line this repo's playground/Cargo.toml ships with.
GIT_DEP_TEMPLATE = (
    'ndl = {{ git = "https://github.com/N0C4ps/NDL-Tech.git", features = {features} }}'
)
LOCAL_DEP_TEMPLATE = 'ndl = {{ path = "../crates/ndl", features = {features} }}'

FEATURES_RE = re.compile(r"features\s*=\s*(\[[^\]]*\])")


def read_features(line: str) -> str:
    match = FEATURES_RE.search(line)
    if not match:
        raise ValueError(f"could not find `features = [...]` in dependency line: {line!r}")
    return match.group(1)


def current_status(text: str) -> str:
    for line in text.splitlines():
        if GIT_DEP_RE.search(line):
            return "git"
        if LOCAL_DEP_RE.search(line):
            return "local"
    raise ValueError("no `ndl = { git = ... }` or `ndl = { path = ... }` dependency line found")


def switch(text: str, target: str) -> str:
    lines = text.splitlines(keepends=True)
    out = []
    switched = False
    for line in lines:
        if GIT_DEP_RE.search(line) or LOCAL_DEP_RE.search(line):
            features = read_features(line)
            newline_suffix = "\n" if line.endswith("\n") else ""
            template = GIT_DEP_TEMPLATE if target == "git" else LOCAL_DEP_TEMPLATE
            out.append(template.format(features=features) + newline_suffix)
            switched = True
            continue
        out.append(line)
    if not switched:
        raise ValueError("no `ndl` dependency line found to switch")
    return "".join(out)


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")

    parser = argparse.ArgumentParser(
        description=(
            "Switch playground/Cargo.toml's `ndl` dependency between the git URL "
            "(default, what's committed) and a local path (../crates/ndl), to test "
            "unpushed local changes through the playground."
        )
    )
    parser.add_argument(
        "mode",
        nargs="?",
        default="status",
        choices=["status", "on", "off"],
        help="status: show current mode (default); on: use local path; off: use git dependency",
    )
    parser.add_argument(
        "--file",
        type=Path,
        default=DEFAULT_PLAYGROUND_CARGO_TOML,
        help=f"path to playground's Cargo.toml (default: {DEFAULT_PLAYGROUND_CARGO_TOML})",
    )
    args = parser.parse_args()

    if not args.file.is_file():
        print(f"error: not found: {args.file}", file=sys.stderr)
        return 2

    text = args.file.read_text(encoding="utf-8")

    try:
        if args.mode == "status":
            status = current_status(text)
            label = "local path (../crates/ndl)" if status == "local" else "git (committed default)"
            print(f"{args.file}: using {label}")
            return 0

        target = "local" if args.mode == "on" else "git"
        current = current_status(text)
        if current == target:
            print(f"{args.file}: already using {'local path' if target == 'local' else 'git'}, no change.")
            return 0

        updated = switch(text, target)
        args.file.write_text(updated, encoding="utf-8")
        label = "local path (../crates/ndl)" if target == "local" else "git (committed default)"
        print(f"{args.file}: switched to {label}.")
        if target == "local":
            print("remember to run `playground_use_local.py off` before committing.")
        return 0
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
