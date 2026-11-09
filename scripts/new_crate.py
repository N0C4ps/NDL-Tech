#!/usr/bin/env python3
# tags: [scaffold, crate, workspace, facade]
"""Scaffold a new ndl-* crate and wire it into the `ndl` facade crate.

Creates crates/<name>/{Cargo.toml,src/lib.rs} following the repo's standard
stub template (see docs/architecture.md), then registers the crate as an
optional dependency + feature flag of crates/ndl, and adds the matching
`pub use` re-export to crates/ndl/src/lib.rs — the same wiring described in
how-tos/add-new-crate.md.

Use --dry-run to preview every change without writing anything.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CRATES_DIR = REPO_ROOT / "crates"
FACADE_DIR = CRATES_DIR / "ndl"


def feature_name(crate_name: str) -> str:
    return crate_name.removeprefix("ndl-")


def crate_cargo_toml(crate_name: str, description: str) -> str:
    return f"""[package]
name = "{crate_name}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "{description}"
"""


def crate_lib_rs(crate_name: str, description: str) -> str:
    return f"""//! `{crate_name}` {description}
//!
//! Não conhece o domínio de quem a consome, como as demais crates do
//! ecossistema NDL (ver docs/architecture.md no root do workspace).
//!
//! Ainda não implementada: esta crate é um stub nesta fase do projeto.
"""


def insert_optional_dependency(cargo_toml_text: str, crate_name: str) -> str:
    lines = cargo_toml_text.splitlines(keepends=True)
    dep_line = f'{crate_name} = {{ path = "../{crate_name}", optional = true }}\n'

    out: list[str] = []
    in_deps = False
    inserted = False
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped == "[dependencies]":
            in_deps = True
            out.append(line)
            continue
        if in_deps and stripped.startswith("[") and stripped != "[dependencies]":
            out.append(dep_line)
            inserted = True
            in_deps = False
        out.append(line)
    if in_deps and not inserted:
        out.append(dep_line)
        inserted = True
    if not inserted:
        raise ValueError("could not find a [dependencies] section to insert into")
    return "".join(out)


def insert_feature(cargo_toml_text: str, crate_name: str, feature: str) -> str:
    lines = cargo_toml_text.splitlines(keepends=True)
    feature_line = f'{feature} = ["{crate_name}"]\n'

    out: list[str] = []
    i = 0
    inserted_feature = False
    in_full_array = False
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        if stripped.startswith("full = ["):
            if not inserted_feature:
                out.append(feature_line)
                inserted_feature = True
            if stripped.endswith("]"):
                # single-line `full = [...]`
                inner = stripped[len("full = ["):-1]
                entries = [e.strip() for e in inner.split(",") if e.strip()]
                entries.append(f'"{feature}"')
                out.append("full = [" + ", ".join(entries) + "]\n")
                i += 1
                continue
            in_full_array = True
            out.append(line)
            i += 1
            continue
        if in_full_array:
            if stripped == "]":
                out.append(f'    "{feature}",\n')
                out.append(line)
                in_full_array = False
                i += 1
                continue
            out.append(line)
            i += 1
            continue
        out.append(line)
        i += 1

    if not inserted_feature:
        raise ValueError("could not find a `full = [...]` feature array to insert into")
    return "".join(out)


def insert_facade_reexport(lib_rs_text: str, crate_name: str, feature: str) -> str:
    rust_crate_ident = crate_name.replace("-", "_")
    block = (
        f'\n#[cfg(feature = "{feature}")]\n'
        f"pub use {rust_crate_ident} as {feature};\n"
    )
    return lib_rs_text.rstrip("\n") + "\n" + block


def main() -> int:
    # Windows consoles often default stdout to a legacy codepage, which
    # mangles the accented Portuguese text in descriptions/doc comments —
    # force UTF-8 regardless of terminal/redirection.
    sys.stdout.reconfigure(encoding="utf-8")

    parser = argparse.ArgumentParser(
        description=__doc__.strip().splitlines()[0],
        epilog=(
            "Note (Windows): if accented characters in `description` come out "
            "mangled in the files this writes, re-run with PYTHONUTF8=1 set "
            "(e.g. `PYTHONUTF8=1 python3 scripts/new_crate.py ...`) — some "
            "Windows shells pass non-ASCII arguments through the legacy ANSI "
            "codepage instead of UTF-8 unless Python's UTF-8 mode is forced."
        ),
    )
    parser.add_argument("name", help='crate name, e.g. "ndl-audio" (must start with "ndl-")')
    parser.add_argument(
        "description",
        help="one-line description, used as the Cargo.toml `description` and lib.rs doc intro",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print every file that would be created/modified instead of writing them",
    )
    args = parser.parse_args()

    crate_name = args.name
    if not re.fullmatch(r"ndl-[a-z0-9][a-z0-9-]*", crate_name):
        print('error: crate name must look like "ndl-something" (lowercase, hyphenated)', file=sys.stderr)
        return 2

    crate_dir = CRATES_DIR / crate_name
    if crate_dir.exists():
        print(f"error: {crate_dir} already exists", file=sys.stderr)
        return 2

    feature = feature_name(crate_name)
    facade_cargo = FACADE_DIR / "Cargo.toml"
    facade_lib = FACADE_DIR / "src" / "lib.rs"
    if not facade_cargo.is_file() or not facade_lib.is_file():
        print(f"error: facade crate not found at {FACADE_DIR}", file=sys.stderr)
        return 2

    new_cargo_toml = crate_cargo_toml(crate_name, args.description)
    new_lib_rs = crate_lib_rs(crate_name, args.description)

    updated_facade_cargo = insert_feature(
        insert_optional_dependency(facade_cargo.read_text(encoding="utf-8"), crate_name),
        crate_name,
        feature,
    )
    updated_facade_lib = insert_facade_reexport(
        facade_lib.read_text(encoding="utf-8"), crate_name, feature
    )

    changes = [
        (crate_dir / "Cargo.toml", new_cargo_toml),
        (crate_dir / "src" / "lib.rs", new_lib_rs),
        (facade_cargo, updated_facade_cargo),
        (facade_lib, updated_facade_lib),
    ]

    if args.dry_run:
        for path, content in changes:
            print(f"--- {path.relative_to(REPO_ROOT)} " + "-" * 40)
            print(content)
        print(
            f"(dry run) would create {crate_name} and wire feature `{feature}` into "
            f"the ndl facade. Remember to also add a row to the README crate table."
        )
        return 0

    (crate_dir / "src").mkdir(parents=True, exist_ok=False)
    (crate_dir / "Cargo.toml").write_text(new_cargo_toml, encoding="utf-8")
    (crate_dir / "src" / "lib.rs").write_text(new_lib_rs, encoding="utf-8")
    facade_cargo.write_text(updated_facade_cargo, encoding="utf-8")
    facade_lib.write_text(updated_facade_lib, encoding="utf-8")

    print(f"created crates/{crate_name} and wired feature `{feature}` into the ndl facade.")
    print("remaining manual step: add a row for it to the crate table in README.md.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
