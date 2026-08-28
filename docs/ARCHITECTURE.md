# KAKASHI

## Deterministic System Replication & Equivalence Engine

**Project:** `elci-group/kakashi`
**Language:** Rust
**Status:** Technical Directive
**Architectural Class:** Deterministic analysis, replication, validation and convergence utility
**Primary Principle:** *Observe → Characterise → Model → Reproduce → Validate → Diff → Converge*

This document is the governing architectural directive for Kakashi, as specified. See `README.md` for what is actually implemented today versus what remains a directive for future work.

---

## 1. Mission

Kakashi shall be the ELCI deterministic copycat utility.

Its purpose is to take a **reference system** and produce a **deterministic replica** whose externally observable behaviour, interfaces, configuration semantics, structural properties and selected implementation characteristics converge toward those of the reference.

Kakashi shall support replication of:

* CLI applications
* libraries
* HTTP/API services
* daemons
* configuration-driven applications
* data-processing utilities
* developer tools
* Rust applications
* polyglot applications
* local binaries
* repository-level software systems

Kakashi shall explicitly distinguish between:

1. **Behavioural equivalence**
2. **Interface equivalence**
3. **Configuration equivalence**
4. **Data/schema equivalence**
5. **Structural equivalence**
6. **Operational equivalence**
7. **Performance equivalence**
8. **Binary equivalence**

A replication job shall declare which equivalence dimensions are required.

Kakashi must never claim full equivalence merely because a candidate compiles or produces superficially similar output.

---

# 2. Core Doctrine

Kakashi shall operate according to five rules.

### Rule 1 — Evidence before implementation

Kakashi must characterise the reference before attempting to reproduce it.

### Rule 2 — Determinism before intelligence

LLMs may assist interpretation and implementation, but the surrounding system must determine:

* what was observed;
* what constitutes evidence;
* what tests were generated;
* what differences exist;
* what constitutes convergence;
* whether a mutation is permitted.

### Rule 3 — Reference and replica are immutable evidence sources

The reference system shall be treated as an experimental control.

Kakashi shall never modify the reference during normal replication.

### Rule 4 — Every claim must be testable

A statement such as:

> "Kakashi has reproduced the target."

is invalid without an associated equivalence report.

### Rule 5 — Replication is iterative convergence

Kakashi shall continue through:

```text
REFERENCE
    ↓
OBSERVATION
    ↓
CHARACTERISATION
    ↓
SYSTEM MODEL
    ↓
REPLICA
    ↓
VALIDATION
    ↓
DIFFERENTIAL ANALYSIS
    ↓
REPAIR
    ↓
REVALIDATION
```

until either:

* the declared equivalence gates pass;
* the configured convergence budget is exhausted;
* an irreducible incompatibility is identified;
* human intervention is required.

---

# 3. ELCI Integration Doctrine

Kakashi shall not independently recreate functionality already provided by ELCI tooling.

Instead it shall operate as an orchestration layer over the ELCI ecosystem.

The intended architecture is:

```text
                         KAKASHI
                            │
             ┌──────────────┼──────────────┐
             │              │              │
         OBSERVE         MODEL          VALIDATE
             │              │              │
       ┌─────┴─────┐    ┌───┴────┐    ┌────┴─────┐
       │           │    │        │    │          │
   Isopod       Traci Fract   Chakra  Deckhand   Poka
       │           │    │        │       │         │
       └───────────┴────┴────────┴───────┴─────────┘
                            │
                         PADAGONIA
                            │
                     persistent evidence
                            │
       ┌────────────────────┼────────────────────┐
       │                    │                    │
   Cambrian              Salience            Tempcheq
   decisions             priority             effort
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                         EXECUTE
                            │
                         Kaptaind
                            │
                  version / Git / release
```

The integrations below are mandatory architectural adapters rather than optional conceptual references.

---

# 4. PADAGONIA — Kakashi's Evidence Graph

PADAGONIA shall be Kakashi's preferred persistent knowledge substrate.

PADAGONIA already provides ontology-aware vocabulary, immutable nodes/edges, provenance and confidence metadata, parallel binary storage, traversal, filtering, vector search, snapshots and restore. Kakashi shall exploit these capabilities rather than maintaining a second bespoke knowledge graph.

