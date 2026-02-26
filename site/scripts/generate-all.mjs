#!/usr/bin/env node

/**
 * Master generation script. Runs all content generators.
 */

import { generateDocsPages } from "./generate-docs-pages.mjs";
import { generateWorkflowPages } from "./generate-workflow-pages.mjs";
import { generateReferencePages } from "./generate-reference-pages.mjs";

console.log("=== Generating docs pages ===");
const docs = generateDocsPages();

console.log("\n=== Generating workflow pages ===");
const workflows = generateWorkflowPages();

console.log("\n=== Generating reference pages ===");
const reference = generateReferencePages();

const total = docs.generated.length + workflows.generated.length + reference.generated.length;
console.log(`\nAll generation complete. ${total} pages generated.`);
