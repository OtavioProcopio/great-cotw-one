"""Gera THIRD-PARTY-LICENSES.md com as licenças das dependências que vão no app.

Rust: pacotes que entram na compilação do app (`cargo tree -e normal`), com licença declarada.
Interface: dependências de produção de `ui/package.json`.

Uso: python scripts/third_party.py   (rode na raiz do repositório, depois de `npm --prefix ui ci`)
"""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def rust_packages() -> list[tuple[str, str, str, str]]:
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--locked"],
            cwd=ROOT, capture_output=True, text=True, encoding="utf8", check=True,
        ).stdout
    )
    tree = subprocess.run(
        ["cargo", "tree", "-p", "great-cotw-one", "-e", "normal", "--prefix", "none", "--locked", "--target", "x86_64-pc-windows-msvc"],
        cwd=ROOT, capture_output=True, text=True, encoding="utf8", check=True,
    ).stdout
    used = set(re.findall(r"^([\w\-]+) v([\w.\-+]+)", tree, re.M))
    workspace = set(meta["workspace_members"])
    out = []
    for pkg in meta["packages"]:
        if pkg["id"] in workspace or (pkg["name"], pkg["version"]) not in used:
            continue
        out.append((pkg["name"], pkg["version"], pkg.get("license") or pkg.get("license_file") or "VER O PACOTE", pkg.get("repository") or ""))
    return sorted(set(out))


def npm_packages() -> list[tuple[str, str, str, str]]:
    pkg = json.loads((ROOT / "ui" / "package.json").read_text(encoding="utf8"))
    out = []
    for name in sorted(pkg.get("dependencies", {})):
        info = json.loads((ROOT / "ui" / "node_modules" / name / "package.json").read_text(encoding="utf8"))
        repo = info.get("repository")
        repo = repo.get("url", "") if isinstance(repo, dict) else (repo or "")
        out.append((name, info["version"], info.get("license", "VER O PACOTE"), repo))
    return out


def table(rows: list[tuple[str, str, str, str]]) -> str:
    lines = ["| Pacote | Versão | Licença | Origem |", "|---|---|---|---|"]
    lines += [f"| {n} | {v} | {lic} | {repo} |" for n, v, lic, repo in rows]
    return "\n".join(lines)


def main() -> None:
    text = (
        "# Licenças de terceiros\n\n"
        "O Great COTW One usa as bibliotecas abaixo, cada uma com a sua licença (MIT, Apache-2.0, BSD e similares).\n"
        "Os textos completos das licenças acompanham o código de cada biblioteca, nos endereços de origem.\n"
        "Este arquivo é gerado por `scripts/third_party.py`; não edite à mão.\n\n"
        "## Aplicativo (Rust)\n\n" + table(rust_packages()) + "\n\n"
        "## Interface (JavaScript)\n\n" + table(npm_packages()) + "\n"
    )
    (ROOT / "THIRD-PARTY-LICENSES.md").write_text(text, encoding="utf8", newline="\n")
    print("ok:", len(text.splitlines()), "linhas")


if __name__ == "__main__":
    main()
