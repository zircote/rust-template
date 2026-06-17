#!/usr/bin/env node

/**
 * Generates Starlight MDX pages from source markdown docs.
 * Reads docs-mapping.json for source-to-output mapping.
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..", "..");
const siteRoot = resolve(__dirname, "..");

function buildLinkMap(mappingPages, basePath = "") {
    const linkMap = {};
    for (const page of mappingPages) {
        const filename = page.source.split("/").pop();
        // Internal links must include the Pages base path (e.g. /rust-template),
        // otherwise they resolve to the domain root and 404. `basePath` mirrors
        // `base` in astro.config.mjs.
        const url = basePath + "/" + page.output.replace(/\.mdx$/, "/");
        linkMap[filename] = url;
    }
    linkMap["CONTRIBUTING.md"] =
        "https://github.com/zircote/rust-template/blob/main/CONTRIBUTING.md";
    linkMap["SECURITY.md"] = "https://github.com/zircote/rust-template/blob/main/SECURITY.md";
    linkMap["README.md"] = "https://github.com/zircote/rust-template/blob/main/README.md";
    return linkMap;
}

function rewriteLinks(content, linkMap) {
    return content.replace(
        /\]\(([^)]*?([A-Z][A-Z0-9_-]+\.md))(#[^)]*)?\)/gi,
        (match, fullPath, filename, anchor) => {
            if (linkMap[filename]) {
                return `](${linkMap[filename]}${anchor || ""})`;
            }
            return match;
        },
    );
}

function stripFrontmatter(content) {
    return content.replace(/^---\n[\s\S]*?\n---\n/, "");
}

function extractTitle(content) {
    const match = content.match(/^#\s+(.+)$/m);
    return match ? match[1].trim() : null;
}

function buildFrontmatter(page, extractedTitle) {
    const title = page.title || extractedTitle || "Untitled";
    const lines = ["---"];
    lines.push(`title: "${title}"`);
    if (page.description) {
        lines.push(`description: "${page.description}"`);
    }
    if (page.sidebarLabel && page.sidebarLabel !== title) {
        lines.push(`sidebar:`);
        lines.push(`  label: "${page.sidebarLabel}"`);
    }
    lines.push("---");
    return lines.join("\n");
}

/**
 * Generate docs pages, optionally to a custom output directory.
 * @param {string} [outputBase] - Override output base directory (for freshness checks)
 * @returns {{ generated: string[], skipped: string[] }}
 */
export function generateDocsPages(outputBase) {
    const mapping = JSON.parse(readFileSync(join(__dirname, "docs-mapping.json"), "utf-8"));
    const linkMap = buildLinkMap(mapping.pages, mapping.basePath ?? "");
    const outDir = outputBase || join(siteRoot, mapping.outputDir);
    const generated = [];
    const skipped = [];

    for (const page of mapping.pages) {
        const sourcePath = join(projectRoot, page.source);
        if (!existsSync(sourcePath)) {
            console.warn(`  SKIP: ${page.source} (not found)`);
            skipped.push(page.source);
            continue;
        }

        const raw = readFileSync(sourcePath, "utf-8");
        const stripped = stripFrontmatter(raw);
        const extractedTitle = extractTitle(stripped);
        const frontmatter = buildFrontmatter(page, extractedTitle);

        // Strip HTML comments which are invalid in MDX
        let body = stripped.replace(/<!--[\s\S]*?-->/g, "");
        // Rewrite markdown links to Starlight URLs
        body = rewriteLinks(body, linkMap);
        const h1Match = body.match(/^#\s+.+\n+/);
        if (h1Match) {
            body = body.slice(h1Match[0].length);
        }

        const content = `${frontmatter}\n\n${body.trim()}\n`;
        const outPath = join(outDir, page.output);
        mkdirSync(dirname(outPath), { recursive: true });
        writeFileSync(outPath, content, "utf-8");
        console.log(`  OK: ${page.output}`);
        generated.push(page.output);
    }

    return { generated, skipped };
}

// Run directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
    console.log("Generating docs pages...");
    const { generated, skipped } = generateDocsPages();
    console.log(`\nDone: ${generated.length} generated, ${skipped.length} skipped.`);
}
