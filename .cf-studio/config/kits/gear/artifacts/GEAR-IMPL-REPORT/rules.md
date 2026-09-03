# GEAR-IMPL-REPORT Rules

**Artifact**: GEAR-IMPL-REPORT
**Kit**: gear

```pdsl
UNIT GearImplReportAuthoring

PURPOSE:
  Record the gate-audited close-out of one implementation phase.

WHEN:
  - REQUIRE closing an implementation phase

DO:
  - LOAD {gear_impl_report_template} for structure
  - SET ID = cpt-{hierarchy-prefix}-report-{slug}; one report per phase
  - RUN copy the phase's gate commands verbatim into Gate Results and execute them; record outcomes with evidence
  - RUN python3 {scripts}/check_phase_gates.py <decomposition-file> --report <report-file> — the report covers every gate of its phase

RULES:
  - ALWAYS write the report only after every gate command exits zero — a report over failing gates is a violation
  - ALWAYS record deviations honestly; "None" is an explicit statement, not an omission
  - ALWAYS reroute material deviations through the decomposition workflow before the next phase
  - NEVER edit a closed report except to correct evidence links; new work gets a new report
```

```pdsl
UNIT GearImplReportOmissions

PURPOSE:
  Enforce GEAR-IMPL-REPORT scope boundaries. Report as a violation if found.

RULES:
  - NEVER report a gate outcome without a command execution behind it (GEAR-RPT-NO-001, CRITICAL)
  - NEVER omit a gate command that the phase declares (GEAR-RPT-NO-002, CRITICAL)
  - NEVER plan future work beyond naming the next phase (GEAR-RPT-NO-003, LOW) — planning lives in the decomposition
```

```pdsl
UNIT GearImplReportValidate

PURPOSE:
  Run deterministic validation on the GEAR-IMPL-REPORT.

DO:
  - RUN cfs validate --artifact <path> — must report PASS
  - RUN cfs check-language <artifact-file> — must report PASS
  - RUN python3 {scripts}/check_phase_gates.py <decomposition-file> --report <report-file> — must report PASS
```
