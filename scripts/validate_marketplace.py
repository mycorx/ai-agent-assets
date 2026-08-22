#!/usr/bin/env python3
"""Validate this repo's marketplace manifest and every cataloged plugin:
marketplace.json parses and carries the required fields, each plugin's source
resolves to a local directory with a valid plugin.json, command/skill
frontmatter is present, and README links resolve. No external services or
dependencies."""

import json
import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
MARKETPLACE_FIELDS = ["name", "owner", "plugins"]
MANIFEST_FIELDS = ["name", "version", "description"]
SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+$")
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")

errors = []


def err(msg):
    errors.append(msg)


def load_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        err(f"{path.relative_to(ROOT)}: invalid JSON ({e})")
        return None
    except FileNotFoundError:
        err(f"{path.relative_to(ROOT)}: file not found")
        return None


def parse_frontmatter(path):
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        err(f"{path.relative_to(ROOT)}: missing frontmatter block")
        return {}
    try:
        end_index = lines[1:].index("---") + 1
    except ValueError:
        err(f"{path.relative_to(ROOT)}: unterminated frontmatter block")
        return {}
    try:
        fields = yaml.safe_load("\n".join(lines[1:end_index]))
    except yaml.YAMLError as exc:
        err(f"{path.relative_to(ROOT)}: invalid YAML frontmatter ({exc})")
        return {}
    if not isinstance(fields, dict):
        err(f"{path.relative_to(ROOT)}: frontmatter must be a YAML mapping")
        return {}
    return fields


def check_required(path, fields, required):
    for key in required:
        if not fields.get(key):
            err(f"{path.relative_to(ROOT)}: missing or empty '{key}' in frontmatter")


def validate_marketplace():
    marketplace = load_json(ROOT / ".claude-plugin" / "marketplace.json")
    if marketplace is None:
        return []
    for key in MARKETPLACE_FIELDS:
        if not marketplace.get(key):
            err(f"marketplace.json: missing or empty '{key}'")
    plugins = marketplace.get("plugins", [])
    if not isinstance(plugins, list) or not plugins:
        err("marketplace.json: 'plugins' must be a non-empty array")
        return []
    return plugins


def resolve_plugin_dir(entry):
    name = entry.get("name")
    source = entry.get("source")
    if not name or not source:
        err(f"marketplace.json: plugin entry missing 'name' or 'source' ({entry})")
        return None
    if not isinstance(source, str) or not source.startswith("./"):
        err(f"marketplace.json: plugin '{name}' source '{source}' must be a local './...' path")
        return None
    plugin_dir = (ROOT / source).resolve()
    if not plugin_dir.is_dir():
        err(f"marketplace.json: plugin '{name}' source '{source}' does not resolve to a directory")
        return None
    return plugin_dir


def validate_plugin_manifest(plugin_dir):
    manifest = load_json(plugin_dir / ".claude-plugin" / "plugin.json")
    if manifest is None:
        return
    for key in MANIFEST_FIELDS:
        if not manifest.get(key):
            err(f"{(plugin_dir / '.claude-plugin' / 'plugin.json').relative_to(ROOT)}: missing or empty '{key}'")
    version = manifest.get("version", "")
    if version and not SEMVER_RE.match(version):
        err(f"{(plugin_dir / '.claude-plugin' / 'plugin.json').relative_to(ROOT)}: version '{version}' is not x.y.z semver")


def validate_commands(plugin_dir):
    commands_dir = plugin_dir / "commands"
    if not commands_dir.is_dir():
        return
    for path in sorted(commands_dir.glob("*.md")):
        fields = parse_frontmatter(path)
        check_required(path, fields, ["description"])


def validate_skills(plugin_dir):
    skills_dir = plugin_dir / "skills"
    if not skills_dir.is_dir():
        return
    for skill_dir in sorted(p for p in skills_dir.iterdir() if p.is_dir()):
        skill_md = skill_dir / "SKILL.md"
        if not skill_md.is_file():
            err(f"{skill_dir.relative_to(ROOT)}: missing SKILL.md")
            continue
        fields = parse_frontmatter(skill_md)
        check_required(skill_md, fields, ["name", "description"])
        if fields.get("name") and fields["name"] != skill_dir.name:
            err(
                f"{skill_md.relative_to(ROOT)}: name '{fields['name']}' does not "
                f"match directory '{skill_dir.name}'"
            )


def validate_readme_links():
    for readme in sorted(ROOT.rglob("README.md")):
        if any(part in (".git", "_audit") for part in readme.relative_to(ROOT).parts):
            continue
        text = readme.read_text(encoding="utf-8")
        for target in LINK_RE.findall(text):
            target = target.strip()
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            target_path = target.split("#", 1)[0]
            if not target_path:
                continue
            resolved = (readme.parent / target_path).resolve()
            if not resolved.exists():
                err(f"{readme.relative_to(ROOT)}: broken link to '{target}'")


def main():
    plugins = validate_marketplace()
    for entry in plugins:
        plugin_dir = resolve_plugin_dir(entry)
        if plugin_dir is None:
            continue
        validate_plugin_manifest(plugin_dir)
        validate_commands(plugin_dir)
        validate_skills(plugin_dir)
    validate_readme_links()

    if errors:
        print(f"validate-marketplace: {len(errors)} error(s):\n")
        for e in errors:
            print(f"  - {e}")
        sys.exit(1)

    print("validate-marketplace: OK")


if __name__ == "__main__":
    main()
