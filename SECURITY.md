# Security Policy

## Supported versions
Aval is pre-alpha. Only the latest commit on `main` and the latest tagged release (once there is one) receive security fixes. This table will be updated at each release.

| Version | Supported |
| --- | --- |
| `main` | yes |
| tagged releases | latest only |

## Reporting a vulnerability
**Please do not open a public issue, discussion, or pull request for a vulnerability.**

Use GitHub private vulnerability reporting:
1. Go to the repository on GitHub: https://github.com/Nic69Han/Aval
2. Open the **Security** tab (labelled "Security and quality" in some layouts).
3. Click **Report a vulnerability** and fill in the form: summary, details, steps to reproduce or proof of concept, and impact.

See [GitHub's documentation](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability). The report is visible only to you and the maintainers, and you are credited on the resulting advisory.

If the Report button is not available (a maintainer must enable the feature in repository settings), open a public issue titled "Security contact request" **without any details**, and a maintainer listed in [GOVERNANCE.md](GOVERNANCE.md) will arrange a private channel.

## What is in scope
- The compiler, checker, runtime support code, and generated Rust for memory-safety or sandbox issues (notably the `unsafe` FFI surface and `resource` handling).
- Miscompilation that violates a locked spec guarantee in a way that creates a security impact (for example, an `effects none` function performing I/O).
- The playground and any hosted service (injection, sandbox escape, resource exhaustion).
- Supply-chain issues in the build, CI, or release process.

Out of scope: vulnerabilities in programs written in Aval that Aval never claimed to prevent; denial of service from intentionally pathological input to a local CLI; issues in third-party dependencies with no Aval-specific impact (report upstream).

## What to expect
Aval is maintained by a small team; these are targets, not guarantees:
- Acknowledgement within 5 days.
- Initial assessment within 14 days.
- Coordinated disclosure: we aim to fix and publish an advisory within 90 days of the report and will agree a date with you; earlier if actively exploited.
- Credit in the advisory unless you prefer otherwise.

Only maintainers read security reports.

## Safe harbour
We will not pursue action against researchers who act in good faith, avoid privacy violations and service disruption, and give us reasonable time to fix before public disclosure.