Kakashi shall represent at minimum:

```text
ReferenceSystem
    ├── Artifact
    ├── Binary
    ├── File
    ├── Module
    ├── Symbol
    ├── Interface
    ├── Command
    ├── Input
    ├── Output
    ├── Configuration
    ├── Dependency
    ├── Behaviour
    ├── Observation
    ├── Test
    ├── Difference
    ├── Hypothesis
    ├── Replica
    └── EquivalenceClaim
```

Relations shall include:

```text
contains
depends_on
calls
imports
exports
accepts
produces
observed_as
equivalent_to
differs_from
derived_from
validated_by
contradicts
implements
generated_from
supersedes
```

Every significant observation must carry:

* source;
* timestamp;
* evidence;
* confidence;
* provenance;
* execution context;
* reference version;
* replica version;
* observation method.

The graph therefore becomes Kakashi's **persistent system model**, not merely a cache.

---

# 5. ISOPOD — Environmental Observation

Isopod shall be used for environmental and operational observation where available.

Kakashi must distinguish:

```text
SYSTEM BEHAVIOUR
vs
ENVIRONMENTAL BEHAVIOUR
```

For example:

```text
"binary returns exit code 2"
```

is different from:

```text
"binary returns exit code 2 on this machine because dependency X is unavailable"
```

Environmental observations shall therefore record:

* OS;
* architecture;
* kernel;
* CPU;
* available memory;
* filesystem;
* environment variables;
* installed dependencies;
* PATH;
* locale;
* timezone;
* network availability;
* relevant hardware capabilities.

This prevents environmental accidents from being incorrectly incorporated into the replica.

---

# 6. TRACI — Behavioural Trace

Traci shall form the primary behavioural tracing interface.

Every executable replication experiment should generate a structured trace containing:

```text
input
pre-state
action
stdout
stderr
exit_code
filesystem_delta
network_delta
process_delta
output_artifacts
timing
post-state
```

Where possible, Kakashi shall capture traces at multiple levels:

```text
L0 — process
L1 — syscall / OS observable
L2 — filesystem
L3 — network
L4 — application interface
L5 — semantic output
```

Kakashi shall never rely exclusively on stdout comparison when deeper observable behaviour can be tested.

---

# 7. FRACT — Structural Understanding

Fract shall be Kakashi's preferred architectural-analysis engine for source replicas.

Kakashi should delegate:

* module analysis;
* complexity analysis;
* cohesion analysis;
* dependency analysis;
* duplication analysis;
* architectural entropy detection;
* structural comparison.

Fract's existing model of indexing a project, detecting structural entropy, producing semantic context and validating changes provides an appropriate precedent for Kakashi's structural convergence subsystem.

Kakashi shall compare:

```text
REFERENCE STRUCTURE
        ↓
REFERENCE ARCHITECTURE
        ↕
REPLICA ARCHITECTURE
```

and identify whether a behavioural match is being achieved through:

* genuine architectural equivalence;
* a compatible alternative implementation;
* accidental equivalence;
* test-specific hardcoding.

The last category must be explicitly detected.

---

# 8. CHAKRA — Dataflow Reconstruction

Chakra shall be used where dataflow reconstruction is required.

Kakashi shall construct a dataflow model:

```text
INPUT
  ↓
PARSER
  ↓
NORMALISER
  ↓
TRANSFORM
  ↓
DECISION
  ↓
OUTPUT
```

The model shall allow Kakashi to identify:

* transformation stages;
* branching;
* state transitions;
* dependencies;
* side effects;
* information loss;
* nondeterministic boundaries.

This is particularly important where two implementations produce the same output but reach it through materially different paths.

---

# 9. CAMBRIAN — Replication Decision Engine

Cambrian shall be used for ranked intervention selection.

Kakashi should not blindly attempt every possible repair.

Each discovered difference shall become a candidate intervention:

```text
Difference
    ↓
Possible causes
    ↓
Possible interventions
    ↓
Cost
    ↓
Risk
    ↓
Expected convergence
    ↓
Rank
```

