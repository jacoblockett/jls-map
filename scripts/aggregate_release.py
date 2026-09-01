#!/usr/bin/env python3

import argparse
import hashlib
import json
import re
import shutil
import zipfile
from pathlib import Path

TARGETS = [
    "windows-x64",
    "windows-arm64",
    "macos-x64",
    "macos-arm64",
    "linux-x64-gnu",
    "linux-arm64-gnu",
    "linux-x64-musl",
    "linux-arm64-musl",
]
SEMVER = re.compile(r"^\d+\.\d+\.\d+$")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", default="build/targets")
    parser.add_argument("--output", default="build/release")
    parser.add_argument("--release-tag", required=True)
    args = parser.parse_args()

    input_root = Path(args.input).resolve()
    output = Path(args.output).resolve()
    shutil.rmtree(output, ignore_errors=True)
    output.mkdir(parents=True)

    fragments: dict[str, tuple[dict, Path]] = {}
    for path in input_root.rglob("fragment-*.json"):
        fragment = json.loads(path.read_text(encoding="utf-8"))
        target = fragment.get("target")
        if target not in TARGETS:
            raise ValueError(f"invalid target fragment: {target}")
        if target in fragments:
            raise ValueError(f"duplicate target fragment: {target}")
        fragments[target] = (fragment, path)

    missing = [target for target in TARGETS if target not in fragments]
    if missing:
        raise ValueError(f"missing target fragments: {', '.join(missing)}")

    versions = {fragment[0].get("version") for fragment in fragments.values()}
    minimums = {fragment[0].get("min_installer") for fragment in fragments.values()}
    names = {fragment[0].get("name") for fragment in fragments.values()}
    formats = {fragment[0].get("format") for fragment in fragments.values()}
    if names != {"map"} or formats != {1}:
        raise ValueError("target fragments disagree on Map identity or format")
    if len(versions) != 1 or not SEMVER.fullmatch(str(next(iter(versions)))):
        raise ValueError("target fragments disagree on Map version")
    if len(minimums) != 1 or not SEMVER.fullmatch(str(next(iter(minimums)))):
        raise ValueError("target fragments disagree on minimum installer version")

    version = next(iter(versions))
    min_installer = next(iter(minimums))
    artifacts: dict[str, dict[str, str]] = {}
    release_base = f"https://github.com/jacoblockett/jls-map/releases/download/{args.release_tag}"

    for target in TARGETS:
        fragment, fragment_path = fragments[target]
        artifact = fragment.get("artifact") or {}
        expected_name = f"map-{target}.zip"
        archive = fragment_path.parent / expected_name
        if artifact.get("name") != expected_name or not archive.is_file():
            raise ValueError(f"missing or misnamed archive for {target}")
        expected_url = f"{release_base}/{expected_name}"
        if artifact.get("url") != expected_url:
            raise ValueError(f"unexpected release URL for {target}")
        actual_hash = sha256(archive)
        if artifact.get("sha256") != actual_hash:
            raise ValueError(f"SHA-256 mismatch for {target}")

        with zipfile.ZipFile(archive, "r") as package:
            package_manifest = json.loads(package.read("manifest.json"))
            if package_manifest.get("name") != "map" or package_manifest.get("version") != version:
                raise ValueError(f"package metadata mismatch for {target}")
            runtime_artifacts = package_manifest.get("runtime_artifacts") or {}
            if list(runtime_artifacts) != [target]:
                raise ValueError(f"package for {target} does not contain exactly its own runtime mapping")
            runtime_path = runtime_artifacts[target]
            if runtime_path not in package.namelist():
                raise ValueError(f"package for {target} is missing its runtime")

        shutil.copy2(archive, output / expected_name)
        artifacts[target] = {
            "url": expected_url,
            "sha256": actual_hash,
        }

    release_manifest = {
        "format": 1,
        "name": "map",
        "version": version,
        "min_installer": min_installer,
        "artifacts": artifacts,
    }
    (output / "manifest.json").write_text(
        json.dumps(release_manifest, indent=2) + "\n",
        encoding="utf-8",
    )

    files = sorted(path.name for path in output.iterdir() if path.is_file())
    expected = sorted([f"map-{target}.zip" for target in TARGETS] + ["manifest.json"])
    if files != expected:
        raise ValueError(f"unexpected release bundle: {files}")

    print(f"Built verified Map {version} release bundle")


if __name__ == "__main__":
    main()
