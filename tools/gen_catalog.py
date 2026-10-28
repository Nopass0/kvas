#!/usr/bin/env python3
"""Writes the shader and script catalogue pages of the docs site (Russian and English) from the library folders.
    python tools/gen_catalog.py      (run from the repository root, after tools/gen_shaders.py and tools/gen_scripts.py)
"""
import os
import re

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
LIB = os.path.join(ROOT, "crates", "kvas_editor", "library")


def first_comment(path):
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if line.startswith("//"):
                return line.lstrip("/ ").strip()
            if line.startswith("///"):
                return line.lstrip("/ ").strip()
    return ""


def script_doc(path):
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    m = re.search(r"^\s*///\s*(.+)$", text, re.M)
    return m.group(1).strip() if m else ""


def shaders():
    groups = {}
    base = os.path.join(LIB, "shaders")
    for dirpath, _, files in os.walk(base):
        for name in sorted(files):
            if not name.endswith(".wgsl"):
                continue
            rel = os.path.relpath(os.path.join(dirpath, name), base).replace("\\", "/")
            parts = rel.split("/")
            if parts[0] == "fx":
                group = "Camera / " + (parts[1] if len(parts) > 2 else "general")
            else:
                group = "Object / " + (parts[0] if len(parts) > 1 else "general")
            groups.setdefault(group, []).append((rel, first_comment(os.path.join(dirpath, name))))
    return groups


def scripts():
    groups = {}
    base = os.path.join(LIB, "scripts")
    for dirpath, _, files in os.walk(base):
        for name in sorted(files):
            if not name.endswith(".rs"):
                continue
            rel = os.path.relpath(os.path.join(dirpath, name), base).replace("\\", "/")
            parts = rel.split("/")
            group = parts[-2] if len(parts) >= 2 else "general"
            groups.setdefault(group, []).append((rel, script_doc(os.path.join(dirpath, name))))
    return groups


def page(russian):
    sh, sc = shaders(), scripts()
    n_sh, n_sc = sum(len(v) for v in sh.values()), sum(len(v) for v in sc.values())
    out = []
    if russian:
        out.append(f"# Каталог шейдеров и скриптов\n\nСтраница создаётся автоматически (`python tools/gen_catalog.py`). В каждый новый проект кладутся **{n_sh} шейдеров** (`assets/shaders/library/`) и **{n_sc} скриптов** (`scripts/src/<папка>/`), а также материалы ([каталог](./nodes/materials)).\n")
        out.append("::: info Как использовать\nШейдер объекта: перетащите `.wgsl` на объект или выберите его в поле `shader` компонента MeshRenderer. Шейдер камеры: перетащите в окно сцены или добавьте в «Эффекты камеры» (`params.x` - сила). Константы в начале файла - настройки. Скрипт: перетащите файл из панели Project на объект; поля появятся в Инспекторе.\n:::\n")
        out.append("::: warning Шейдер вместо материала\nШейдер объекта сейчас **заменяет** материал на объекте, а не накладывается поверх. Совместить шейдер и материал можно в графе нод (`.kshader` / `.kmat`) или собрав WGSL из графа через **Export as WGSL shader**. Слои шейдеров поверх материала - в планах.\n:::\n")
        out.append(f"## Шейдеры ({n_sh})\n")
    else:
        out.append(f"# Shader and script catalogue\n\nThis page is generated (`python tools/gen_catalog.py`). Every new project gets **{n_sh} shaders** (`assets/shaders/library/`) and **{n_sc} scripts** (`scripts/src/<folder>/`), plus the materials ([catalogue](./nodes/materials)).\n")
        out.append("::: info How to use\nObject shader: drag the `.wgsl` onto an object or pick it in the `shader` field of the MeshRenderer. Camera shader: drop it into the scene view or add it to the camera effects (`params.x` is the strength). The constants at the top of a file are its settings. Script: drag the file from the Project panel onto an object; its fields show in the Inspector.\n:::\n")
        out.append("::: warning Shader instead of material\nAn object shader currently **replaces** the material on an object; it is not layered on top. To combine them use a node graph (`.kshader` / `.kmat`) or export the graph with **Export as WGSL shader**. Shader layers over a material are planned.\n:::\n")
        out.append(f"## Shaders ({n_sh})\n")
    for group in sorted(sh):
        items = sh[group]
        names = ", ".join(f"`{os.path.splitext(os.path.basename(r))[0]}`" for r, _ in items)
        out.append(f"### {group} ({len(items)})\n\n{names}\n")
    out.append(f"## {'Скрипты' if russian else 'Scripts'} ({n_sc})\n")
    for group in sorted(sc):
        items = sc[group]
        out.append(f"### {group} ({len(items)})\n")
        out.append("| Script | Description |\n|---|---|" if not russian else "| Скрипт | Описание |\n|---|---|")
        for rel, doc in items:
            out.append(f"| `{os.path.splitext(os.path.basename(rel))[0]}` | {doc.replace('|', '/')} |")
        out.append("")
    return "\n".join(out) + "\n"


def write(rel, text):
    path = os.path.join(ROOT, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print("wrote", rel)


write("docs/library.md", page(True))
write("docs/en/library.md", page(False))