Cambrian should therefore answer:

> "What is the highest-value deterministic change to the replica that is most likely to eliminate the largest verified equivalence gap?"

Kakashi itself remains responsible for enforcing the resulting decision.

---

# 10. SALIENCE — Difference Prioritisation

Kakashi may discover hundreds or thousands of differences.

These must not all receive equal attention.

Salience shall prioritise differences according to:

```text
impact
confidence
frequency
user visibility
equivalence importance
repair cost
regression risk
```

Example:

```text
1. Wrong exit code       → critical
2. Wrong JSON schema     → critical
3. Wrong default config  → high
4. Wrong log timestamp   → low
5. Internal naming       → informational
```

The equivalence contract determines the final weighting.

---

# 11. TEMPCHEQ — Effort and Inference Control

Tempcheq shall control expensive replication work.

Kakashi must not repeatedly spend inference or compute budget analysing already-understood behaviour.

Experiments should be assigned an expected information value:

```text
InformationGain / Cost
```

Low-value experiments shall be deferred.

Caching must operate at several levels:

```text
observation cache
trace cache
parse cache
build cache
test cache
inference cache
equivalence cache
```

Cache keys must include all relevant determinism inputs.

---

# 12. DECKHAND — Build-Surface Hygiene

Deckhand shall be used to maintain deterministic build surfaces.

Before characterisation or reproduction:

```text
clean workspace
normalised artifacts
known build state
known generated files
known dependency state
```

Kakashi must explicitly distinguish:

```text
source state
generated state
build state
runtime state
```

Build artefacts must never silently become source evidence.

---

# 13. POKA — Execution / Experimentation Layer

Poka shall serve as the controlled experimentation layer where applicable.

Every experiment must have:

```text
experiment_id
reference_id
replica_id
input_fixture
environment_fingerprint
execution_policy
timeout
resource_limits
expected_observables
actual_observables
result
```

Experiments must be replayable.

A successful experiment should therefore be executable again from its recorded manifest.

---

# 14. KAPTAIND — Git, Versioning and Provenance Boundary

Kaptaind shall own Kakashi's repository lifecycle wherever practical.

Kakashi shall not invent an independent versioning regime.

Kaptaind already performs semantic change clustering, diff analysis, automated versioning, Git commit generation, audit persistence and release qualification.

Kakashi shall therefore emit structured change evidence that Kaptaind can consume.

A replication iteration should produce a semantic change record such as:

```text
replication:
  target: <reference>
  iteration: 17
  equivalence_delta: -0.083
  behaviours_fixed: 12
  behaviours_regressed: 0
  structural_delta: +0.014
  tests_added: 9
  confidence: 0.96
```

Kaptaind then determines the appropriate repository/version transition.

Kakashi must never silently mutate the Git history.

---

# 15. AMBER — Dependency Determinism

Dependency differences must be explicitly characterised.

Amber shall be used to assess dependency policy and replacement/optimisation where available.

Kakashi must distinguish:

```text
reference dependency
replica dependency
dependency version
dependency feature flags
transitive dependency
system dependency
```

A dependency substitution is only considered equivalent after validation.

---

# 16. SHATTER / FRACT — Controlled Structural Repair

Where a structural mismatch is identified, Kakashi shall prefer deterministic transformation mechanisms over arbitrary LLM rewriting.

The repair hierarchy shall be:

```text
1. deterministic transformation
2. known ELCI refactoring primitive
3. constrained generated patch
4. LLM-assisted implementation
5. human intervention
```

Any LLM-generated repair must subsequently become ordinary source code subject to deterministic validation.

Kakashi must never treat model confidence as proof of equivalence.

---

# 17. 3FORM — Output Normalisation

3form shall be used where output formatting is part of the observable contract.

Kakashi must distinguish:

```text
semantic difference
presentation difference
```

For example:

```json
{"a":1,"b":2}
```

and:

```json
{
  "b": 2,
  "a": 1
}
```

may be semantically equivalent but textually different.

The equivalence policy must declare which comparison applies:

```text
byte
text
token
syntax tree
semantic structure
```

---

# 18. MATRON — Dependency / Binary Acquisition

