#!/usr/bin/env node

/**
 * Extracts reference sections from the project root CLAUDE.md
 * and generates individual Starlight MDX pages.
 */

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..", "..");
const siteRoot = resolve(__dirname, "..");

/**
 * Sections to extract from CLAUDE.md.
 * heading: the ### heading text to match
 * output: output filename under reference/
 * title: page title
 * description: page description
 */
const SECTIONS = [
    {
        heading: "Lint Configuration",
        output: "lint-configuration.mdx",
        title: "Lint Configuration",
        description: "Clippy lint groups, denied lints, allowed lints, and thresholds.",
    },
    {
        heading: "Cargo Profiles",
        output: "cargo-profiles.mdx",
        title: "Cargo Profiles",
        description: "Build profiles for dev, release, and release-debug configurations.",
    },
    {
        heading: "Error Handling",
        output: "error-handling.mdx",
        title: "Error Handling",
        description: "Error type design, Result alias, and propagation conventions.",
    },
    {
        heading: "Builder Pattern",
        output: "builder-pattern.mdx",
        title: "Builder Pattern",
        description: "Consuming-self builder pattern with const fn.",
    },
    {
        heading: "Import Ordering",
        output: "import-ordering.mdx",
        title: "Import Ordering",
        description: "Import grouping and ordering conventions.",
    },
    {
        heading: "Doc Comments",
        output: "doc-comments.mdx",
        title: "Doc Comments",
        description: "Documentation comment structure and requirements.",
    },
    {
        heading: "Formatting",
        output: "formatting.mdx",
        title: "Formatting",
        description: "rustfmt configuration and formatting rules.",
    },
    {
        heading: "Testing",
        output: "testing.mdx",
        title: "Testing Reference",
        description: "Test types, locations, coverage requirements, and property testing.",
    },
    {
        heading: "Ownership and Borrowing",
        output: "ownership-and-borrowing.mdx",
        title: "Ownership and Borrowing",
        description: "Guidelines for ownership, borrowing, and parameter types.",
    },
    {
        heading: "Type Design",
        output: "type-design.mdx",
        title: "Type Design",
        description: "Newtype patterns, derive macros, and enum vs trait guidance.",
    },
];

function extractSection(content, heading) {
    // Find the ### heading
    const headingPattern = new RegExp(
        `^###\\s+${heading.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\s*$`,
        "m",
    );
    const match = content.match(headingPattern);
    if (!match) return null;

    const start = match.index + match[0].length;

    // Find the next heading at the same or higher level (##, ###)
    const rest = content.slice(start);
    const nextHeading = rest.match(/^#{2,3}\s+/m);
    const body = nextHeading ? rest.slice(0, nextHeading.index) : rest;

    // Strip trailing HR separators (with possible blank lines) and HTML comments
    let cleaned = body.trim();
    cleaned = cleaned.replace(/(\n\s*)*---\s*$/, "").trim();
    cleaned = cleaned.replace(/<!--[\s\S]*?-->/g, "").trim();
    return cleaned;
}

/**
 * Generate reference pages from CLAUDE.md sections.
 * @param {string} [outputBase] - Override output base directory
 * @returns {{ generated: string[], skipped: string[] }}
 */
export function generateReferencePages(outputBase) {
    const claudeMdPath = join(projectRoot, "CLAUDE.md");
    const content = readFileSync(claudeMdPath, "utf-8");
    const outDir = outputBase || join(siteRoot, "src", "content", "docs");
    const generated = [];
    const skipped = [];

    for (const section of SECTIONS) {
        const body = extractSection(content, section.heading);
        if (!body) {
            console.warn(`  SKIP: Section "${section.heading}" not found`);
            skipped.push(section.heading);
            continue;
        }

        const frontmatter = [
            "---",
            `title: "${section.title}"`,
            `description: "${section.description}"`,
            "---",
        ].join("\n");

        const pageContent = `${frontmatter}\n\n${body}\n`;
        const outPath = join(outDir, "reference", section.output);
        mkdirSync(dirname(outPath), { recursive: true });
        writeFileSync(outPath, pageContent, "utf-8");
        console.log(`  OK: reference/${section.output}`);
        generated.push(`reference/${section.output}`);
    }

    return { generated, skipped };
}

// Run directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
    console.log("Generating reference pages from CLAUDE.md...");
    const { generated, skipped } = generateReferencePages();
    console.log(`\nDone: ${generated.length} generated, ${skipped.length} skipped.`);
}
