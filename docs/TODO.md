# TODO.md

## Project: Vision-Impaired Web Reader (Rust + Tauri)

---

## Phase 0: Project Setup

### Repo Setup
- [x] Initialize git repository
- [x] Create Rust project following Tauri conventions
- [x] Add Tauri scaffold
- [x] Validate Linux development baseline: `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `pnpm test:ui` pass when the documented native dependencies are installed
- [x] Setup Cargo.toml and internal modules for:
  - app_core
  - browser
  - extractor
  - dom_inspector
  - page_model
  - narration
  - tts
  - asr
  - audio_io
  - commands
  - ocr
  - config
  - state
  - logging
- [x] Define deterministic Rust tool interfaces and structured result schemas
- [x] Define common `ToolResult<T>` envelope and shared data types
- [x] Define SKILL.md discovery locations and loading rules
- [x] Define SKILL.md frontmatter schema and validation rules
- [x] Define planner input/output schema and validation rules
- [ ] Define per-tool input schemas and argument validation constraints
- [ ] Replace open-ended schema strings with closed enums where the valid set is known

### Dependencies
- [x] Add dom_smoothie
- [x] Add chromiumoxide
- [x] Add leptess (OCR)
- [x] Add cpal + rodio (audio)
- [ ] Integrate kitten_tts_rs
- [x] Add whisper backend bindings
- [x] Add OpenAI API client support for command resolution
- [x] Add Ollama API client support for command resolution
- [x] Add OpenAI API client support for optional remote TTS
- [x] Add OpenAI API client support for optional remote ASR/Whisper
- [x] Define remote planner provider selection between OpenAI and Ollama
- [x] Set default remote planner model to `gpt-5.4-mini`
- [x] Set default remote ASR model to `gpt-4o-mini-transcribe`
- [x] Set default local Whisper model size to `tiny`
- [x] Set default local KittenTTS voice to `Bruno`

### Config Module
- [x] Define final field set for planner provider selection and profiles
- [x] Define final field set for TTS provider selection and profiles
- [x] Define final field set for ASR provider selection and profiles
- [x] Define final field set for remote API settings and credential references
- [x] Define final field set for provider failover preferences
- [x] Define config schema for persistent playback volume and playback speed
- [x] Define named remote and local provider profiles
- [x] Define `SecretRef` support for env, file, and inline secret resolution
- [x] Define validation rules for category-specific profile references and provider modes
- [x] Document secure default examples using environment-variable secret references
- [x] Define exact shipped `config.example.toml` contents and first-launch defaults
- [x] Use the initial shipped profile names:
  - planner remote: `openai-default`
  - TTS remote: `openai-tts-default`
  - TTS local: `kitten-default`
  - ASR remote: `openai-transcribe-default`
  - ASR local: `whisper-default`
- [x] Define config schema for confirmation policy settings
- [x] Define config schema for OCR threshold settings
- [x] Define config schema for model-management settings
- [x] Define config schema for spoken feedback style settings
- [x] Load and validate provider config from TOML
- [x] Persist updated audio settings on change and reload them on app startup

---

## Phase 1: Core Browser + Extraction

### Deterministic Tool Core: Wave 1
- [x] Implement `open_url`
- [x] Implement `go_back`
- [x] Implement `go_forward`
- [x] Implement `reload_page`
- [x] Implement `get_page_snapshot`
- [x] Implement `extract_page_model`
- [x] Implement `list_interactive_elements`
- [x] Implement `find_element`
- [x] Implement `click_element`
- [x] Implement `scroll_page`
- [x] Implement `read_region`
- [x] Implement `read_next_region`
- [x] Implement `read_previous_region`
- [x] Implement `stop_speaking`
- [x] Implement `start_listening`
- [x] Implement `stop_listening`
- [x] Implement `transcribe_command`
- [x] Implement `set_tts_voice`
- [x] Implement `set_playback_volume`
- [x] Implement `set_playback_speed`
- [x] Implement `set_browser_visibility`
- [x] Implement `get_agent_state`
- [x] Implement `get_runtime_status`
- [x] Implement `confirm_action`
- [x] Implement `report_result`
- [ ] Finalize input schema for all Wave 1 tools
- [ ] Finalize output schema for all Wave 1 tools
- [ ] Finalize shared enums for Wave 1 tools

### Browser Module
- [ ] Launch Chromium browser (visible mode)
- [ ] Add headless toggle
- [ ] Implement open_url()
- [x] Implement go_back()
- [x] Implement go_forward()
- [x] Implement reload_page()
- [x] Track browser history state and expose can-go-back/can-go-forward signals
- [ ] Implement runtime browser visibility switching when supported
- [ ] Implement get_html()
- [ ] Implement screenshot_png()
- [ ] Implement eval_js()

### Extractor Module
- [ ] Integrate dom_smoothie
- [ ] Validate dom_smoothie output quality against target page model
- [ ] Add provisional acceptance checks for weak DOM extraction using configurable sparse-text thresholds
- [ ] Parse HTML → article
- [ ] Extract:
  - title
  - paragraphs
  - headings
- [ ] Return structured data

### Page Model
- [ ] Define structs
- [ ] Convert extractor output → PageModel
- [ ] Maintain region ordering

---

## Phase 2: Narration + TTS

### TTS Module
- [x] Wrap kitten_tts_rs
- [x] Keep local TTS as the default provider
- [x] Add optional OpenAI-backed remote TTS provider
- [x] Load model
- [x] Implement synthesize()
- [x] Add voice + speed config
- [x] Expose deterministic voice-selection updates through `set_tts_voice`
- [x] Apply persisted playback speed through kitten_tts_rs native speed control
- [x] Apply persisted playback speed through OpenAI TTS native speed control when remote TTS is active
- [x] Add TTS provider selection
- [ ] Expose KittenTTS voice choices: `Bella`, `Jasper`, `Luna`, `Bruno`, `Rosie`, `Hugo`, `Kiki`, `Leo`
- [ ] Add caching layer (optional)

### Narration Module
- [x] Implement cursor state
- [x] Read title
- [x] Read next region
- [x] Read previous region
- [x] Repeat region
- [x] Stop playback

### Audio IO
- [x] Implement playback using rodio
- [x] Handle interruption
- [x] Implement playback volume control
- [x] Expose deterministic playback-volume updates through `set_playback_volume`
- [x] Expose deterministic playback-speed updates through `set_playback_speed`
- [x] Apply persisted playback volume on startup
- [x] Apply persisted playback speed on startup through the active TTS backend

---

## Phase 3: Commands + ASR

### ASR Module
- [x] Capture audio via cpal
- [x] Implement push-to-talk
- [x] Integrate Whisper
- [x] Keep local Whisper as the default provider
- [x] Add optional OpenAI-backed remote ASR provider
- [x] Add ASR provider selection
- [x] Expose deterministic listening lifecycle tools for start, stop, and one-shot transcription
- [x] Return transcript

### Commands Module
- [x] Define command resolver interface
- [x] Define planner contract and status model
- [x] Implement OpenAI-backed command resolver
- [x] Implement Ollama-backed command resolver
- [x] Implement active LLM provider selection
- [x] Load Pi-style SKILL.md files as workflow guidance
- [x] Discover skills from project, user, and bundled locations with precedence rules
- [x] Load bundled built-in skill metadata from `docs/SKILLS.md` or generated equivalents
- [x] Validate SKILL.md frontmatter and reject invalid skills
- [x] Validate bundled skill intent tags and allowed-tool hints
- [x] Keep `IntentName` enum aligned with bundled `intent:<Name>` tags and normalized command families
- [x] Rank eligible skills by precedence, intent match, lexical overlap, and tool alignment
- [x] Load only top-ranked skills into planner context
- [x] Select deterministic tools from planner output
- [x] Return structured tool calls instead of free-form action text
- [x] Validate planner tool names and argument schemas before execution
- [x] Add canonical planner JSON examples that match the documented `IntentName` strings and tool argument field names
- [x] Add schema/fixture validation so planner example payloads stay aligned with generated JSON schema
- [x] Implement bounded step execution and replanning loop
- [x] Enforce confirmation policy for ambiguous or risky actions
- [x] Make confirmation confidence threshold configurable with default `0.90`
- [x] Allow click actions without confirmation by default, via config
- [x] Always require confirmation for submit actions
- [ ] Map phrases to intents:
  - [x] next
  - [x] previous
  - [x] repeat
  - [x] stop
  - [x] open url
  - [x] go back
  - [x] go forward
  - [x] reload page
  - [x] get current url
  - [x] read page
  - [x] start listening
  - [x] stop listening
  - [x] transcribe command
  - [x] focus field
  - [x] fill field
  - [x] type into field
  - [x] submit form
  - [x] fill and submit form
  - [x] set volume
  - [x] increase volume
  - [x] decrease volume
  - [x] mute
  - [x] get volume
  - [x] set playback speed
  - [x] increase playback speed
  - [x] decrease playback speed
  - [x] get playback speed
  - [x] set TTS voice
  - [x] toggle browser visibility
  - [x] get status
- [x] Normalize absolute volume commands from percent or normalized decimal input
- [x] Normalize relative volume commands using default, small, and large step sizes
- [x] Normalize volume query commands to `get_volume`
- [x] Normalize absolute playback speed commands from `x`, `times`, or percent input
- [x] Normalize relative playback speed commands using default, small, and large step sizes
- [x] Normalize playback speed query commands to `get_playback_speed`
- [x] Normalize browser visibility commands into explicit target visibility updates
- [x] Route current-URL and runtime-status queries through `get_agent_state` and `get_runtime_status`
- [x] Normalize relative audio-setting phrases to `SetPlaybackVolume` and `SetPlaybackSpeed` planner intents
- [x] Normalize status/history/listening phrases to their dedicated planner intent variants before tool selection
- [x] Normalize form-filling phrases to `FillInput` and form-submission phrases to `SubmitForm`
- [x] Add normalization examples for ambiguous utterances, mixed commands, and follow-up corrections
- [x] Handle fuzzy matching

### App Integration
- [x] Route ASR → command → action
- [x] Display transcript in UI

---

## Phase 4: DOM + OCR

### Deterministic Tool Core: Wave 2
- [x] Implement `focus_element`
- [x] Implement `type_into_element`
- [x] Implement `submit_active_form`
- [x] Implement `capture_screenshot`
- [x] Implement `run_ocr`
- [x] Implement `merge_ocr_into_page_model`
- [x] Finalize input schema for all Wave 2 tools
- [x] Finalize output schema for all Wave 2 tools
- [x] Finalize shared enums for Wave 2 tools

### DOM Inspector
- [x] Map regions to bounding boxes
- [x] Extract links
- [x] Attach geometry to PageModel

### OCR Module
- [ ] Integrate leptess
- [x] Crop screenshot regions
- [ ] Run OCR on region
- [ ] Merge OCR into PageModel
- [ ] Trigger OCR when no extractable text is found
- [ ] Make sparse-text OCR thresholds configurable
- [ ] Default sparse-text OCR thresholds to `200` characters or fewer than `2` readable regions
- [ ] Prefer region OCR before broader OCR when possible

---

## Phase 5: UI (Tauri)

### Basic UI
- [ ] URL input
- [ ] Open button
- [ ] Read button
- [ ] Stop button
- [ ] Next / Previous buttons
- [x] Push-to-talk button
- [x] Add nearby playback volume control
- [x] Add nearby playback speed control
- [ ] Ensure normal operation is fully voice-controlled

### Settings UI
- [ ] TTS model selection
- [ ] Voice selection
- [ ] Speed control
- [ ] Volume control
- [x] Visible/headless toggle
- [ ] Planner provider selection
- [ ] TTS provider selection
- [ ] ASR provider selection
- [ ] Remote API configuration inputs or references
- [ ] Local model configuration inputs or references
- [ ] Provider failover toggle where supported
- [ ] Secret entry UX that stores references or masked secrets safely
- [ ] Add settings for confirmation threshold and click-without-confirmation behavior
- [ ] Add settings for OCR thresholds
- [ ] Add model management controls and manual download button
- [ ] Provide an easy path to config controls from missing-model warnings/errors

### Voice Settings Control
- [x] Add voice commands for playback volume adjustment
- [x] Add voice commands for playback speed adjustment
- [x] Add voice commands for querying current playback volume
- [x] Add voice commands for querying current playback speed
- [x] Clamp voice-driven playback speed changes to configured limits
- [x] Clamp voice-driven volume changes to configured limits
- [x] Persist normalized voice-driven volume changes immediately
- [x] Persist normalized voice-driven playback speed changes immediately
- [x] Speak current playback volume on query
- [x] Speak current playback speed on query

### Status UI
- [x] Current page title
- [x] Current region
- [x] Listening indicator
- [x] Speaking indicator
- [x] Browser visibility indicator
- [x] Back/forward availability indicator

---

## Phase 6: State + Integration

### State Module
- [ ] Central app state struct
- [ ] Track:
  - current page
  - browser visibility
  - browser history state
  - narration index
  - audio state
  - ASR state
  - listening state
  - playback volume
  - playback speed
  - active TTS voice
  - pending confirmation state
- [ ] Re-read effective speech settings before each new utterance
- [ ] Apply changed speech settings on the next utterance, not mid-utterance

### Event Flow
- [ ] UI → app_core → modules
- [ ] Commands → narration
- [ ] Navigation → extractor → model

---

## Phase 7: Logging + Error Handling

### Logging
- [ ] Add structured logging
- [ ] Log:
  - commands
  - extraction results
  - OCR fallback usage
  - errors

### Error Handling
- [ ] Graceful failure on:
  - browser issues
  - TTS errors
  - ASR errors
  - OCR errors
  - LLM provider errors
  - remote TTS provider errors
  - remote ASR provider errors

---

## Phase 8: Testing

### Unit Tests
- [ ] Command parsing
- [ ] LLM provider selection behavior
- [ ] Default local model profile selection behavior
- [ ] TTS provider selection behavior
- [ ] ASR provider selection behavior
- [ ] Deterministic tool result schemas
- [ ] Common tool envelope serialization/deserialization
- [ ] Planner input/output schema serialization/deserialization
- [ ] Per-tool input schema validation
- [ ] Enum serialization/deserialization and validation
- [ ] Provider config serialization/deserialization and validation
- [ ] Secret reference resolution and masking behavior
- [x] Audio settings persistence and validation
- [ ] Browser history state serialization and boundary behavior
- [ ] Runtime status schema serialization and provider-mode reporting
- [ ] Deterministic listening state transitions and one-shot transcription tool behavior
- [ ] Deterministic browser visibility and audio-setting tool clamping behavior
- [x] Voice command parsing for playback volume and playback speed
- [x] Volume normalization from percent, decimal, and relative phrases
- [x] Playback speed normalization from multiplier, percent, and relative phrases
- [x] Volume and playback speed query command normalization and spoken response formatting
- [ ] SKILL.md frontmatter validation and precedence resolution
- [ ] Skill ranking and top-N selection behavior
- [ ] Reject unknown tools and invalid planner transitions
- [ ] Reject invalid tool arguments before execution
- [ ] Element matching and resolution behavior
- [x] Pending plan execution state serialization and resume bookkeeping
- [x] ExecutionOutcome mapping from PlannerStatus and step transitions
- [ ] Page model building
- [ ] Navigation logic

### Integration Tests
- [ ] Load page → extract → read
- [ ] ASR → command → action
- [x] Planner output → deterministic tool execution
- [ ] Back/forward/reload tools update browser history state correctly
- [ ] Browser visibility changes are reflected in runtime status and UI state
- [ ] Listening start/stop/transcribe tools update runtime state correctly
- [x] Deterministic audio-setting tools persist and report the updated values
- [ ] Planner requests confirmation before risky execution
- [x] Queued confirmation flows resume at the stored follow-up step after explicit user approval
- [x] Rejected or timed-out confirmation flows clear pending state and replan without executing the queued side-effecting step
- [ ] Submit actions always require confirmation
- [ ] Click actions may proceed without confirmation when configured
- [ ] Fill-field workflows resolve the intended input and write the requested value
- [ ] Fill-and-submit workflows require confirmation before form submission
- [ ] Ambiguous element matches ask the user to clarify instead of silently choosing one
- [ ] Mixed commands such as fill-and-submit are decomposed into safe bounded plans
- [ ] Follow-up corrections such as `no, the other field` reuse recent context when available
- [ ] Replanning after tool failure or ambiguous result
- [ ] LLM unavailable with no local provider → report command interpretation unavailable
- [ ] Remote TTS selected → speech output succeeds
- [x] Remote ASR selected → transcript is returned
- [x] Playback volume and speed changes persist across app restart
- [x] Voice command changes to playback volume and speed persist across app restart
- [ ] Changed speech settings apply on the next utterance only

### Agentic Tests
- [ ] Add planner-skill regression fixtures with browser state, transcript, expected selected skills, and expected tool sequence
- [ ] Assert that the correct bundled skills were selected for representative tasks
- [ ] Add fixtures for ambiguous clicks, form filling, fill-and-submit, and follow-up corrections
- [ ] Build a growing corpus of in-the-wild problematic pages for agentic regression coverage

---

## Phase 9: v2 Notes (DO NOT IMPLEMENT)

### Wake Word
- [ ] Evaluate TensorFlow Lite micro_speech

### LLM Action Resolver
- [ ] Candidate extraction system
- [ ] LLM ranking
- [ ] Confidence gating
- [ ] Evaluate broader open-ended UI grounding beyond the deterministic v1 tool layer

### Deferred Exploration
- [ ] Evaluate whether advanced UI action grounding should remain v2-only

---

## Deliverables

- [ ] Working desktop app
- [ ] README.md
- [ ] SPECS.md
- [ ] TODO.md
- [ ] Example configs
