# GEAR-INTENT Rules

**Artifact**: GEAR-INTENT
**Kit**: gear

```pdsl
UNIT GearIntentAuthoring

PURPOSE:
  Record one classified gear-creation request as the root of the gear's traceability chain.

WHEN:
  - REQUIRE recording or revising a gear-creation intent

DO:
  - LOAD {gear_intent_template} for structure
  - RUN read project config for ID prefix from {cf-studio-path}/config/artifacts.toml
  - RUN python3 {scripts}/find_similar_gears.py --catalog {gear_catalog} with the request's capability keywords
  - RUN record the lookup results verbatim in Similar Gears; never omit a close match
  - SET the classification from the routing table: public-contract impact => never small fix; capability owned by an existing gear => plugin or feature
  - SET the reuse profile from the archetype tables shipped with the catalog
  - RUN fill Document Set from the classification (full set for new gear; host specs reuse for plugin; none for small fix)

RULES:
  - ALWAYS follow {gear_intent_template} structure; all sections present and non-empty
  - ALWAYS use ID convention cpt-{hierarchy-prefix}-intent-{slug}
  - ALWAYS state the reuse-or-build call explicitly, with the losing options named
  - NEVER leave placeholders (TODO, TBD, FIXME)
  - NEVER proceed past intent while the requester has not confirmed the route decision
```

```pdsl
UNIT GearIntentOmissions

PURPOSE:
  Enforce GEAR-INTENT scope boundaries. Report as a violation if found.

RULES:
  - NEVER include requirements catalogs (GEAR-INT-NO-001, CRITICAL) — requirements belong in GEAR-PRD
  - NEVER include architecture or crate layout (GEAR-INT-NO-002, CRITICAL) — they belong in GEAR-DESIGN
  - NEVER classify a public-contract change as a small fix (GEAR-INT-NO-003, CRITICAL)
  - NEVER skip or truncate the similar-gears lookup (GEAR-INT-NO-004, HIGH) — an empty result must show the query used
  - NEVER present the reuse profile without naming the reference gear (GEAR-INT-NO-005, MEDIUM)
```

```pdsl
UNIT GearIntentValidate

PURPOSE:
  Run deterministic validation on the GEAR-INTENT.

DO:
  - RUN cfs toc <artifact-file>
  - RUN cfs validate --artifact <path> — must report PASS
  - RUN cfs validate-toc <artifact-file> — must report PASS
  - RUN cfs check-language <artifact-file> — must report PASS
```
