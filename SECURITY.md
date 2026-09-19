
Security Policy
Status
Engineering Civilization is pre-1.0 research/engineering software.
The sandbox is hardened but is not a formal security boundary.

Reporting a vulnerability
Please report privately via GitHub Security Advisories, or by email to the
maintainer. Do not open public issues for security-sensitive reports.

In scope
Sandbox weaknesses (seccomp bypass, container escape via the EC path).
API authentication bypass.
Denial of service via crafted input (AST depth, request body size).
Out of scope
Host kernel, Docker daemon, or container-runtime vulnerabilities.
Network-level attacks (the API binds localhost by default).
Issues requiring a pre-compromised host.
Current hardening
--network none, --cap-drop ALL, --read-only, non-root user,
--pids-limit, no-new-privileges, seccomp allowlist, tmpfs workspace,
and forced container cleanup on timeout/IO.

Known gaps (see README "Known limitations")
Escape-vector test/production seccomp parity is incomplete.
API key comparison is not yet constant-time; no rate/body limits.
