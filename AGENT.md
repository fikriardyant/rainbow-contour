# AGENT.md - Rainbow Contour Project

Dedicated development workhorse for **Rainbow Contour** (Auto Cut & Fill Difference Map Generator).

---

## Workspace & Project Context

- **Project Root**: `/home/cells/Documents/Antigravity Project/Rainbow-Contour`
- **Repository**: `git@github.com:fikriardyant/rainbow-contour.git` (Private)
- **Primary Tech Stack**: 
  - Rust Native Engine (`rayon`, `ezdxf`/DXF parser, matrix interpolation)
  - Python / Standalone CLI Wrappers (`.sh` / `.bat`)
  - Standalone HTML Canvas Visualizer (Neobrutalism UI, PDF & DXF isoline exports)

---

## Workflow Rules & Guidelines

### 1. Codebase Search Policy (Mandatory Graphify Usage)
- **Graphify First**: For any structural search, architecture tracing, file relationship checking, or answering "how does X work" within this codebase:
  - Check if `graphify-out/graph.json` exists in project root. If it exists, execute `graphify query "<question>"` via terminal.
  - If no index exists yet, run `graphify .` (or follow the `graphify` skill) to generate the knowledge graph before manual recursive grep.
  - Manual file reading (`read_file` / `search_files`) is reserved ONLY for non-code assets or when graphify is unsuited.

### 2. Development Workflow (Superpowers Skill Suite)
- **Spec-First & Plan-First**:
  - Technical Specifications live in `docs/superpowers/specs/`.
  - Implementation Plans live in `docs/superpowers/plans/`.
- **TDD (Test-Driven Development)**:
  - Write failing test first $\rightarrow$ verify failure $\rightarrow$ implement minimum code $\rightarrow$ verify pass.
  - Green-gate verification (tests, linting, build checks) is strictly required before marking tasks complete.

### 3. Coding Discipline (Ponytail Mode)
- **Lazy Senior Dev Mode**:
  - YAGNI (You Aren't Gonna Need It) & stdlib/native first.
  - Minimal diffs, no unnecessary dependencies or abstractions.
  - Mark deliberate simplifications or performance ceilings with a `ponytail:` comment naming the trade-off.

### 4. Decision & Assumption Protocol ("Iya. Tapi..." Discipline)
- **Validation Mandate**:
  - When given a clear instruction, adopt an immediate execution posture ("Iya").
  - Validate edge cases, hardware constraints, or technical gaps ("Tapi...").
  - If making domain-sensible engineering assumptions, **always present the assumption explicitly to Fikri for validation**.

### 5. GitHub Release & Versioning Rules
- **Release Title Format**: Every release title must follow: `vX.Y.Z (Short Description)` (e.g., `v1.1.1 (Configurable Ongrade Range, High-Contrast Palette & Clean Uniform Legend)`).
- **Changelog Included**: The release body on GitHub must contain the detailed changelog sections (`Added`, `Changed`, `Fixed`) plus direct links to the multi-OS release binaries (Windows `.zip`, Linux `.tar.gz`, macOS ARM64 & Intel `.tar.gz`).
- **Tag Discipline**: Do not push or bump git tags unless explicitly commanded by Fikri.
- **Gitignore Safety**: Keep build artifacts, output directories (`output*/`, `sample_demo_output/`), and large survey attachments strictly gitignored.
