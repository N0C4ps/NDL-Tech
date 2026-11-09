#!/usr/bin/env python3
# tags: [architecture, isolation, lint, cargo, ci]
"""Check that NDL crates never runtime-depend on another ndl-* crate.

Enforces the crate-isolation rule in docs/architecture.md: every crate in
crates/ (except the `ndl` facade, whose whole job is to re-export the others)
must not declare another `ndl-*` crate under [dependencies] or
[target.*.dependencies]. An `ndl-*` crate is only allowed to appear under
[dev-dependencies] (e.g. ndl-render pulls in ndl-math only for its tests).
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CRATES_DIR = REPO_ROOT / "crates"
FACADE_CRATE = "ndl"

SECTION_RE = re.compile(r"^\[([^\]]+)\]\s*$")
DEP_LINE_RE = re.compile(r"^\s*(ndl(?:-[a-zA-Z0-9_]+)?)\s*=")


def parse_sections(cargo_toml: Path) -> dict[str, list[str]]:
    """Return {section_name: [raw_lines]} for every section in the file."""
    sections: dict[str, list[str]] = {}
    current = "(preamble)"
    sections[current] = []
    for line in cargo_toml.read_text(encoding="utf-8").splitlines():
        match = SECTION_RE.match(line.strip())
        if match:
            current = match.group(1)
            sections.setdefault(current, [])
            continue
        sections[current].append(line)
    return sections


def runtime_dependency_sections(sections: dict[str, list[str]]) -> dict[str, list[str]]:
    """Sections that count as runtime deps: [dependencies] and
    [target.'cfg(...)'.dependencies], but never anything with 'dev-dependencies'
    or 'build-dependencies' in its name."""
    result = {}
    for name, lines in sections.items():
        if "dev-dependencies" in name or "build-dependencies" in name:
            continue
        if name == "dependencies" or name.endswith(".dependencies"):
            result[name] = lines
    return result


def find_ndl_deps(lines: list[str]) -> list[str]:
    found = []
    for line in lines:
        match = DEP_LINE_RE.match(line)
        if match:
            found.append(match.group(1))
    return found


def check_crate(cargo_toml: Path) -> list[str]:
    crate_name = cargo_toml.parent.name
    violations = []
    if crate_name == FACADE_CRATE:
        return violations

    sections = parse_sections(cargo_toml)
    for section_name, lines in runtime_dependency_sections(sections).items():
        for dep in find_ndl_deps(lines):
            if dep == crate_name:
                continue
            violations.append(
                f"{crate_name}: `{dep}` is a runtime dependency in [{section_name}] "
                f"({cargo_toml.relative_to(REPO_ROOT)}) — only [dev-dependencies] may "
                f"reference another ndl-* crate (see docs/architecture.md)"
            )
    return violations


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")

    parser = argparse.ArgumentParser(
        description=(
            "Fail if any NDL crate (other than the `ndl` facade) declares a "
            "runtime dependency on another ndl-* crate, instead of only in "
            "[dev-dependencies]."
        )
    )
    parser.add_argument(
        "--crates-dir",
        type=Path,
        default=CRATES_DIR,
        help=f"directory containing one subfolder per crate (default: {CRATES_DIR})",
    )
    args = parser.parse_args()

    if not args.crates_dir.is_dir():
        print(f"error: crates dir not found: {args.crates_dir}", file=sys.stderr)
        return 2

    all_violations: list[str] = []
    checked = 0
    for cargo_toml in sorted(args.crates_dir.glob("*/Cargo.toml")):
        checked += 1
        all_violations.extend(check_crate(cargo_toml))

    if checked == 0:
        print(f"error: no Cargo.toml found under {args.crates_dir}", file=sys.stderr)
        return 2

    if all_violations:
        print("crate isolation violations found:\n")
        for violation in all_violations:
            print(f"  - {violation}")
        print(f"\n{len(all_violations)} violation(s) across {checked} crate(s) checked.")
        return 1

    print(f"OK: {checked} crate(s) checked, no isolation violations.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