Matrón shall be used as the preferred ELCI package/binary acquisition mechanism where applicable.

Kakashi must record:

* package source;
* package version;
* artifact hash;
* architecture;
* build provenance;
* installation method.

A replicated environment must be reproducible from its acquisition manifest.

---

# 19. PADAGONIA + DREAMSEQ — Long-Term Replication Memory

Kakashi shall preserve successful replication knowledge.

A completed replication shall generate an **Equivalence Chronicle** containing:

```text
reference fingerprint
environment fingerprints
system model
observations
experiments
failed hypotheses
successful hypotheses
repairs
regressions
final equivalence score
known limitations
```

Dreamseq may subsequently analyse these histories to identify recurring replication patterns.

The objective is that Kakashi becomes progressively better at recognising classes of systems without sacrificing deterministic execution.

---

# 20. VASILIS / VISUAL ASSET PIPELINE

Where a reference application's visual interface is part of its contract, Kakashi shall expose visual evidence to the ELCI visual-asset pipeline.

Visual equivalence may include:

* logos;
* icons;
* screenshots;
* layout;
* typography;
* colour tokens;
* spacing;
* component hierarchy.

Kakashi shall not treat visual similarity as a substitute for behavioural equivalence.

Visual comparison is an independent equivalence dimension.

---

# 21. ELCI EVIDENCE MODEL

All Kakashi subsystems shall emit the common ELCI evidence envelope.

Minimum schema:

```text
subject
source
timestamp
evidence
confidence
severity
measurements
relationships
proposed_actions
provenance
```

Kakashi-specific extensions:

```text
reference_id
replica_id
experiment_id
equivalence_dimension
observation_method
comparison_method
environment_fingerprint
input_fingerprint
output_fingerprint
```

No subsystem should invent an incompatible evidence format.

---

# 22. Deterministic Fingerprinting

Every replication target must receive a canonical fingerprint.

The fingerprint should cover, where applicable:

```text
source tree
file metadata
dependency graph
configuration
CLI surface
API surface
schema
binary
runtime environment
test corpus
behaviour corpus
```

Use domain-specific hashes rather than a single monolithic hash.

For example:

```text
SOURCE_SHA
DEPENDENCY_SHA
INTERFACE_SHA
CONFIG_SHA
BEHAVIOUR_SHA
BINARY_SHA
ENVIRONMENT_SHA
```

This permits precise diagnosis.

---

# 23. Reference Characterisation

Kakashi shall perform characterisation in stages.

### Stage A — Static discovery

Determine:

* files;
* languages;
* build system;
* dependencies;
* binaries;
* entry points;
* configuration;
* APIs.

### Stage B — Interface discovery

Determine:

* CLI commands;
* flags;
* environment variables;
* files;
* HTTP endpoints;
* schemas;
* exit codes.

### Stage C — Behavioural probing

Generate controlled inputs and record outputs.

### Stage D — Boundary probing

Probe:

* empty input;
* malformed input;
* extreme values;
* missing dependencies;
* unavailable resources;
* permission failures;
* timeouts;
* concurrency;
* repeated execution.

### Stage E — State reconstruction

Determine:

* persistent state;
* transient state;
* caches;
* generated files;
* databases;
* locks;
* side effects.

### Stage F — Performance characterisation

Measure:

* latency;
* throughput;
* memory;
* CPU;
* I/O;
* startup;
* steady-state behaviour.

---

# 24. Test Synthesis

Kakashi shall automatically construct a **Reference Behaviour Corpus**.

Every discovered behaviour should become a replayable fixture.

Example:

```text
fixtures/
  cli/
  api/
  malformed/
  boundary/
  filesystem/
  configuration/
  concurrency/
  failure/
  performance/
```

Each fixture shall contain:

```text
input
environment
expected_observables
comparison_policy
provenance
```

The reference corpus becomes the primary regression boundary for the replica.

---

# 25. Differential Execution

For every eligible test:

```text
INPUT
 ├──► REFERENCE
 │       ↓
 │   OBSERVATION
 │
 └──► REPLICA
         ↓
     OBSERVATION

REFERENCE OBSERVATION
          ↕
       COMPARATOR
          ↕
REPLICA OBSERVATION
```

