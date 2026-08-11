# ASTER CEE v2 revision log

Prepared: 11 August 2026

## Scope

This revision is a presentation and publication-engineering pass over the existing ASTER manuscript. It does not add experimental observations, change measured values, or alter the paper's technical conclusions.

## Figure revisions

- Rebuilt the graphical abstract as two coordinated swimlanes: normal exact-scope derivation and post-compromise Root-Epoch healing. The second lane explicitly distinguishes independent replacement from share refresh and retains both reconstruction paths under ambiguous evidence.
- Rebuilt Figure 1 as a compact two-panel architecture figure. Panel A numbers the six normal-operation interactions; Panel B separates independent Root-Epoch replacement, evidence-bounded migration, conclusive commit, and ambiguous preservation.
- Rebuilt Figure 2 as a two-panel capability-scope experiment. Panel A reports accepted outputs; Panel B reports unauthorized spill. Exact, projected service/account, and wildcard configurations use the recorded q values 1, 2, 4, 8, 16, and 32.
- Rebuilt Figure 3 with all three recorded sequences: old-root exposed, healed, and the share-refresh control. The control remains at 100 exposed credentials, matching the recorded negative-control result.
- Added Figure 4 near Section 6.7. It visualizes the manuscript's existing migration state machine, including NewOnly commit, OldOnly abort, the ambiguous-evidence transition to UnknownOutcome, and adapter-authorized reconciliation. The figure states that UnknownOutcome is a safety state and is never silently cleared.
- Exported every figure as SVG, PNG, and PDF. The graphical abstract is additionally exported as a 2656 x 1062 pixel, 300 dpi, LZW-compressed TIFF.

## Manuscript synchronization

- Replaced the former Figures 1-3 and inserted Figure 4 at its first substantive discussion.
- Moved Figures 2 and 3 from the evaluation section to the corresponding authorization and Root-Epoch protocol discussions; the evaluation section now points back to them.
- Rewrote all four captions so each figure is independently interpretable and identifies the semantics of its panels, curves, arrows, or evidence classes.
- Added an explicit text reference to Figure 4 in Section 6.7.
- Converted the durable migration candidate descriptor into editable Table 2 and renumbered the later tables consistently. The manuscript now contains Figures 1-4 and Tables 1-10 in first-appearance order.

## Formula and typography repairs

- Normalized Rank, Unrank, Eval, and Verify to upright mathematical operators.
- Standardized Root-Epoch keys, exact-domain quantities, effective entropy, permutations, and related subscripts throughout the manuscript.
- Normalized random key sampling to a dollar-marked assignment arrow rendered as a native Word equation.
- Rebuilt display and inline mathematics through Word's equation representation; removed space- or tab-based pseudo-alignment.
- Kept display equations together, stabilized spacing around theorem/proposition blocks, and visually checked proof endings.
- Converted structured protocol fields to a fixed-width editable table instead of tab-aligned text.

## Table and page-layout repairs

- Applied consistent table borders, header shading, cell margins, type sizes, column widths, alignment, repeating headers, and row non-splitting rules.
- Kept the complete candidate-descriptor table on one page.
- Standardized image width, centering, caption spacing, heading spacing, page margins, footer page numbers, and continuous line numbering.
- Suppressed visually competing manuscript headers.

## Evidence provenance

- Figure 2 was regenerated from `data/rq2_summary.json` and `data/table_rq2.csv`.
- Figure 3 was regenerated from `data/rq4_summary.json` and `data/table_rq4.csv`.
- `data/RESULT_PROVENANCE.json` records the mapping from submitted results to the revised figures.
- No missing data point was inferred or fabricated.

## Final verification

- Rendered the final DOCX to a 32-page A4 PDF and inspected every rendered page, with full-resolution checks of the title page, all figure pages, the repaired descriptor table, the principal evaluation tables, and the references.
- Confirmed four embedded figures, ten editable tables, continuous line numbering, no tracked insertions/deletions, no blank PDF pages, and no encrypted PDF content.
- Confirmed that fragmented operator spellings such as `U n r a n k` and `p w d` do not occur.
- Confirmed that figure/table numbering, captions, cross-references, abstract length, highlights, and the 21-item reference list remain internally consistent.
