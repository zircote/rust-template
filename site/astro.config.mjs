import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import astroMermaid from "astro-mermaid";

export default defineConfig({
    // `site` is the deployment ORIGIN; `base` is the path the project is
    // served under (GitHub Pages serves project sites at <domain>/<repo>).
    // Without `base`, assets emit at /_astro/... and 404 under /rust-template/,
    // rendering the site unstyled. With it, they resolve at /rust-template/_astro/.
    site: "https://zircote.com",
    base: "/rust-template",
    integrations: [
        astroMermaid(),
        starlight({
            title: "rust-template",
            head: [
                {
                    tag: "meta",
                    attrs: {
                        property: "og:image",
                        content: "https://zircote.com/rust-template/og-image.svg",
                    },
                },
                {
                    tag: "meta",
                    attrs: { property: "og:image:width", content: "1280" },
                },
                {
                    tag: "meta",
                    attrs: { property: "og:image:height", content: "640" },
                },
                {
                    tag: "meta",
                    attrs: { name: "twitter:card", content: "summary_large_image" },
                },
                {
                    tag: "meta",
                    attrs: {
                        name: "twitter:image",
                        content: "https://zircote.com/rust-template/og-image.svg",
                    },
                },
            ],
            logo: {
                light: "./src/assets/logo-light.svg",
                dark: "./src/assets/logo-dark.svg",
                replacesTitle: true,
            },
            social: [
                {
                    icon: "github",
                    label: "GitHub",
                    href: "https://github.com/zircote/rust-template",
                },
            ],
            sidebar: [
                {
                    label: "Overview",
                    items: [{ label: "Introduction", slug: "index" }],
                },
                {
                    label: "Tutorials",
                    items: [
                        {
                            label: "Your First Project",
                            slug: "tutorials/first-project",
                        },
                    ],
                },
                {
                    label: "Getting Started",
                    items: [
                        {
                            label: "Getting Started",
                            slug: "getting-started/getting-started",
                        },
                        {
                            label: "Configuration",
                            slug: "getting-started/configuration",
                        },
                        {
                            label: "Customization",
                            slug: "getting-started/customization",
                        },
                        {
                            label: "GitHub Template Features",
                            slug: "getting-started/github-template-features",
                        },
                        {
                            label: "Copilot Jumpstart",
                            slug: "getting-started/copilot-jumpstart",
                        },
                    ],
                },
                {
                    label: "Reference",
                    items: [
                        {
                            label: "API Documentation",
                            slug: "reference/api",
                        },
                        {
                            label: "Lint Configuration",
                            slug: "reference/lint-configuration",
                        },
                        {
                            label: "Cargo Profiles",
                            slug: "reference/cargo-profiles",
                        },
                        {
                            label: "Error Handling",
                            slug: "reference/error-handling",
                        },
                        {
                            label: "Builder Pattern",
                            slug: "reference/builder-pattern",
                        },
                        {
                            label: "Import Ordering",
                            slug: "reference/import-ordering",
                        },
                        {
                            label: "Doc Comments",
                            slug: "reference/doc-comments",
                        },
                        {
                            label: "Formatting",
                            slug: "reference/formatting",
                        },
                        {
                            label: "Ownership & Borrowing",
                            slug: "reference/ownership-and-borrowing",
                        },
                        {
                            label: "Type Design",
                            slug: "reference/type-design",
                        },
                        {
                            label: "Testing",
                            slug: "reference/testing",
                        },
                    ],
                },
                {
                    label: "Explanation",
                    items: [
                        {
                            label: "Architecture & Design",
                            slug: "explanation/architecture",
                        },
                    ],
                },
                {
                    label: "CI/CD & Workflows",
                    items: [
                        {
                            label: "Pipeline Overview",
                            slug: "workflows/ci-workflows",
                        },
                        {
                            label: "Code Quality",
                            slug: "workflows/code-quality",
                        },
                        {
                            label: "Spell Check",
                            slug: "workflows/spell-check",
                        },
                        {
                            label: "Workflow Reference",
                            collapsed: true,
                            items: [
                                { label: "pipeline", slug: "workflows/ref-pipeline" },
                                { label: "ci-checks", slug: "workflows/ref-ci-checks" },
                                {
                                    label: "ci-coverage",
                                    slug: "workflows/ref-ci-coverage",
                                },
                                {
                                    label: "ci-test-matrix",
                                    slug: "workflows/ref-ci-test-matrix",
                                },
                                {
                                    label: "code-quality",
                                    slug: "workflows/ref-code-quality",
                                },
                                {
                                    label: "docs-deploy",
                                    slug: "workflows/ref-docs-deploy",
                                },
                                {
                                    label: "release-docker",
                                    slug: "workflows/ref-release-docker",
                                },
                                {
                                    label: "docker-hub",
                                    slug: "workflows/ref-docker-hub",
                                },
                                {
                                    label: "benchmark",
                                    slug: "workflows/ref-benchmark",
                                },
                                {
                                    label: "benchmark-regression",
                                    slug: "workflows/ref-benchmark-regression",
                                },
                                {
                                    label: "fuzz-testing",
                                    slug: "workflows/ref-fuzz-testing",
                                },
                                {
                                    label: "mutation-testing",
                                    slug: "workflows/ref-mutation-testing",
                                },
                                {
                                    label: "secrets-scan",
                                    slug: "workflows/ref-secrets-scan",
                                },
                                {
                                    label: "security-audit",
                                    slug: "workflows/ref-security-audit",
                                },
                                {
                                    label: "spell-check",
                                    slug: "workflows/ref-spell-check",
                                },
                                {
                                    label: "stale",
                                    slug: "workflows/ref-stale",
                                },
                                {
                                    label: "contributors",
                                    slug: "workflows/ref-contributors",
                                },
                                {
                                    label: "dependabot-automerge",
                                    slug: "workflows/ref-dependabot-automerge",
                                },
                                {
                                    label: "nightly",
                                    slug: "workflows/ref-nightly",
                                },
                                {
                                    label: "template-init",
                                    slug: "workflows/ref-template-init",
                                },
                                {
                                    label: "copilot-setup-steps",
                                    slug: "workflows/ref-copilot-setup-steps",
                                },
                                {
                                    label: "adr-validation",
                                    slug: "workflows/ref-adr-validation",
                                },
                                {
                                    label: "adr-viewer",
                                    slug: "workflows/ref-adr-viewer",
                                },
                                {
                                    label: "daily-docs-review",
                                    slug: "workflows/ref-daily-docs-review-lock",
                                },
                            ],
                        },
                    ],
                },
                {
                    label: "Testing",
                    items: [
                        {
                            label: "Property-Based Testing",
                            slug: "testing/property-based-testing",
                        },
                        {
                            label: "Mutation Testing",
                            slug: "testing/mutation-testing",
                        },
                        {
                            label: "Fuzz Testing",
                            slug: "testing/fuzz-testing",
                        },
                        {
                            label: "Coverage",
                            slug: "testing/coverage",
                        },
                        {
                            label: "Test Matrix",
                            slug: "testing/test-matrix",
                        },
                    ],
                },
                {
                    label: "Security",
                    items: [
                        {
                            label: "Attested Delivery",
                            slug: "security/attested-delivery",
                        },
                        {
                            label: "Signed Releases",
                            slug: "security/signed-releases",
                        },
                        {
                            label: "Secrets Scan",
                            slug: "security/secrets-scan",
                        },
                        {
                            label: "Container Scan",
                            slug: "security/container-scan",
                        },
                        {
                            label: "SBOM",
                            slug: "security/sbom",
                        },
                    ],
                },
                {
                    label: "Distribution",
                    items: [
                        {
                            label: "Package Managers",
                            slug: "distribution/package-managers",
                        },
                        {
                            label: "Docker Registries",
                            slug: "distribution/docker-registries",
                        },
                        {
                            label: "Alternative Registries",
                            slug: "distribution/alternative-registries",
                        },
                    ],
                },
                {
                    label: "Observability",
                    items: [
                        {
                            label: "Metrics Dashboard",
                            slug: "observability/metrics-dashboard",
                        },
                        {
                            label: "Benchmark Regression",
                            slug: "observability/benchmark-regression",
                        },
                    ],
                },
                {
                    label: "UX",
                    items: [
                        {
                            label: "Shell Completions",
                            slug: "ux/shell-completions",
                        },
                        {
                            label: "Man Pages",
                            slug: "ux/man-pages",
                        },
                    ],
                },
                {
                    label: "Runbooks",
                    items: [
                        {
                            label: "Releasing",
                            slug: "runbooks/releasing",
                        },
                        {
                            label: "Dependency Updates",
                            slug: "runbooks/dependency-updates",
                        },
                        {
                            label: "Security Response",
                            slug: "runbooks/security-response",
                        },
                        {
                            label: "CI Troubleshooting",
                            slug: "runbooks/ci-troubleshooting",
                        },
                    ],
                },
                {
                    label: "Design Decisions",
                    items: [
                        {
                            label: "ADR-0001: Use ADRs",
                            slug: "design-decisions/adr-0001",
                        },
                        {
                            label: "ADR-0002: Doc Structure",
                            slug: "design-decisions/adr-0002",
                        },
                    ],
                },
            ],
        }),
    ],
});