The comparator must produce a structured difference rather than merely pass/fail.

Example:

```text
difference:
  dimension: exit_code
  reference: 2
  replica: 1
  severity: critical
  confidence: 1.0
```

---

# 26. Equivalence Scoring

Kakashi shall maintain independent scores.

```text
B = behavioural equivalence
I = interface equivalence
C = configuration equivalence
D = data/schema equivalence
S = structural equivalence
O = operational equivalence
P = performance equivalence
X = binary equivalence
```

A composite score may be calculated, but the individual scores must remain authoritative.

A high composite score must never conceal failure in a mandatory dimension.

Example:

```text
Behaviour       0.99
Interface       1.00
Configuration   0.97
Schema          1.00
Structure       0.81
Operations      0.94
Performance     0.88
Binary          N/A

Overall         0.94
Gate            FAIL
Reason          structural threshold not met
```

---

# 27. Anti-Cheating Controls

Kakashi must actively detect replicas that merely memorise the test corpus.

Controls shall include:

* hidden/generated probes;
* metamorphic tests;
* randomised valid inputs;
* randomised invalid inputs;
* boundary expansion;
* temporal variation;
* environment variation;
* state-reset testing;
* repeated execution;
* unseen fixture testing.

The reference corpus must therefore not be the entirety of the validation corpus.

---

# 28. Metamorphic Testing

Where exact expected outputs cannot be predicted, Kakashi shall derive relations.

Examples:

```text
input A → output A

transform(A) → output B

relationship(output A, output B) must hold
```

This is particularly useful for:

* parsers;
* formatters;
* numerical utilities;
* compilers;
* data processors;
* search systems.

Metamorphic relations shall themselves be recorded as evidence.

---

# 29. Failure Equivalence

Failure behaviour is part of the target.

Kakashi must characterise:

* error messages;
* error codes;
* exit codes;
* partial output;
* rollback behaviour;
* cleanup;
* logging;
* retries;
* timeouts;
* degraded modes.

The replica must not be considered equivalent merely because successful cases work.

---

# 30. Nondeterminism

Kakashi must explicitly identify nondeterministic outputs.

Examples:

```text
timestamps
random identifiers
process IDs
memory addresses
unordered collections
network timing
randomised algorithms
model outputs
```

The comparison engine shall classify these as:

```text
deterministic
conditionally deterministic
nondeterministic
unknown
```

Where nondeterminism is legitimate, Kakashi shall compare invariants rather than literal values.

---

# 31. Security Model

Kakashi shall treat the reference system as potentially hostile.

Never automatically execute:

* untrusted install scripts;
* arbitrary post-build hooks;
* unknown binaries;
* privileged operations;
* network requests;
* destructive commands.

Execution must occur inside an explicit policy boundary.

The policy shall specify:

```text
filesystem permissions
network access
CPU limit
memory limit
process limit
execution timeout
environment variables
working directory
privileges
```

Reference execution and replica execution should use equivalent isolation policies whenever practical.

---

# 32. Reproducibility

Every replication must be reproducible from its manifest.

A replication manifest shall contain:

```text
target
target_version
reference_hash
source_location
environment
dependencies
tool_versions
ELCI_versions
configuration
test_corpus
equivalence_policy
execution_policy
random_seed
```

If a replication cannot be reproduced, Kakashi must report:

```text
REPRODUCIBILITY: FAILED
```

rather than silently presenting the result as deterministic.

---

# 33. CLI Contract

The initial CLI should expose:

```text
kakashi init
kakashi inspect
kakashi fingerprint
kakashi characterise
kakashi observe
kakashi probe
kakashi model
kakashi replicate
kakashi validate
kakashi diff
kakashi converge
kakashi report
kakashi replay
kakashi export
kakashi status
```

Core workflow:

```bash
kakashi inspect <reference>
kakashi characterise <reference>
kakashi model <reference>
kakashi replicate <reference> --out <replica>
kakashi validate <reference> <replica>
kakashi converge <reference> <replica>
kakashi report <replication-id>
```

