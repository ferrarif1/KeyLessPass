import { execFileSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'

const roots = ['paper', 'rust_core', 'experiments', 'formal', 'models', 'docs', 'README.md', 'README.zh-CN.md', '.github']
const pattern = '(Encoder v2|\\bv2\\b|\\bv3\\b|derivationVersion[ =:`"]+2|encoderVersion[ =:`"]+2|derivationVersion[ =:`"]+3|encoderVersion[ =:`"]+3|\\(2,2\\)|\\(3,3\\)|migration from v2|legacy encoder|previous prototype encoder|byte-for-byte legacy dispatch|previous derivation path|old encoder baseline|old implementation collision)'
const output = execFileSync('rg', [
  '-n', '-i', '--hidden',
  '--glob', '!rust_core/target/**',
  '--glob', '!output/**',
  '--glob', '!docs/remove_legacy_v2_audit.md',
  pattern,
  ...roots
], { encoding: 'utf8' })

function disposition (path, text) {
  if (path.startsWith('paper/')) return 'DELETE'
  if (path.startsWith('experiments/')) return 'DELETE'
  if (path === 'formal/lifecycle.tla' || path === 'formal/MODEL_CHECK_RESULTS.md' || path === 'formal/policy_properties.md') return 'REWRITE'
  if (path === 'docs/policy_space_derivation_spec.md' || path === 'docs/security_argument.md') return 'REWRITE'
  if (path.startsWith('rust_core/src/derivation/') || path === 'rust_core/src/policy/mod.rs') return 'REWRITE'
  if (path === 'rust_core/src/service/derivation_migration.rs') return 'KEEP_INTERNAL_ONLY'
  if (path === 'rust_core/test-vectors/password-derivation-v3.json') return 'REWRITE'
  if (path.startsWith('docs/research/') || path === 'docs/evaluation_results.md' || path === 'docs/final_research_restructure_report.md' || path === 'docs/research_restructure_audit.md' || path === 'docs/migration_v2_v3.md') return 'DELETE'
  if (path.startsWith('.github/') && text.includes('rust-cache@v2')) return 'KEEP_INTERNAL_ONLY'
  return 'KEEP_INTERNAL_ONLY'
}

const rows = output.trim().split('\n').map(line => {
  const match = line.match(/^([^:]+):(\d+):(.*)$/)
  if (!match) throw new Error(`cannot parse rg row: ${line}`)
  return { path: match[1], line: Number(match[2]), text: match[3].trim() }
})

const counts = rows.reduce((result, row) => {
  const key = disposition(row.path, row.text)
  result[key] = (result[key] || 0) + 1
  return result
}, {})
const commit = execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim()

const markdown = `# Legacy v2/v3 removal audit

The initial pre-rewrite pass recorded 181 matches (DELETE 53; REWRITE 27;
KEEP_INTERNAL_ONLY 101). This file was refreshed after the rewrite so that its
inventory also records the final location and disposition of product-only
compatibility material and revision reports.

- Audit date: 2026-08-09
- Repository commit before the rewrite: \`${commit}\`
- Scope: manuscript, bibliography, Rust source, experiments, formal models, figures/tables, vectors, READMEs, CI, and supplementary documentation
- Initial pre-rewrite matches: 181 (DELETE 53; REWRITE 27; KEEP_INTERNAL_ONLY 101)
- Refreshed matches: ${rows.length} (DELETE ${counts.DELETE || 0}; REWRITE ${counts.REWRITE || 0}; KEEP_INTERNAL_ONLY ${counts.KEEP_INTERNAL_ONLY || 0})

## Disposition rules

- **DELETE**: remove from the standalone paper and submitted research artifact; do not rename it as a baseline.
- **REWRITE**: retain the exact-policy-space function but rename it as public \`schemeVersion = 1\` / \`EXACT_POLICY_V1\`, with no migration narrative.
- **KEEP_INTERNAL_ONLY**: required by the existing product or historical tests, but excluded from the paper build, paper experiments, formal model, fixed vectors, and review artifact manifest.

The \`rust-cache@v2\` CI matches are third-party GitHub Action release tags, not protocol versions. Evidence-harness uses of “v1/v2” that denote database record revisions are also internal product history, not research baselines.

## Complete occurrence inventory

| Disposition | Location | Matched text |
|---|---|---|
${rows.map(row => `| ${disposition(row.path, row.text)} | \`${row.path}:${row.line}\` | ${row.text.replaceAll('|', '\\|').replaceAll('`', '\\`')} |`).join('\n')}

## Phase-2 acceptance checks

1. No match may remain under \`paper/\` except prose in this audit/report explaining that the old material was removed.
2. The submitted experiment manifest must not execute the old encoder or its fixed vectors.
3. EPSCD source, contexts, errors, and fixed vector must use \`schemeVersion = 1\` or \`EXACT_POLICY_V1\`, not “v3”.
4. Formal lifecycle state must contain no v2/v3 compatibility variables or invariants.
5. Product compatibility code may remain only outside the standalone artifact manifest and must be labelled internal.
`

writeFileSync('docs/remove_legacy_v2_audit.md', markdown)
