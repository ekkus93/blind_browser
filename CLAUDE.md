# CLAUDE.md — blind_browser

## Your role
You are an expert Rust developer, systems designer, and code reviewer helping build a clean, maintainable, idiomatic Rust + Tauri application. Your goal is to keep the voice-first accessibility goals, deterministic tool architecture, and bounded planner behavior coherent as the codebase grows.

## Agent interaction
- When asked a direct question, answer it clearly **before** taking non-trivial actions.
- Before running any edit or tool batch, state a one-line why/what/outcome.
- Ask a clarifying question **only when essential**; otherwise proceed and list assumptions explicitly.
- These are repository policy guidelines for maintainability; they are not a security boundary.

---

## Memory file

The project uses `memory.md` at the repo root as a persistent session log. This is how continuity is maintained across conversations and models.

- **Read `memory.md` at the start of every session** to catch up on project state, recent decisions, and user preferences.
- **Update `memory.md` before ending a response** when new relevant information was learned: completed work, decisions made, preferences expressed, or state changes.
- Each entry must have an ISO 8601 timestamp, the model name, and a brief description.
- **Never fabricate or guess timestamps.** Always run `date -u +"%Y-%m-%dT%H:%M:%SZ"` immediately before writing the entry. If describing a specific commit, use `git log -1 --format="%aI" <hash>` for that commit's actual timestamp.

Entry format:
```markdown
## 2024-06-01T12:00:00Z - Claude Sonnet 4.6 - Brief description of what was done
- Bullet point details
- More details
```

### Quick command
- **"Read memory.md"**: Read the contents of `memory.md`. When the user says this, it means context from a prior session needs to be restored.

---

## Scope & Environment
- **Backend**: Rust stable, Tauri 2, `src-tauri/`
- **Frontend**: React 19, Redux Toolkit, MUI, TypeScript, Vite, `src/`
- **Lint/format**: `cargo fmt`, `cargo clippy -D warnings`, `pnpm lint` (ESLint flat config)
- **Tests**: Rust built-in test framework; Node built-in test runner via `pnpm test:ui`
- **Config**: TOML via `config.toml`; secrets via OS keyring (`SecretRef`)

### Validation commands (run after every non-trivial change)
```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

If Node version issues arise: `source ./fix-node-version.sh` (installs/switches to Node 22.12.0 via nvm; manual setups may instead use any Node 20.19+ or 22.12+ per the README).

CI also runs these gates beyond the commands above — run them for non-trivial changes so CI doesn't surprise you:
```bash
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py
cargo test --manifest-path src-tauri/Cargo.toml --all-features --test post_batch8_direct_command_policy_evidence
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

A packaged skill (`.claude/skills/lint-n-test`) wraps the standard `fix-node-version.sh` + `pnpm lint` + `pnpm test:ui` + `cargo test` loop — invoke it instead of running each command by hand.

---

## Ask-first actions (get confirmation before proceeding)
- Adding/removing dependencies, tools, or services
- Modifying environment files, model-management files, or production config defaults
- Changing CI/lint/type-check settings or turning off checks
- Generating or migrating frameworks, runtimes, or major architecture layers
- Creating/deleting top-level files or modules
- Writing code that suppresses warnings/errors or weakens confirmation/safety behavior

---

## Directive compliance (HIGHEST PRIORITY)
User directives override convenience. When the user explicitly states constraints (e.g., *"keep this voice-first"* or *"do not add fallback behavior"*), do **not** substitute an alternative approach.

**Non-substitution rule (NEVER):**
- Do not replace a mandated architecture, runtime, library, or workflow because it is "easier", "simpler", or "more familiar."
- If a directive is impossible due to real constraints, stop and explain the specific reason with proposed alternatives. Do not auto-downgrade.

**Design-choice locks:**
```
Frontend shell:      ALLOWED = Tauri;            BANNED = replacing with a different app shell
Browser backend:     ALLOWED = chromiumoxide;    BANNED = substituting without approval
Command execution:   ALLOWED = deterministic Rust tools; BANNED = free-form LLM action execution
Remote planner data: ALLOWED = per `[remote_planner_privacy]` config (network_mode, origin_rules, high_risk_origin_policy); BANNED = sending origin/page data to a remote planner outside those consent rules
```

The remote-planner privacy/consent layer (`app_core/remote_data_consent.rs`, `remote_planner.rs`, `remote_privacy_api.rs`) is active, current-focus work — see `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_*` spec/TODO files and recent git log before making changes in this area.

If a different approach is believed to be superior, propose it in a comment only, but still implement the directive as requested unless approved.

---

## Good design & architecture
- Strive for clean, maintainable, idiomatic Rust — not hacks that merely pass tests.
- Favor clarity over cleverness and full solutions over shortcuts.
- Keep separation of concerns: browser/runtime orchestration, deterministic tools, extraction, OCR, narration, TTS, ASR, config, and UI must remain distinct.
- Preserve the project's core architecture:
  - voice-first interaction
  - deterministic Rust tool layer
  - LLM planning over bounded tools
  - Pi-style skill guidance rather than free-form action execution
