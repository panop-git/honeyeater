# Security policy

## Reporting a vulnerability

If you believe you have found a security vulnerability in honeyeater — including in the library's published crates, in its test infrastructure (`honeyeater-test`), or in the oracle-generation tooling (`tools/oracle-gen/`) — please report it privately rather than opening a public issue.

Use **GitHub's private vulnerability reporting**:

1. Navigate to the repository's **Security** tab on GitHub.
2. Click **Report a vulnerability**.
3. Provide a description, reproduction steps, and any relevant context.

GitHub will route the report to the project maintainers privately. The report stays confidential until a fix is ready and a coordinated disclosure date is agreed.

If GitHub private vulnerability reporting is unavailable for any reason (e.g. the feature is disabled on a fork you found the issue in), the fallback is to open an issue requesting a private contact channel — without describing the vulnerability publicly.

## What counts as a security issue

honeyeater is a numerical library, not a network or cryptographic library, so the realistic security-relevant failure modes are narrower than for typical software. Issues we treat as security-sensitive:

- Memory-safety bugs in honeyeater's own code. The codebase forbids `unsafe`, so any path that produces undefined behaviour in safe Rust is a real issue and a high priority — it likely indicates a soundness bug in either honeyeater or one of its dependencies.
- Panics on safely-constructed inputs that a downstream operator cannot guard against (e.g. a numerical input that puts a kernel into an infinite loop or stack overflow).
- Vulnerabilities in honeyeater's transitive dependencies that affect a kernel's public surface.
- Issues in the reference-vector or oracle pipeline that could allow a hostile vector to escalate into code execution at test time.

Issues we do **not** treat as security-sensitive (but still welcome as ordinary bug reports):

- Numerical inaccuracy that falls outside a kernel's documented tolerance. This is a correctness bug, not a vulnerability.
- Performance regressions.
- API-design disagreements.

## Disclosure timeline

honeyeater is pre-v0.0.1 and has no formal SLAs. The intent is to triage reports promptly and coordinate disclosure when a fix is ready. Once the project reaches a stable release, this section will be expanded with concrete timeline commitments.

## Scope

This policy covers the published `honeyeater` crate and its workspace members. Reports about transitive dependencies should ideally be filed with the upstream maintainer; we are happy to help coordinate where the issue affects honeyeater's API.

The Panop downstream runtime (`docs/downstream-context.md`) is a separate project with its own security policy; vulnerability reports about that runtime should be directed there.
