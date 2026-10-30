# GitHub dev → staging promotion

GitHub promotion is an intentional exception to the older archive-only guidance
in AGENTS.md and TOOLS.md. Forgejo/Gitea workflows remain unchanged.

Roll out the same `.github/workflows/greg-promote.yml` and
`.github/scripts/greg-gates.sh` to **both main and dev through PRs**. Updating
main alone cannot restore the push trigger or script on dev. Keep these files
together when promoting or synchronizing branches; do not substitute the
provider-specific `scripts/greg-gates.sh` (its coverage policy differs).

Push events evaluate their immutable event SHA. Manual dispatch evaluates dev;
`dry_run` defaults to true. Required runner tools are Bash, Git, Python 3.11+, the
Rust toolchain, cargo-tarpaulin, cargo-audit, and gitleaks. Missing gate tools
block promotion. Rust requires Cargo.toml and Cargo.lock; the staging ref and
common history must exist. Existing optional .NET/Node checks are retained.

The evaluator validates child exit status, verdict, checked-out SHA and report.
The report is uploaded even after evaluation failure, under a run/attempt-specific
artifact name. The issue job downloads that artifact into an isolated directory;
missing artifacts produce a diagnostic with job results and a run link. Existing
blocked issues get their body refreshed. The tracking issue closes only after
both gates and the actual staging push succeed. Actor restriction, dry-run,
coverage >=90% with <=5 percentage point drop, security checks and non-force
fast-forward pushes remain in place.

Validation (Python 3.11+ and PyYAML): `python3 .github/tests/test_promotion.py`. These tests use temporary
repositories and stub evaluators, never build the app or push staging. After PR
merge, dispatch with `dry_run: true` on the intended branch and inspect the gate
artifact/summary before relying on the next authorized dev push. Do not rerun the
old failed run: it retains the old workflow revision. Issue #41 stays open until
a real promotion succeeds. Documentation is tracked in-repo; no wiki edit needed.
