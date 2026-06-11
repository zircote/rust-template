# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| latest  | Yes                |
| < latest | No                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via [GitHub Security Advisories](https://github.com/zircote/rust-template/security/advisories/new).

### What to Include

- A description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours of the report
- **Initial assessment**: Within 1 week
- **Fix and disclosure**: Coordinated with the reporter, typically within 90 days

### Disclosure Policy

We follow responsible disclosure practices:

1. The reporter privately notifies us of the vulnerability.
2. We work together to understand and fix the issue.
3. We release a patched version.
4. The vulnerability is publicly disclosed after users have had time to update.

### Scope

This policy applies to the rust_template crate and its published artifacts. Third-party dependencies
are managed via `cargo-deny` and audited regularly through our CI pipeline.

## Security Measures

This project employs several security practices:

- **cargo-deny**: Audits dependencies for known vulnerabilities, license compliance, and banned crates
- **cargo-audit**: Checks for known security advisories in dependencies
- **Dependabot**: Automated dependency updates for security patches
- **No unsafe code**: The crate forbids `unsafe` unless explicitly justified
- **Minimal dependencies**: Only essential dependencies are included
- **SHA-pinned actions**: Every GitHub Actions `uses:` reference is pinned to a full commit SHA, enforced by a `pin-check` CI gate
- **Attested releases**: Container images are signed and attested (SLSA provenance, signature, SBOM, vulnerability report) by a centralized signer workflow and verified fail-closed before anything publishes

## Verifying Release Artifacts

Container images are signed and attested by the centralized signer workflow
`zircote/.github/.github/workflows/sign-and-attest.yml` (SLSA Build L3:
the signing identity is the central workflow, not this repository).
Prerequisites: `gh` CLI authenticated, `cosign` installed.

### Resolve the digest for a tag

```bash
DIGEST=$(gh api 'users/zircote/packages/container/rust-template/versions?per_page=20' \
  --jq '[.[] | select((.metadata.container.tags // []) | index("<tag>"))][0].name')
```

### SLSA provenance

`--repo` asserts where the build ran; `--signer-workflow` asserts the
signing identity. Both are required — `--repo` alone fails by design.

```bash
gh attestation verify "oci://ghcr.io/zircote/rust-template@${DIGEST}" \
  --repo zircote/rust-template \
  --signer-workflow zircote/.github/.github/workflows/sign-and-attest.yml \
  --predicate-type https://slsa.dev/provenance/v1
```

### Keyless signature

```bash
cosign verify "ghcr.io/zircote/rust-template@${DIGEST}" \
  --certificate-identity-regexp '^https://github.com/zircote/\.github/\.github/workflows/sign-and-attest\.yml@.*$' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

### SBOM and vulnerability report attestations

```bash
cosign verify-attestation "ghcr.io/zircote/rust-template@${DIGEST}" \
  --type cyclonedx \
  --certificate-identity-regexp '^https://github.com/zircote/\.github/\.github/workflows/sign-and-attest\.yml@.*$' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
# Vulnerability report: same command with
#   --type "https://in-toto.io/attestation/vulns/v0.1"
```

### Release binaries

Binaries attached to a GitHub Release carry build provenance attested by
this repository's own release workflow (no `--signer-workflow` needed):

```bash
gh release download <tag> --repo zircote/rust-template
gh attestation verify rust_template-linux-amd64 --repo zircote/rust-template
sha256sum --check SHA256SUMS
```
