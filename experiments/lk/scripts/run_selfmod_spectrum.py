#!/usr/bin/env python3
"""Freeze the local instrument and run a fresh v11 directory. No overwrite."""
import argparse
import datetime
import hashlib
import json
import pathlib
import shutil
import subprocess
import tarfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=pathlib.Path)
    parser.add_argument("--preflight", action="store_true")
    args = parser.parse_args()
    subprocess.run(["cargo", "build", "--offline", "--locked", "--release",
                    "-p", "hg-selfmod", "--bin", "selfmod-spectrum"], cwd=ROOT, check=True)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    source_paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "provenance.json",
                    ROOT / "docs/selfmod_v11_spectrum_preregistration.md"]
    for folder in ("crates", "configs", "scripts", "vendor"):
        source_paths.extend(p for p in (ROOT / folder).rglob("*")
                            if p.is_file() and not set(p.relative_to(ROOT).parts) & {"target", "__pycache__"})
    sources = {str(p.relative_to(ROOT)): digest(p) for p in sorted(source_paths)}
    archive = out / "source.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        for path in sorted(source_paths):
            tar.add(path, arcname=str(path.relative_to(ROOT)), recursive=False)
    binary = out / "selfmod-spectrum"
    shutil.copy2(ROOT / "target/release/selfmod-spectrum", binary)
    provenance = {"frozen_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  "engineering_preflight": args.preflight,
                  "sources_sha256": sources, "source_archive_sha256": digest(archive),
                  "binary_sha256": digest(binary),
                  "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
                  "build_command": "cargo build --offline --locked --release -p hg-selfmod --bin selfmod-spectrum"}
    (out / "instrument_provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    command = [str(binary)] + (["--preflight"] if args.preflight else []) + [str(out)]
    with (out / "run.log").open("x") as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            print(line, end="", flush=True)
            log.write(line)
            log.flush()
        code = process.wait()
    if code:
        raise SystemExit(code)
    print(f"Frozen evidence: {out}", flush=True)


if __name__ == "__main__":
    main()
