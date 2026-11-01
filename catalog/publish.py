#!/usr/bin/env python3
"""Publish a mod to the F1 Multiplayer addon catalog, without a launcher update.

    python catalog/publish.py <addon folder> --repo <owner>/<name>

The script packs the addon (tools/pack_addon.py), works out its size and SHA-256, and adds or
replaces that version in catalog/v1/addons. The launcher reads that file from the repository's
main branch, so the new mod shows in the Addon Browser after the next catalog refresh.

It does not upload anything. It prints the GitHub CLI commands that upload the packed file as a
release asset; run them, then commit and push catalog/v1/addons. The asset's SHA-256 is checked by
the launcher before it installs, so upload exactly the file this script packed.
"""

import argparse
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, ".."))
PACK = os.path.join(ROOT, "tools", "pack_addon.py")
CATALOG = os.path.join(HERE, "v1", "addons")


def now():
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def sha256_and_size(path):
    digest = hashlib.sha256()
    size = 0
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            digest.update(chunk)
            size += len(chunk)
    return digest.hexdigest(), size


def load_catalog(path):
    if not os.path.exists(path):
        return {"addons": []}
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("addon", help="the addon folder, with swofty.json at its root")
    ap.add_argument("--repo", required=True, help="GitHub repository, owner/name")
    ap.add_argument("--catalog", default=CATALOG, help="the catalog file to update")
    ap.add_argument("--out", default=os.path.join(ROOT, "build", "addons"), help="where the packed file goes")
    args = ap.parse_args()

    manifest_path = os.path.join(args.addon, "swofty.json")
    with open(manifest_path, encoding="utf-8") as f:
        m = json.load(f)
    addon_id, version = m["id"], m["version"]

    os.makedirs(args.out, exist_ok=True)
    subprocess.run([sys.executable, PACK, args.addon, args.out], check=True)
    packed = os.path.join(args.out, f"{addon_id}.swoftyaddon")
    if not os.path.exists(packed):
        sys.exit(f"the packer did not write {packed}")

    asset_name = f"{addon_id}-{version}.swoftyaddon"
    asset = os.path.join(args.out, asset_name)
    shutil.copyfile(packed, asset)
    sha, size = sha256_and_size(asset)
    tag = f"addon-{addon_id}-{version}"
    url = f"https://github.com/{args.repo}/releases/download/{tag}/{asset_name}"

    compat = m.get("compat", {})
    entry_version = {
        "version": version,
        "launcher": compat.get("launcher"),
        "game": compat.get("game", ""),
        "size": size,
        "sha256": sha,
        "url": url,
        "published_at": now(),
    }

    catalog = load_catalog(args.catalog)
    addons = catalog.setdefault("addons", [])
    existing = next((a for a in addons if a.get("id") == addon_id), None)
    if existing is None:
        existing = {"id": addon_id, "created_at": now(), "versions": []}
        addons.append(existing)
    existing.update({
        "name": m.get("name", addon_id),
        "author": m.get("author", ""),
        "summary": m.get("summary", (m.get("description") or "")[:140]),
        "description": m.get("description", ""),
        "category": m.get("category"),
        "kind": m.get("kind", []),
        "homepage": m.get("homepage"),
        "updated_at": now(),
        "latest": entry_version,
    })
    existing["versions"] = [v for v in existing.get("versions", []) if v.get("version") != version]
    existing["versions"].append(entry_version)

    os.makedirs(os.path.dirname(args.catalog), exist_ok=True)
    with open(args.catalog, "w", encoding="utf-8", newline="\n") as f:
        json.dump(catalog, f, indent=2, ensure_ascii=False)
        f.write("\n")

    print(f"packed {asset}")
    print(f"size {size} bytes, sha256 {sha}")
    print(f"catalog updated: {args.catalog}")
    print()
    print("Upload the asset (first time for this version):")
    print(f'  gh release create {tag} "{asset}" --repo {args.repo} --title "{m.get("name", addon_id)} {version}" --notes "{addon_id} {version}"')
    print("or, if the release exists:")
    print(f'  gh release upload {tag} "{asset}" --repo {args.repo}')
    print()
    print("Then commit and push catalog/v1/addons.")


if __name__ == "__main__":
    main()
