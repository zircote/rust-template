---
diataxis_type: how-to
---
# Software Bill of Materials (SBOM)

Automated generation of a Software Bill of Materials in CycloneDX format for supply chain transparency and compliance.

## Reference

| Field | Value |
|---|---|
| Workflow | `.github/workflows/release.yml` (job `sbom`) |
| Tool | `anchore/sbom-action` |
| Format | CycloneDX JSON |
| Triggers | Version tags, `workflow_dispatch` dry-run |

### What an SBOM is

A machine-readable inventory of:

- All dependencies (direct and transitive)
- License information
- Package versions
- Supplier information

Common uses: supply chain security (EO 14028 compliance), vulnerability tracking, license compliance, and dependency auditing.

### CI pipeline stages

During a release the `sbom` job in `release.yml`:

1. Downloads the built platform binaries.
2. Generates a CycloneDX JSON SBOM with `anchore/sbom-action` (output `${bin}-${version}-sbom.cdx.json`).
3. Attests the SBOM with `actions/attest-sbom`, binding every binary to the SBOM.
4. Uploads it as a build artifact and attaches it to the GitHub release.

### SBOM contents

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "metadata": {
    "component": {
      "type": "application",
      "name": "rust_template",
      "version": "0.1.0"
    }
  },
  "components": [
    {
      "type": "library",
      "name": "serde",
      "version": "1.0.0",
      "licenses": [{ "license": { "id": "MIT" } }],
      "purl": "pkg:cargo/serde@1.0.0"
    }
  ]
}
```

### Compliance coverage

- **Executive Order 14028** — machine-readable format (CycloneDX; SPDX also acceptable to regulators), dependency enumeration, license identification, supplier information.
- **NIST SP 800-161r1** — supply chain risk management.

## How-to

### Generate an SBOM locally

The CI job uses `anchore/sbom-action`, which wraps Syft. The local equivalent is `syft` directly (or `cargo cyclonedx` for a Cargo-native CycloneDX document):

```bash
# Install syft (the engine behind anchore/sbom-action)
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin

# Generate a CycloneDX JSON SBOM
syft dir:. -o cyclonedx-json > sbom.cdx.json

# View SBOM
cat sbom.cdx.json | jq '.components[] | {name, version, licenses}'
```

Alternatively, a Cargo-native CycloneDX generator:

```bash
cargo install cargo-cyclonedx
cargo cyclonedx --format json
```

Verify: `sbom.cdx.json` parses as JSON and lists component entries.

### Download an SBOM from a release

```bash
# Download from a GitHub release
wget https://github.com/zircote/rust-template/releases/download/v0.1.0/rust_template-0.1.0-sbom.cdx.json

# Validate with a CycloneDX-aware tool
cyclonedx validate --input-file rust_template-0.1.0-sbom.cdx.json
```

Verify: the validator reports the document as valid.

### Customize generation

```bash
# SPDX JSON output (also accepted by regulators)
syft dir:. -o spdx-json

# Restrict to a single artifact
syft dir:. -o cyclonedx-json --select-catalogers cargo
```

Verify: the output header reflects the requested format.

### Troubleshooting

**Missing dependencies** — refresh the lockfile first:

```bash
cargo update
syft dir:. -o cyclonedx-json
```

**License issues** — unknown licenses appear blank or as `NOASSERTION`; declare your crate's license in `Cargo.toml`:

```toml
[package]
license = "MIT"
```

**Format errors** — validate the document:

```bash
cyclonedx validate --input-file sbom.cdx.json
```

Verify: validation completes without errors.

## Why this matters

An SBOM turns "trust us, the dependencies are fine" into a verifiable artifact. When a new CVE lands against a transitive dependency, the inventory answers "are we affected?" in seconds instead of a manual `cargo tree` audit, and the same document satisfies the machine-readable enumeration that EO 14028 and NIST SP 800-161r1 require. Generating it at release time and attesting it over the shipped binaries binds the bill of materials to exactly the versions that shipped, so the record reflects the released artifact rather than a drifting development tree.

## Links

- [anchore/sbom-action](https://github.com/anchore/sbom-action)
- [Syft](https://github.com/anchore/syft)
- [CycloneDX Specification](https://cyclonedx.org/specification/overview/)
- [SPDX Specification](https://spdx.github.io/spdx-spec/)
- [NTIA Minimum Elements](https://www.ntia.gov/files/ntia/publications/sbom_minimum_elements_report.pdf)
- [Executive Order 14028](https://www.whitehouse.gov/briefing-room/presidential-actions/2021/05/12/executive-order-on-improving-the-nations-cybersecurity/)
- [CI Workflows reference](../template/CI-WORKFLOWS.md)
