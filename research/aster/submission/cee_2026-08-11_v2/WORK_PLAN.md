# ASTER CEE visual and typesetting revision plan

Prepared: 11 August 2026

## Evidence boundary

- Preserve all technical claims, experimental results, theorem statements, and limitations.
- Draw Figure 2 from `data/rq2_summary.json` and `data/table_rq2.csv`.
- Draw Figure 3 from `data/rq4_summary.json` and `data/table_rq4.csv`; the share-refresh control is shown as persistently exposed because the recorded experiment states that refreshing shares preserved all outputs.
- Do not introduce new measurements, interpolate unmeasured points, or treat negative controls as published baselines.

## Work sequence

1. Rebuild the graphical abstract as two concise swimlanes.
2. Rebuild Figure 1 as a numbered normal-operation architecture with a separate healing inset.
3. Rebuild Figure 2 as accepted-output and unauthorized-spill panels using the six recorded authorization budgets.
4. Rebuild Figure 3 with old-root exposure, healed credentials, and the share-refresh negative control.
5. Add Figure 4 from the Section 6.7 migration state machine without changing its transitions.
6. Normalize mathematical operator names, subscripts, structured records, captions, tables, and figure references in the manuscript source.
7. Generate submission-grade DOCX and PDF, render every page, and correct all visible defects.
8. Produce the revision log, delivery summary, integrity manifest, submission ZIP, and a new non-overwriting backup.

## Visual system

- White background; Arial/Liberation Sans labels; restrained blue, teal, amber, red, and gray palette.
- Consistent 1.4-1.8 pt strokes, rounded boxes, filled arrowheads, and panel labels.
- SVG is the editable source; PNG exports target print-safe high resolution; the graphical abstract also receives a 300 dpi TIFF export.
- All manuscript figures are inserted inline to minimize Word/LibreOffice anchoring differences.

## Acceptance gates

- Graphical Abstract and Figures 1-4 have matching style and readable labels at manuscript size.
- Figure 4 is cited and placed next to Section 6.7.
- Figure/table numbering and textual references are consistent.
- Final DOCX/PDF contain no clipped figures, broken formulas, displaced subscripts, table overflow, or header/footer interference.
- Every final PDF page is visually inspected after the last rebuild.

## Completion record

All eight work-sequence items and all acceptance gates were completed on 11 August 2026. The final review manuscript is 32 A4 pages; its four figures, ten tables, equations, captions, and cross-references were checked after the last pagination change. The integrity-checked delivery archive is `final/ASTER_CEE_Submission_Package_v2.zip`.
