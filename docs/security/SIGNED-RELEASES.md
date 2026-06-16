---
diataxis_type: explanation
---
# Signed Releases & SLSA Provenance

## Overview

Every release artifact carries cryptographic attestations, and nothing publishes unverified: a fail-closed verification job runs **before** the GitHub Release exists.

**Workflows:**
- `.github/workflows/release.yml` - SLSA build provenance and CycloneDX SBOM attestations on every release binary, verified fail-closed before the release is published
- `.github/workflows/publish.yml` - provenance attestation on the `.crate` archive that crates.io actually serves
- `.github/workflows/pipeline.yml` - container image signing and attestation via the centralized `zircote/.github` signer workflow (SLSA Build L3), then fail-closed verification

The canonical verification commands live in [SECURITY.md](../../SECURITY.md#verifying-release-artifacts). This document explains the architecture and expands on each artifact type.

## Why Attest Releases?

- **Authenticity**: Verify artifacts were built by this repository's workflows
- **Integrity**: Detect tampering or corruption
- **Non-repudiation**: The attestation binds the artifact to the exact commit, workflow, and run
- **Compliance**: Meet supply chain security requirements

## GitHub Artifact Attestations (Release Binaries)

### How It Works

1. **Tag pushed** - `release.yml` triggers; binary name and version are resolved from `cargo metadata`
2. **Build binaries** - 5 platform targets, named `{bin}-{version}-{platform}` (e.g. `rust_template-0.1.0-linux-amd64`)
3. **Attest provenance** - `actions/attest-build-provenance` attaches SLSA build provenance to each binary at build time
4. **Generate + attest SBOM** - a CycloneDX SBOM is generated (`anchore/sbom-action`) and bound to every binary via `actions/attest-sbom`
5. **Verify fail-closed** - a dedicated job runs `gh attestation verify` (provenance and SBOM) against every artifact; any failure blocks the release
6. **Publish release** - binaries, the SBOM, and a single `{bin}-{version}-checksums.txt` file are attached to the GitHub Release

A tag publishes nothing unattested. Test and `cargo-audit` gates also run in the same workflow, because tags are not guaranteed to point at CI-green commits.

### Verifying Release Binaries

Prerequisite: an authenticated `gh` CLI.

```bash
# Download the release assets
gh release download v0.1.0 --repo USER/REPO

# Verify SLSA build provenance
gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO

# Verify the SBOM attestation
gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO \
  --predicate-type https://cyclonedx.org/bom
```

### Verifying Checksums

```bash
shasum -a 256 -c rust_template-0.1.0-checksums.txt
```

### Verifying the Published Crate

`publish.yml` downloads the `.crate` that crates.io serves, byte-compares it against the locally packaged archive (a mismatch fails the workflow), and attests it — the attestation covers the registry bytes, not a local rebuild:

```bash
curl -fsSL -A 'release-check' \
  -O https://static.crates.io/crates/rust_template/rust_template-0.1.0.crate
gh attestation verify rust_template-0.1.0.crate --repo USER/REPO
```

### Keyless Signing

Attestations are signed keyless via Sigstore:
- No private keys to manage
- Uses OIDC identity (GitHub Actions)
- Transparency log (Rekor) for auditability
- Certificate from Fulcio CA

**Benefits:**
- No key rotation needed
- No key compromise risk
- Publicly verifiable
- Auditable via transparency log

## Container Image Attestations

Container images are **not** signed by this repository. They are signed and attested by the centralized signer workflow `zircote/.github/.github/workflows/sign-and-attest.yml`, then verified fail-closed by `docker-verify` in `pipeline.yml`. Under SLSA Build L3 the signing identity is the central workflow, not this repo — so verification must assert both where the build ran (`--repo`) and who signed (`--signer-workflow`):

```bash
# Resolve the digest for a tag
DIGEST=$(gh api 'users/USER/packages/container/REPO/versions?per_page=20' \
  --jq '[.[] | select((.metadata.container.tags // []) | index("<tag>"))][0].name')

# SLSA provenance — --repo alone fails by design
gh attestation verify "oci://ghcr.io/USER/REPO@${DIGEST}" \
  --repo USER/REPO \
  --signer-workflow zircote/.github/.github/workflows/sign-and-attest.yml \
  --predicate-type https://slsa.dev/provenance/v1

# Keyless signature
cosign verify "ghcr.io/USER/REPO@${DIGEST}" \
  --certificate-identity-regexp '^https://github.com/zircote/\.github/\.github/workflows/sign-and-attest\.yml@.*$' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

The central signer also attaches SBOM and vulnerability-report attestations; see [SECURITY.md](../../SECURITY.md#verifying-release-artifacts) for those commands.

## SLSA Provenance

### What is SLSA?

**SLSA** (Supply chain Levels for Software Artifacts) is a framework for ensuring software supply chain integrity.

**Levels:**
- **SLSA 1**: Documentation of build process
- **SLSA 2**: Version control + build service
- **SLSA 3**: Hardened builds (non-falsifiable provenance)
- **SLSA 4**: Hermetic, reproducible builds

### Who Signs What

| Artifact | Attested by | Verify with |
|---|---|---|
| Release binaries + SBOM | This repo's `release.yml` | `gh attestation verify <file> --repo USER/REPO` |
| Published `.crate` | This repo's `publish.yml` | `gh attestation verify <crate> --repo USER/REPO` |
| Container images | Central `zircote/.github` signer (SLSA Build L3) | `gh attestation verify oci://... --repo USER/REPO --signer-workflow ...` |

No `--signer-workflow` flag is needed for binaries and crates — they are attested by this repository's own workflows.

### Inspecting Provenance

```bash
# Print the full verification result, including the provenance statement
gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO \
  --format json | jq '.[0].verificationResult.statement'

# Extract specific fields
gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO \
  --format json | jq '.[0].verificationResult.statement.predicate.buildDefinition'
```

## Integration Examples

### Docker

Verify a binary before adding it to an image:

```dockerfile
# Verify provenance before adding to image (gh CLI in the build stage)
RUN gh release download v0.1.0 --repo USER/REPO \
      --pattern 'rust_template-0.1.0-linux-amd64' && \
    gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO
```

### CI Consumers

Any pipeline consuming release binaries should verify before use:

```bash
gh attestation verify "$ARTIFACT" --repo USER/REPO || exit 1
```

## Advanced Configuration

### Custom Signing Keys

For organizations with existing PKI, GPG signatures can be layered on top of attestations:

```yaml
- name: Import GPG key
  run: echo "${{ secrets.GPG_PRIVATE_KEY }}" | gpg --import

- name: Sign with GPG
  run: |
    for file in *; do
      gpg --detach-sign --armor "$file"
    done
```

**Verify GPG:**
```bash
gpg --verify rust-template.asc rust-template
```

## Security Best Practices

### 1. Minimize Attack Surface

- **Use official actions** with commit SHA pinning (enforced by the `pin-check` CI gate)
- **Limit permissions** to minimum required (`id-token: write` and `attestations: write` only on jobs that attest)
- **Avoid secrets** in logs or artifacts

### 2. Verify Everything

- **Verify dependencies** before building (`cargo-deny`, `cargo-audit` gates)
- **Verify artifacts** before they publish — the release workflow's verify job is fail-closed
- **Verify on the consuming side** — in-pipeline success is necessary, not sufficient

### 3. Audit Trail

- **Rekor** transparency log records every attestation signature
- **Archive provenance** long-term
- **Monitor certificates** for unexpected issuance

### 4. User Education

- **Document verification** in SECURITY.md (canonical commands)
- **Provide examples** of verification
- **Link to tools** (`gh attestation`, cosign for images)

## Troubleshooting

### `gh attestation verify` Fails

```bash
# Inspect what attestations exist for the artifact
gh attestation verify rust_template-0.1.0-linux-amd64 --repo USER/REPO --format json
```

**Common issues:**
- Wrong `--repo` (must be the repository whose workflow attested the artifact)
- Missing `--signer-workflow` for container images (they are signed by the central workflow; `--repo` alone fails by design)
- Wrong `--predicate-type` (SBOM attestations need `https://cyclonedx.org/bom`; image provenance needs `https://slsa.dev/provenance/v1`)
- Artifact was modified after download (checksum it against `{bin}-{version}-checksums.txt`)
- Unauthenticated `gh` CLI

### Release Was Not Published

If the `verify` job fails, the release is intentionally never created — that is the fail-closed design working. Check the `Verify Attestations` job logs, fix the cause, and re-release with a new tag. Never re-run `release.yml` against an existing tag: builds are not reproducible and re-publishing would overwrite released assets with different bytes.

## Monitoring & Compliance

### Rekor Transparency Log

All Sigstore signatures (attestations and image signatures) are logged to Rekor:

**URL:** https://search.sigstore.dev/

### Compliance Reports

Generate reports for audits:

```bash
# Verify every asset of every release
gh release list --repo USER/REPO --json tagName -q '.[].tagName' | while read -r tag; do
  echo "Release: $tag"
  gh release download "$tag" --repo USER/REPO --dir "audit/$tag"
  for f in audit/$tag/*; do
    case "$f" in *checksums.txt) continue ;; esac
    gh attestation verify "$f" --repo USER/REPO
  done
done
```

## Links

- [GitHub Artifact Attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)
- [SLSA Framework](https://slsa.dev/)
- [Sigstore](https://www.sigstore.dev/)
- [Rekor Transparency Log](https://github.com/sigstore/rekor)
- [CycloneDX](https://cyclonedx.org/)
- [Supply Chain Security Guide](https://slsa.dev/spec/v1.0/)