Every command shall support machine-readable output.

Preferred formats:

```text
human
json
jsonl
```

---

# 34. State Layout

Kakashi shall maintain an explicit state directory:

```text
.kakashi/
├── config.toml
├── manifest.json
├── fingerprints/
├── observations/
├── traces/
├── experiments/
├── corpus/
├── models/
├── evidence/
├── differences/
├── interventions/
├── reports/
├── snapshots/
└── audit.jsonl
```

Nothing inside `.kakashi/` should be treated as authoritative source code.

---

# 35. Architecture

Recommended Rust workspace:

```text
kakashi/
├── crates/
│   ├── kakashi-cli/
│   ├── kakashi-core/
│   ├── kakashi-model/
│   ├── kakashi-observe/
│   ├── kakashi-trace/
│   ├── kakashi-probe/
│   ├── kakashi-compare/
│   ├── kakashi-replicate/
│   ├── kakashi-converge/
│   ├── kakashi-evidence/
│   ├── kakashi-elci/
│   └── kakashi-sandbox/
├── fixtures/
├── schemas/
├── docs/
└── tests/
```

`kakashi-elci` shall contain adapters rather than coupling core replication logic directly to individual ELCI implementations.

---

# 36. Adapter Contract

Every ELCI integration shall implement a common adapter model:

```text
detect()
capabilities()
version()
health()
invoke()
collect()
translate_evidence()
```

An unavailable ELCI component must produce:

```text
CAPABILITY_UNAVAILABLE
```

rather than silently changing Kakashi's semantics.

Where a deterministic local fallback exists, it may be used, but the evidence must state that the fallback was used.

---

# 37. LLM Boundary

LLMs may be used for:

* hypothesis generation;
* semantic interpretation;
* source comprehension;
* implementation assistance;
* test proposal;
* explanation.

LLMs must not be the final authority for:

* equivalence;
* security;
* successful validation;
* release qualification;
* deterministic scoring;
* provenance.

The governing architecture is:

```text
LLM proposes
      ↓
Kakashi records
      ↓
Deterministic system tests
      ↓
Evidence
      ↓
Kakashi decides
```

---

# 38. Convergence Algorithm

The canonical convergence loop is:

```text
1. Load reference model.
2. Load replica model.
3. Execute differential corpus.
4. Generate differences.
5. Classify differences.
6. Estimate root causes.
7. Rank interventions.
8. Apply one bounded intervention batch.
9. Rebuild.
10. Re-run deterministic gates.
11. Re-run differential tests.
12. Measure delta.
13. Accept or rollback.
14. Persist evidence.
15. Repeat.
```

An intervention that reduces one difference while introducing greater regressions must be rejected.

---

# 39. Monotonic Progress Requirement

Every accepted iteration must satisfy at least one of:

```text
equivalence ↑
confidence ↑
coverage ↑
reproducibility ↑
known-unknowns ↓
```

and must not introduce an unacceptable regression.

The system shall maintain a convergence ledger:

```text
iteration
before_score
after_score
delta
changes
tests
regressions
decision
```

---

# 40. Rollback

Every mutation must be reversible.

Preferred rollback hierarchy:

```text
working-tree rollback
Git rollback
snapshot restore
replica reconstruction
```

PADAGONIA snapshots should be used for persistent model state where appropriate.

Kaptaind should remain the Git/release authority.

---

# 41. Reporting

Every replication must produce an executive and machine-readable report.

The report shall contain:

```text
Target
Reference fingerprint
Replica fingerprint

Equivalence:
  Behaviour
  Interface
  Configuration
  Schema
  Structure
  Operations
  Performance
  Binary

Coverage
Known differences
Unknown behaviours
Failed tests
Environmental dependencies
Security warnings
Reproducibility
ELCI components used
ELCI fallbacks used
Replication history
Final disposition
```

Possible dispositions:

```text
EQUIVALENT
CONDITIONALLY_EQUIVALENT
PARTIALLY_EQUIVALENT
NON_EQUIVALENT
INCONCLUSIVE
BLOCKED
```

---

# 42. Quality Gates