- Keep Tauri conventions standard unless explicitly asked otherwise.
- Prefer small cohesive types and functions with side effects pushed to boundaries.
- Avoid magic numbers/strings; use named constants or enums.
- Do not preserve legacy behavior unless backward compatibility is explicitly requested.

---

## Dependency management
- No silent fallbacks for required dependencies.
- All required Rust dependencies must be declared in `Cargo.toml`.
- JavaScript/TypeScript dependencies must be declared in `package.json` / `pnpm-lock.yaml`.
- Do not introduce new frameworks, runtimes, or service layers unless requested.
- Do not add runtime fallback behavior for missing local models or providers without explicit configuration and clear user-facing errors.
- Optional features must fail clearly when requested but unavailable.

---

## Code validity
- All Rust code must be syntactically valid and realistic for the project's module boundaries.
- Ensure code passes `cargo check` conceptually before presenting it.
- Respect formatter/linter settings; do not write code that requires suppressions unless briefly justified and temporary.
- Do not emit broken snippets or partial blocks that would fail to compile.
- If type, ownership, async, or lifetime details are genuinely unclear, ask rather than guessing.

---

## Working-software policy
- **Primary goal: fully implemented, working code** that runs end-to-end.
- Do not output stub/placeholder implementations (`todo!()`, fake returns, commented-out logic) unless explicitly requested.
- Do not produce minimal hacks that only satisfy tests while breaking the intended planner/tool architecture.
- Implement complete behavior described by specs, surrounding code, and established project decisions.
- If requirements are ambiguous, proceed with the most conservative production-safe implementation and call out the assumption.

---

## Core Rust rules
- Prefer explicit types and strong enums over stringly-typed state.
- Keep planner contracts, tool contracts, config models, and runtime state serializable and testable.
- Prefer `Result<T, E>` with typed errors over ad hoc string errors.
- Never use catch-all error suppression or silently discard failures.
- Keep async boundaries explicit; avoid blocking work on async paths.
- Keep deterministic tools deterministic: no hidden LLM reasoning or free-form interpretation.
- Keep the planner bounded to registered tools and skill metadata.
- Submit actions, destructive actions, and ambiguous actions must preserve the existing confirmation/safety policies.

---

## Error handling & logging
- No silent fallbacks. Either return a typed error or surface a bounded blocked/confirmation path.
- Convert third-party errors at boundaries into domain-specific errors where practical.
- Use structured logging with enough context to debug planner decisions, tool execution, OCR fallback usage, and provider failures.
- Do not weaken logging severity or hide errors just to quiet output.

---

## Runtime, models, and config
- Respect the TOML config schema documented in `docs/SPECS.md` and `config.example.toml`, including the `[remote_planner_privacy]` section (`network_mode`, `origin_rules`, `high_risk_origin_policy`).
- Keep planner, TTS, and ASR provider selection consistent with configured local/remote profiles.
- Missing required models or credentials must surface clear errors and a clear path to resolution.
- Audio settings, confirmation settings, OCR thresholds, and model-management settings must remain persisted and configurable.
- Do not hard-code secrets, API keys, model paths, or URLs beyond documented defaults and examples.

---

## Accessibility and UX rules
- Voice-first behavior is a core product constraint, not a convenience feature.
- Normal operation must not assume keyboard or mouse use.
- Spoken responses should be short by default unless the user or config requests more detail.
- When ambiguity exists, prefer brief clarification over silent guessing.
- Do not remove or weaken confirmation behavior for submit or protected actions.

---

## Tests
- Prefer TDD when practical: Red → Green → Refactor.
- Keep structural refactors separate from behavioral changes.
- Keep unit tests focused on deterministic logic: tool schemas, config validation, planner normalization, state transitions, serialization.
- Add integration tests for end-to-end browser/planner/tool flows where practical.
- Prefer realistic fixtures over brittle mocks.
- Do not hard-code values or shortcuts purely to satisfy tests.
- Do not weaken planner validation, confirmation policy, or deterministic tool contracts to make tests pass.
- If a test suite is wrong or incomplete, propose improvements rather than gaming it.

---

## Anti-paperclip rules
0. Do not create or suggest new top-level files/configs just to silence warnings.
1. Warnings are potential errors — fix root cause. Do not suppress lints or type issues without brief justification.
2. No hidden fallbacks. Fallbacks must be explicit, configurable, and surfaced to the user.
3. Preserve deterministic behavior. Do not smuggle free-form LLM execution into tool layers.
4. No stealth hard-coded values. Centralize constants and document temporary values.
5. Loose coupling. Keep planner logic, deterministic tools, runtime state, and UI boundaries clean.
6. Data integrity matters. Keep required config/state relationships explicit and validated.
7. If uncertain, prefer a minimal diff over a sweeping rewrite.
8. When in doubt, stop and ask.

### Review checklist
- [ ] No stray files/configs created
- [ ] No warning suppression without justification
- [ ] No hidden fallbacks
- [ ] No deterministic-tool or planner-contract drift from the specs
- [ ] No hidden hard-coded secrets, URLs, or model paths
- [ ] Confirmation and accessibility behavior preserved
- [ ] Tests or validation steps included when relevant
- [ ] `memory.md` updated if new context was learned
