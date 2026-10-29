# Contributing to Kvas

Thanks for helping build Kvas. Contributions from people and AI-assisted workflows are welcome when they are reviewable, tested, documented, and correctly attributed.

## Find or propose work

- Search existing issues and pull requests before opening a duplicate.
- Use a bug report for reproducible defects and a feature request for a bounded proposal. Include the user problem, expected outcome, constraints, and acceptance criteria.
- Discuss architecture-changing, compatibility-sensitive, licensing-sensitive, or GPU-backend work before investing in a large implementation.
- Do not open issues containing secrets, private data, or assets you do not have rights to share.

## Development setup

Follow [DEVELOPMENT.md](DEVELOPMENT.md) to install prerequisites, clone, build, run the editor, and build a game. Read [AGENTS.md](AGENTS.md) for repository-wide architecture, testing, documentation, and performance requirements.

Configure the local pre-commit hook once:

```bash
git config core.hooksPath .githooks
```

The hook runs `tools/pre-commit.ps1`. GitHub Actions runs the same full validation on pull requests. Do not bypass these gates.

## Code changes

- Follow Rust formatting and existing crate patterns; keep changes scoped.
- Add focused regression tests for behavior changes, including error and boundary cases.
- Public API changes require rustdoc with lifecycle, units, errors, compatibility notes, and useful examples.
- Avoid new dependencies unless necessary; explain maintenance, size, license, and platform impact.
- Keep localization, input/accessibility, saved preferences, and theme behavior in mind for editor work.
- Do not commit secrets, build output, `.kvas` state, machine-specific files, recordings, or assets without verified redistribution rights.

## Generated references and libraries

Do not edit generated output as the source of truth. Update the source and run the canonical generator:

- **Material nodes:** update the relevant node implementation and `HELP` descriptions, then run `cargo run -p kvas_engine --example gen_node_docs` and relevant engine/render tests.
- **Built-in materials:** update `material_presets.rs`, then run `cargo run -p kvas_engine --example gen_material_library` and the relevant tests.
- **Shader and script libraries:** use `python tools/gen_shaders.py` and/or `python tools/gen_scripts.py`, then `python tools/gen_catalog.py`. Validate shaders with the render tests and sample scripts with the `games/library_check` project/tests.
- **Script cookbook:** add or update executable examples in `crates/kvas_engine/tests/cookbook.rs`; keep the English and Russian documentation examples consistent.
- **Localization:** add English keys through the existing `t("...")` translation mechanism and update the Russian translation files. Never hard-code new UI strings into one locale.

Inspect generated diffs for unexpected churn before staging them.

## Tests, coverage, and optimization evidence

Run the complete gate before every commit:

```powershell
./tools/pre-commit.ps1
```

Focused tests are useful while iterating but do not replace the full workspace check. GPU-only paths need an adapter-backed test or a clearly labelled manual test; ordinary CI cannot certify hardware behavior.

For performance changes, include a repeatable workload and baseline before optimizing, correctness/regression tests, repeated candidate measurements, variance, and before/after results in the pull request. Record hardware, graphics backend, OS, compiler/profile, scene/assets, warm-up, and sample count. Do not add flaky time limits to unit tests or claim an improvement without evidence. If the repo has no relevant benchmark, add one or mark the performance result unverified. The benchmark harness and stable performance CI are tracked in [ROADMAP.md](ROADMAP.md).

Coverage reports are published by CI as evidence. Kvas does not yet set a numeric global coverage threshold because a meaningful baseline and hardware-specific exclusions have not been established. New and changed behavior must nevertheless have appropriate tests.

## Documentation requirements

Update docs alongside code, not as a follow-up task. Consider module rustdoc, English and Russian guides, editor workflows, scripting/material/audio/render references, generated catalogues, README setup instructions, roadmap status, and changelog. State what works, prerequisites, exact steps, errors/limitations, and platform support. Keep the two locales in sync where a matching page exists. Update links/navigation when adding pages and run the docs build.

## Pull request checklist

Use a clear, imperative title. The description should include:

- Problem and user impact.
- Design and implementation summary, with compatibility/migration notes.
- Tests run, coverage artifact or gaps, and GPU validation status.
- Before/after benchmark evidence for performance work.
- Documentation/locales updated.
- Known limitations and follow-up roadmap issues.
- Screenshots or short captures for visual/editor changes, using only shareable project assets.

Before submitting, run `git diff --check`, inspect the full staged diff, and confirm no unintended or unlicensed files are included. Keep commits focused and do not rewrite shared history.

## Commit message style

Use a short imperative subject, optionally with a scope, for example:

```text
Fix scene asset path validation
render: cache shader pipeline layouts
Docs: add Linux build steps
```

## Community and security

Be respectful and follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). For vulnerabilities, use the private reporting method in [SECURITY.md](SECURITY.md), not a public issue.