A Kakashi release must pass:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check
cargo audit
```

plus:

* deterministic replay tests;
* fixture corpus tests;
* differential comparator tests;
* serialization compatibility tests;
* adapter contract tests;
* sandbox tests;
* regression corpus tests;
* performance regression tests.

Where applicable, ELCI's own quality gates should be invoked rather than duplicated.

---

# 43. Performance

Kakashi should aggressively parallelise independent work.

Parallelisable operations include:

```text
file inspection
static parsing
fingerprinting
probe generation
reference execution
replica execution
trace comparison
fixture validation
embedding/index operations
```

PADAGONIA's parallel block encoding/decoding model should be leveraged for large evidence sets.

Tempcheq should prevent unnecessary expensive work.

Caching should prevent repeated observation of unchanged reference states.

---

# 44. Hardware Awareness

Kakashi should integrate with the ELCI hardware-aware optimisation layer, particularly Padagonia where appropriate for storage and data processing.

Hardware characteristics must be treated as evidence rather than assumptions.

The system should be capable of recording:

```text
CPU
ISA
cores
threads
cache
RAM
storage
GPU
OS
kernel
Rust target
compiler version
```

Performance equivalence must specify whether the comparison is:

```text
absolute
normalised
hardware-relative
```

---

# 45. Deterministic Seeds

All generated experiments shall use explicit seeds.

The seed becomes part of the replication manifest.

No subsystem may silently use an ambient random seed when deterministic replay is requested.

---

# 46. Human Intervention

Human intervention shall be a first-class state.

Kakashi must be able to pause at:

```text
AMBIGUOUS_REFERENCE
UNSAFE_EXECUTION
CONFLICTING_EVIDENCE
LOW_CONFIDENCE
NONDETERMINISTIC_BEHAVIOUR
IRREDUCIBLE_DIFFERENCE
INTERVENTION_REQUIRED
```

The system must preserve enough context for the human to make a decision without restarting characterisation.

---

# 47. Ethical / Legal Boundary

Kakashi is an engineering replication system, not an unrestricted reverse-engineering engine.

Its policy layer must distinguish:

```text
authorised replication
open-source reproduction
internal migration
compatibility implementation
interoperability testing
```

from targets for which execution or copying is not authorised.

The architecture should therefore support policy-controlled modes rather than assuming unrestricted extraction.

---

# 48. Definition of Done

Kakashi v1 shall not be considered complete merely because it can generate a similar project.

It is complete when it can:

1. fingerprint a reference;
2. characterise its interfaces;
3. execute controlled observations;
4. construct a persistent evidence graph;
5. generate a behavioural corpus;
6. construct a system model;
7. generate or accept a replica;
8. execute reference/replica differential tests;
9. identify structured differences;
10. rank corrective interventions;
11. apply bounded changes;
12. validate improvements;
13. rollback regressions;
14. preserve complete provenance;
15. produce a reproducible equivalence report.

---

# 49. Architectural North Star

Kakashi should ultimately behave less like:

```text
"copy this application"
```

and more like:

```text
"construct a scientifically defensible,
machine-replayable equivalence model of this system,
then produce and continuously converge a replica against it."
```

The resulting ELCI relationship is:

```text
                    REFERENCE
                       │
                       ▼
                    KAKASHI
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
       OBSERVE       MODEL       CHARACTERISE
          │            │            │
          └────────────┼────────────┘
                       ▼
                   PADAGONIA
                       │
                       ▼
                    REPLICA
                       │
                       ▼
                  DIFFERENTIAL
                    TESTING
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
          CAMBRIAN            SALIENCE
             │                   │
             └─────────┬─────────┘
                       ▼
                    REPAIR
                       │
                 FRACT/SHATTER
                       │
                       ▼
                   VALIDATE
                       │
                  TEMPCHЕQ
                       │
                       ▼
                    KAPTAIND
                       │
                       ▼
                 VERSION / GIT
                       │
                       ▼
                  NEW STATE
                       │
                       └──────────► KAKASHI
```

**Kakashi therefore becomes the ELCI system's deterministic replication loop: it turns another system into an observable object, an evidence-backed model, a reproducible implementation target, and ultimately a measurable equivalence relationship.**
