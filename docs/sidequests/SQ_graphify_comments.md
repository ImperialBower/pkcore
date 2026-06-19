The graph is built and saved. The doc-extraction subagents had actually completed their work and written valid chunk files before the claude-fable-5 error
surfaced, so once you switched to Opus 4.8 there was nothing to re-run — I picked up the 12 finished chunks and completed the pipeline.

Graph complete. Outputs in /Users/christoph/src/github.com/ImperialBower/pkcore/graphify-out/

- graph.html — interactive graph (aggregated to 259 community nodes, since the full graph's 6,262 nodes exceed the 5,000 viz limit). Run /graphify .
  --obsidian if you want full node-level detail.
- GRAPH_REPORT.md — audit report
- graph.json — raw graph data (6,262 nodes, 13,859 edges, 259 communities)

Scope was src/ + docs/ (272 files): 159 Rust files via free AST extraction (5,963 nodes) plus 113 docs/images via semantic extraction (398 nodes, 728
edges). Est. ~109K input / ~24.5K output tokens.

God Nodes (your core abstractions)

TableNoCell (138 edges) and TableCelled (131) dominate — the two table implementations are the gravitational center of the codebase — followed by Cards
(97), Seats (73), Result (57), PKError (53), Five (52), HandHistory (52), BotProfile (51), and HUPResult (50).

Surprising Connections

The extraction found design docs describing concepts that mirror code without any import linking them:
- Preflop Odds Web Cache Service (EPIC-08) ↔ SolverCache — your web caching design and the GTO solver cache solve the same problem independently.
- Four poker playing styles matrix (ANALYSIS_Player_Types) ↔ BotProfile — the tight/loose × passive/aggressive matrix is the conceptual root of the code
  struct.
- Three separate hand-history formats — OHH (EPIC-19a), the compact serialization format (EPIC-66), and the Nubibus/Pluribus log analysis — all map back to
  one HandHistory struct.

Suggested Questions

The graph flags TableNoCell (betweenness 0.200) and TableCelled (0.087) as the two big cross-community bridges — they wire together 20+ communities each. It
also notes 736 weakly-connected nodes (possible doc gaps) and flags Hand Range Macros (cohesion 0.010) and Table Seat Cells (0.055) as candidates for
splitting into more focused modules.

The most interesting question this graph can answer: Why is TableNoCell the single largest bridge in the codebase, connecting 22 separate communities? Want
me to trace it?