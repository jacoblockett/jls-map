#!/usr/bin/env python3

import argparse
import hashlib
import json
import re
import shutil
import zipfile
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
SEMVER = re.compile(r"^\d+\.\d+\.\d+$")
SKILL_META = re.compile(r"<!--\s*jls-meta:\s*(\{.*?\})\s*-->")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def safe_rel(raw: str, label: str) -> Path:
    if not isinstance(raw, str) or not raw.strip():
        raise ValueError(f"{label} must be a non-empty relative path")
    posix = PurePosixPath(raw)
    if posix.is_absolute() or ".." in posix.parts:
        raise ValueError(f"{label} must stay within the package")
    return Path(*posix.parts)


def copy_declared(source_root: Path, stage_root: Path, raw: str, label: str) -> None:
    rel = safe_rel(raw, label)
    source = source_root / rel
    destination = stage_root / rel
    if not source.exists():
        raise FileNotFoundError(f"missing {label}: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source.is_dir():
        shutil.copytree(source, destination, dirs_exist_ok=True)
    else:
        shutil.copy2(source, destination)


def load_manifest() -> dict:
    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    if manifest.get("format") != 1:
        raise ValueError("manifest format must be 1")
    if manifest.get("name") != "map":
        raise ValueError("manifest name must be map")
    if not SEMVER.fullmatch(str(manifest.get("version", ""))):
        raise ValueError("manifest version must be plain semver")
    if not SEMVER.fullmatch(str(manifest.get("min_installer", ""))):
        raise ValueError("min_installer must be plain semver")
    if manifest.get("runtime") != "rust" or manifest.get("runtime_cli") != "map":
        raise ValueError("Map must declare its Rust map runtime")
    return manifest


def validate_skill_metadata(manifest: dict) -> None:
    text = (ROOT / "SKILL.md").read_text(encoding="utf-8")
    match = SKILL_META.search(text)
    if not match:
        raise ValueError("SKILL.md is missing jls metadata")
    metadata = json.loads(match.group(1))
    if metadata != {"name": "map", "version": manifest["version"], "format": 1}:
        raise ValueError("SKILL.md metadata must match manifest name/version and format 1")


def declared_assets(manifest: dict) -> list[str]:
    assets: set[str] = set(manifest.get("skill_files", []))
    for harness in manifest.get("harness_resources", {}).values():
        for paths in harness.values():
            assets.update(paths)
    assets.update(manifest.get("runtime_files", []))
    fragment = manifest.get("instruction_fragment")
    if fragment:
        assets.add(fragment)
    return sorted(assets)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--runtime", required=True)
    parser.add_argument("--release-tag", required=True)
    parser.add_argument("--output", default="build")
    args = parser.parse_args()

    manifest = load_manifest()
    validate_skill_metadata(manifest)

    runtime_artifacts = manifest.get("runtime_artifacts") or {}
    runtime_rel = runtime_artifacts.get(args.target)
    if not runtime_rel:
        raise ValueError(f"manifest has no runtime artifact for {args.target}")

    runtime = Path(args.runtime).resolve()
    if not runtime.is_file():
        raise FileNotFoundError(f"runtime does not exist: {runtime}")

    output = (ROOT / args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    stage = output / f".stage-map-{args.target}"
    shutil.rmtree(stage, ignore_errors=True)
    stage.mkdir(parents=True)

    package_manifest = dict(manifest)
    package_manifest["runtime_artifacts"] = {args.target: runtime_rel}
    (stage / "manifest.json").write_text(
        json.dumps(package_manifest, indent=2) + "\n",
        encoding="utf-8",
    )

    for asset in declared_assets(manifest):
        copy_declared(ROOT, stage, asset, "package asset")

    runtime_destination = stage / safe_rel(runtime_rel, "runtime artifact")
    runtime_destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(runtime, runtime_destination)

    archive_name = f"map-{args.target}.zip"
    archive = output / archive_name
    archive.unlink(missing_ok=True)
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as package:
        for path in sorted(p for p in stage.rglob("*") if p.is_file()):
            package.write(path, path.relative_to(stage).as_posix())

    with zipfile.ZipFile(archive, "r") as package:
        packaged_manifest = json.loads(package.read("manifest.json"))
        if packaged_manifest.get("runtime_artifacts") != {args.target: runtime_rel}:
            raise ValueError("packaged manifest contains the wrong runtime target set")
        if runtime_rel not in package.namelist():
            raise ValueError("packaged runtime is missing")

    artifact_hash = sha256(archive)
    base = f"https://github.com/jacoblockett/jls-map/releases/download/{args.release_tag}"
    fragment = {
        "format": 1,
        "name": "map",
        "version": manifest["version"],
        "min_installer": manifest["min_installer"],
        "target": args.target,
        "artifact": {
            "name": archive_name,
            "url": f"{base}/{archive_name}",
            "sha256": artifact_hash,
        },
    }
    (output / f"fragment-{args.target}.json").write_text(
        json.dumps(fragment, indent=2) + "\n",
        encoding="utf-8",
    )

    shutil.rmtree(stage)
    print(f"Built {archive}")


if __name__ == "__main__":
    main()
