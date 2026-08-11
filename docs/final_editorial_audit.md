# Final editorial audit before submission

Audit date: 2026-08-09  
Target: *Exact Policy-Space Credential Derivation for Legacy Password Rotation*  
Target class: applied information-security journal (JISA-like scope)

This audit records the state of the manuscript before the final revision. It is
not evidence that a requested citation or venue exists. In particular, the
MFDPG venue requested in the revision brief could not be verified: the official
NDSS 2024 accepted-paper list does not contain MFDPG, while arXiv and the
authors' repository identify a 2023 preprint and an archived artifact. The
revision must preserve that publication status.

| Section or element | Decision | Novelty | Related work | Experiment fairness | Security claims | Style |
|---|---|---|---|---|---|---|
| Abstract | REWRITE | Correct core boundary, but the contribution is obscured by implementation counts. | Does not need a literature catalogue. | Reports a pre-filtered 55-policy run as if it represented the translated corpus. | Correctly distinguishes ideal permutation and prototype, but ends with a validation-report sentence. | Too many numbers and “X of Y” clauses. |
| Introduction | REWRITE | The generation-indexed without-replacement sequence is assessable, but repeated disclaimers interrupt the argument. | Add MFDPG as the closest deterministic-system preprint while retaining published predecessors. | Artifact inventory belongs later. | Keep the distinction between local unbiased choices and final-language uniformity. | Reduce meta-discourse and repeated “does not claim” constructions. |
| Contributions | REWRITE | Retain exact indexing, sequence semantics, history exclusion, and remote evidence. Do not present ordinary recovery as a contribution. | Identify which ingredients are established. | Describe evidence categories, not a checklist of tests. | Tie each theorem to its abstraction. | Replace implementation-inventory prose with three research contributions. |
| Table 1 | REWRITE | Current property columns make the EPSCD row visually dominant. | Add MFDPG with its verified preprint status; do not label it NDSS 2024. | Replace checkmarks with descriptive generation, policy, space, sequence, and rotation semantics. | Use “not specified” for absent paper claims. | Use compact prose rather than marketing-style ticks. |
| Section 2: deterministic generation | EXPAND | Position EPSCD narrowly against deterministic password systems. | Read and discuss MFDPG's policy generation, multi-factor derivation, and revocation mechanism; distinguish it from published peer-reviewed baselines. | Artifact observations must not be conflated with a peer-reviewed-system benchmark. | Do not call MFDPG biased, broken, or insecure. | Integrate prior art as a narrative rather than one paragraph per disclaimer. |
| Section 2: policy representation | KEEP | Correctly treats policy representation as prior art. | Corpus and measurement citations are relevant. | Clarify that translation coverage and compilation completion are different measurements. | No material overclaim. | Minor compression only. |
| Section 2: formal-language generation | REWRITE | Correctly concedes counting/ranking and rank-then-encipher prior art. | Keep Dichopile as the published uniform-language baseline. | Explain that the artifact uses exact integers rather than the paper's optimized numerical configuration. | Avoid turning empirical TVD into a proof. | Reduce dense citation-list prose. |
| Section 5: finite-domain permutation | EXPAND | This is the mathematical core only in its credential-sequence specialization. | Retain arbitrary-domain FPE and FF1 references. | Add corpus-wide walk observations. | Recheck the falling-factorial tail, power-of-two boundary, cap semantics, and real-FF1/ideal-permutation separation. | Move the full derivation to a formal supplement; keep the principal result in the body. |
| Section 6: history and rotation | EXPAND | Uniform history exclusion and evidence-bounded commit are the strongest lifecycle contributions. | Consider related rotation/history work only where directly comparable. | Model checking is evidence for the bounded model, not remote adapters. | Replace Theorem 5 proof sketch with a formal proof; define the exclusion set after deduplication. | Reduce state-machine narration where the figure already carries it. |
| Implementation | KEEP | Appropriate separation of compiler, permutation interface, and lifecycle code. | No new literature gap. | State what is cached for warm measurements. | Keep the concrete backend explicitly partial and fail-closed. | Remove backticks used as prose quotation marks in LaTeX. |
| Evaluation preamble | REWRITE | Classify evidence by research role. | Distinguish published competitor, published algorithmic baseline, preprint artifact observation, mathematical control, and ours. | State host and fixed resource budgets before results. | Avoid “validates theorem” language for samples. | Replace project-report framing with hypotheses and measurements. |
| Section 8.1: policy corpus | REWRITE | Coverage is feasibility evidence, not novelty. | Preserve SOUPS corpus provenance. | Run every one of the 121 exact translations under one predeclared state/time/memory budget; remove `resource-skipped`. | Resource-limit outcomes are results, not unsupported semantics. | Report distributions and failure classes without “all passed”. |
| Section 8.2: MFDPG | EXPAND | Closest deterministic system comparison, but not a claimed published competitor unless publication is verified. | Include source-derived and empirical artifact observations separately. | Use the pinned official repository plus a minimal harness; disclose any substituted factor-to-seed stage. | Do not infer security failure from an enumerable toy policy. | Use neutral wording even if output selection is non-uniform for the tested expression. |
| Section 8.3: Dichopile | REWRITE | Establishes that uniform formal-language generation is prior art. | Keep publication details and algorithm boundary. | Split cold initialization and warm generation for both systems. | Exact arithmetic is an artifact design choice, not the only Dichopile mode. | Do not state a speed-up factor. |
| Non-repetition experiment | MOVE_TO_SUPPLEMENT | The theorem is stronger than 2,000 observed generations. | No comparison with unpublished Encoder versions. | Keep as an implementation consistency check. | Never use the sample as proof of injectivity. | Remove from abstract and reduce正文 prominence. |
| Local performance | REWRITE | Performance supports feasibility only. | Dichopile is not a deterministic credential-system competitor. | Define cold and warm operations symmetrically; report median, P95, P99, and SD. | Exclude storage/network/remote confirmation. | Avoid comparative adjectives unsupported by a matched implementation study. |
| Lifecycle model checking | KEEP | Useful protocol evidence. | No comparison needed. | Report configuration, bounds, state count, depth, and invariant set once. | Say TLC reported no violation within the explored bounds. | Remove “all invariants passed” language. |
| Security discussion | EXPAND | Organize unconditional, ideal-model, and computational properties. | No additional catalogue needed. | Connect walk data to availability only as an observation. | State PRP advantage and resource failure separately; history descriptors are authenticated and deduplicated. | Prefer direct propositions to claim-boundary repetition. |
| Limitations | KEEP | Appropriately limits novelty and deployment scope. | Mention MFDPG status only in related work, not here. | Retain corpus age, platform, and adapter gaps. | Keep Unicode/context rules, state explosion, partial backend, and bounded model limitations. | Remove caveats already stated verbatim elsewhere. |
| Conclusion | REWRITE | State the solved problem and resulting sequence property. | No repeated survey. | Summarize evidence without an inventory of every test. | End with the central limitation: proof-matched total finite-domain backend and broader policy/deployment evidence. | Compress to one or two paragraphs and remove repeated disclaimers. |
| Mathematical controls | MOVE_TO_SUPPLEMENT | Useful diagnostics, not competitors. | Label random mapping, hash modulo, identity permutation, rejection sampling, and uniform rank oracle as controls. | Keep inputs and sample sizes reproducible. | Do not use them to imply a published method is deficient. | Present as a single supplementary table. |
| Recovery profile | KEEP | Supporting functionality only. | Shamir and MFKDF already establish threshold recovery. | No new recovery-performance claim. | Keep factor and share-generation semantics explicit. | Do not let recovery dominate the paper title or contributions. |

## Blocking revisions

1. Replace the 55-policy pre-filter with attempted compilation of all 121
   exact translations under a fixed budget and classify each termination.
2. Add MFDPG accurately: nearest technical preprint and official artifact, not
   a fabricated NDSS 2024 paper.
3. Split cold and warm EPSCD/Dichopile measurements and state the non-equivalent
   algorithmic roles.
4. Supply a full proof of uniform history exclusion and a checked cycle-walk
   derivation.
5. Rewrite the abstract, comparison table, evaluation prose, and conclusion
   after results are fixed.

## Submission risk before revision

The core property is intelligible, but an editor could still view the paper as
an application of established rank-then-encipher machinery with selectively
reported corpus feasibility. The full-corpus resource experiment and neutral
nearest-work positioning are therefore prerequisites for external review, not
cosmetic additions.
