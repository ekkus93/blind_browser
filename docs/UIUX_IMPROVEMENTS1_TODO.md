# UI/UX Improvements TODO — blind_browser

Based on the UI/UX review performed in June 2026.  
**Status key:** PENDING · IN PROGRESS · DONE

---

## Goal

Address the bugs, usability gaps, and accessibility issues identified in the June 2026 UI/UX review.  
The improvements are grouped so each phase can be validated and committed independently.

---

## Implementation strategy

Work in this order:

1. Fix functional bugs and critical usability gaps that block users right now.
2. Build the first-run / unconfigured state so new users can orient themselves.
3. Do the plain-language copy pass — many issues trace back to developer jargon.
4. Improve settings navigation and discoverability.
5. Add progress and feedback for long-running operations.
6. Accessibility polish pass.
7. Full validation and final audit.

---

## Phase 1 — Critical functional fixes

These are bugs or gaps that actively break or mislead users. Fix these first.

### 1.1 Show PTT errors visually, not just to screen readers — DONE

**Problem:** When ASR fails (missing API key, model not found, etc.) the error text is marked `sr-only`. Sighted users see a disabled button with no explanation. This is a functional bug for the majority of desktop users.

- [x] In `src/confirmation-panels/push-to-talk.ts`, locate the `sr-only` error alert (around lines 129–131).
- [x] Add a second visible error element below the push-to-talk button that renders when `state.pushToTalkError` is populated. Use a small `role="alert"` paragraph with the same error text.
- [x] Style the visible error in `src/styles.css` to match the existing inline error pattern used in other panels (e.g., the `.url-input-error` or settings panel error styles).
- [x] Keep the existing `sr-only` element as well — screen readers should still get the announcement. The visible element serves sighted users.
- [x] Update `src/confirmation-panel.test.mjs` or the push-to-talk test file to assert that the error is visible (not only sr-only) when `pushToTalkError` is set.
- [x] Validate: a user who cannot hear TTS must be able to read why the Talk button is disabled.

### 1.2 Add confirmation before "Reset to defaults" in planner settings — DONE

**Problem:** Clicking "Reset to defaults" in the planner settings panel fires immediately with a single click, wiping the user's endpoint, model, and related configuration with no undo.

- [x] In `src/settings-panels/planner.ts`, locate the reset button and its `onClick` handler.
- [x] Before calling the reset Tauri command, show an inline confirmation step. Options:
  - **Option A (recommended):** Replace the reset button with a two-step sequence: click once to reveal a confirmation row ("Reset all planner settings to defaults? This cannot be undone."), then a "Yes, reset" button and a "Cancel" button.
  - **Option B:** Use `window.confirm(...)` as a temporary guard while a better inline solution is built. Mark this with a TODO noting it should be replaced.
- [x] Ensure the confirmation step is keyboard accessible and does not trap focus.
- [x] Ensure the Cancel path restores the UI to exactly the state it was in before the reset button was clicked.
- [x] Update or add a render test covering the two-step reset flow.

### 1.3 Remove the "Provider Failover" section from Runtime settings — DONE

**Problem:** The entire `settings-provider-failover` panel renders disabled checkboxes with the copy "Automatic failover is not available yet." This looks like a misconfiguration rather than an intentional placeholder. It clutters the Runtime settings page and creates false expectation.

- [x] In `src/app-shell.ts`, remove the `renderPanelContent("settings-provider-failover", panelContent)` call from the runtime settings view section.
- [x] In `src/settings-status-panels.ts` (the barrel) and `src/settings-panels/runtime.ts`, the failover panel render function can remain in source but should not be wired into the shell until the feature is implemented.
- [x] Update `src/app-shell.test.mjs` — remove any assertions that expect the failover panel to appear in the runtime view.
- [x] Add a comment in the render function (`renderProviderFailoverNode` or equivalent) noting it should be re-wired once the backend feature ships.

### 1.4 Preserve `settingsView` state when switching workspace ↔ settings — DONE

**Problem:** The Redux store resets `settingsView` to `"overview"` every time the user switches to the settings view. A user who navigates to Settings → ASR, switches to workspace to check something, then returns to settings is dropped back at the overview and must re-navigate.

- [x] In `src/app-shell-store.ts`, find the `setAppView` reducer (or equivalent action that resets `settingsView`).
- [x] Remove the automatic reset of `settingsView` to `"overview"` when switching to settings. The view should only reset when the user explicitly navigates to the overview (e.g., presses the back button from a subpage).
- [x] Verify the back-button flow still works: navigating from a subpage back to the overview should still set `settingsView` to `"overview"`.
- [x] Verify the workspace → settings transition still sets focus correctly.
- [x] Update `src/app-shell.test.mjs` or `src/dom-seams.test.mjs` to assert that settings subpage state is preserved across a workspace roundtrip.

---

## Phase 2 — First-run and unconfigured state

New users currently open the app to a disabled Talk button and no explanation. This phase adds orientation for users who haven't configured TTS, ASR, or the planner yet.

### 2.1 Add a setup-required banner on the workspace ✅ DONE

**Problem:** When ASR or TTS is not configured, the Talk button is disabled. There is no visible guidance pointing the user toward settings.

- [x] In `src/settings-panels/workspace.ts` (or wherever the push-to-talk area is assembled), detect when `state.pushToTalkError` indicates a setup-related failure (model unavailable, API key missing, provider disabled).
- [x] When that condition is true, render a visible banner or inline card above or below the push-to-talk button with short copy such as: **"Voice input isn't set up yet. Open settings to configure your microphone and speech providers."**
- [x] Include a button in that banner that triggers navigation to the settings view (use the same `onAppViewSelect("settings")` handler already available).
- [x] The banner should disappear as soon as the error condition clears (i.e., ASR becomes available).
- [x] Ensure the banner text is announced by screen readers (`role="status"` or `aria-live="polite"`).
- [x] Add a render test that asserts the banner appears when push-to-talk is in a setup-error state and disappears when it is not.

### 2.2 Add status indicators to the settings overview cards ✅ DONE

**Problem:** The four settings cards ("Open planner setup", "Open TTS setup", etc.) look identical regardless of whether the section is configured and working or broken and unconfigured. A user cannot tell which area needs attention without entering each one.

- [x] Define a status indicator type: `"ok" | "warning" | "error" | "unconfigured"`.
- [x] In `src/main.ts` or `src/runtime-refresh.ts`, derive a per-section status from the live runtime state:
  - **Planner**: `"ok"` if endpoint + model are set and API key is present; `"warning"` if key is missing; `"error"` on connectivity failure.
  - **TTS**: `"ok"` if local model is downloaded or remote key is present; `"warning"` if model missing; `"error"` on failure.
  - **ASR**: same pattern as TTS.
  - **Runtime**: `"ok"` if model management reports no missing downloads; `"warning"` if any model is missing.
- [x] Pass these statuses as part of the app shell panel content or navigation handler props so the overview can render them without coupling to runtime state directly.
- [x] In `src/app-shell.ts`, update `renderSettingsSubpageLink` to accept an optional status and render a small indicator beside the chevron (e.g., a colored dot or a short text label like "Action needed").
- [x] In `src/styles.css`, add status dot styles for the overview cards.
- [x] Ensure the status indicators have text alternatives (not color only) — e.g., `aria-label="TTS setup — Action needed"`.
- [x] Add render tests covering each status variant for at least one overview card.

---

## Phase 3 — Plain-language copy pass

Many usability problems in this app trace directly to developer-facing terminology being exposed as user-facing copy. This phase replaces jargon with plain language throughout.

### 3.1 Replace technical abbreviations in settings subpage headings and hero copy ✅ DONE

**Problem:** The settings subpages use abbreviations ("TTS setup", "ASR setup") and developer terms ("Planner") that are not accessible to non-technical users.

- [x] In `src/app-shell.ts`, update the four subpage `<h2>` headings:
  - "Planner setup" → "AI assistant setup"
  - "TTS setup" → "Voice output setup"
  - "ASR setup" → "Voice input setup"
  - "Runtime setup" → "Advanced settings"
- [x] In `src/app-shell.ts`, update the settings overview section headings and eyebrows to match the new subpage names:
  - Eyebrow "Command interpretation" / h2 "Planner" → eyebrow "Command interpretation" / h2 "AI assistant"
  - Eyebrow "Speech output" / h2 "Text to speech" → eyebrow "Speech output" / h2 "Voice output"
  - Eyebrow "Speech input" / h2 "Automatic speech recognition" → eyebrow "Speech input" / h2 "Voice input"
  - Eyebrow "Runtime behavior" / h2 "Runtime" → eyebrow "Advanced" / h2 "Advanced settings"
- [x] Update the subpage link labels to match.
- [x] Update `src/confirmation-panel.test.mjs` to match the new headings and link labels.

### 3.2 Replace abbreviations inside settings panel content ✅ DONE

**Problem:** The individual settings panels still use "TTS", "ASR", and "Planner" in headings and labels.

- [x] In `src/settings-panels/tts.ts`, replaced "TTS provider" → "Voice output provider", "TTS model" → "Voice model", descriptions updated.
- [x] In `src/settings-panels/asr.ts`, replaced "ASR provider" → "Voice input provider", profiles renamed.
- [x] In `src/settings-panels/planner.ts`, replaced "Planner setup" → "AI assistant setup".
- [x] In `src/settings-panels/runtime.ts`: "OCR fallback" → "Screen reading fallback", "Confirmation" → "Action confirmation".
- [x] Updated `src/confirmation-panel.test.mjs` to match new copy.

### 3.3 Format timeouts as seconds instead of milliseconds ✅ DONE

**Problem:** Remote profile detail cards in TTS and ASR settings display `timeoutMs: 30000`. Users do not know whether this is good, bad, or what unit it is in.

- [x] In `src/settings-panels/tts.ts`, replaced raw ms with formatted seconds ("Timeout" showing "30 seconds").
- [x] In `src/settings-panels/asr.ts`, applied the same fix.

### 3.4 Remove or translate technical fields from profile detail cards ✅ DONE

**Problem:** ASR and TTS profile detail cards surface internal implementation fields — `threads`, `temperatureMilli`, raw model path tilde notation — that have no actionable meaning to users.

- [x] In `src/settings-panels/asr.ts`, removed the `threads` row from the local ASR profile detail card.
- [x] In `src/settings-panels/asr.ts`, converted `temperatureMilli` → "Creativity" (divided by 1000, toFixed(2)).

### 3.5 Replace "Region" with "Section" throughout the status panel ✅ DONE

**Problem:** The status panel refers to "Current region" and "Region N of M". "Region" is an internal page-model term. Users of assistive technology will understand "section" better.

- [x] In `src/settings-panels/workspace.ts`, replaced all user-facing "region" strings with "section" (status panel label, URL input button aria-labels).
- [x] Updated render tests that asserted on region-related copy.

### 3.6 Replace "Keyring" with user-friendly language in API key descriptions ✅ DONE

**Problem:** The API key cards say API keys are stored in the "OS keyring". Most users don't know what a keyring is.

- [x] In `src/settings-panels/tts.ts`, replaced "OS keyring" → "securely on your device".
- [x] In `src/settings-panels/asr.ts`, applied the same fix.

### 3.7 Replace "Requires planner change" badge with actionable copy ✅ DONE

**Problem:** When a non-retryable backend error occurs in the confirmation panel, a badge says "Requires planner change". This is meaningless to users and tells them nothing about what to do.

- [x] In `src/confirmation-panels/confirmation.ts`, replaced with "Cannot be retried — open Settings to check your AI assistant configuration."
- [x] Updated the tests in `src/confirmation-panel.test.mjs` to assert on the new text.

---

## Phase 4 — Settings UX improvements

### ✅ DONE 4.1 Style the "Load models for this endpoint" planner option as non-selectable

**Problem:** In the planner settings model dropdown, the placeholder text "Load models for this endpoint" appears as a regular `<option>`. Users may try to select it as if it were a real model, and nothing happens.

- [x] In `src/settings-panels/planner.ts`, find where the placeholder option is rendered in the model selector.
- [x] Change the placeholder `<option>` to be `disabled` and have an empty value, so it cannot be selected. Example: `<option value="" disabled>Load models for this endpoint</option>`.
- [x] Consider also using a `selected` attribute on the placeholder when no model is loaded, so it shows as the current (non-selectable) state.
- [x] Alternatively, if the UX is "no dropdown until models are loaded", hide the dropdown entirely until `availableModels` is non-empty, and replace it with a button row that says "Click 'Load models' to see available options".
- [x] Update any render tests that assert on this dropdown state.

### ✅ DONE 4.2 Narrow the playback speed slider range

**Problem:** The speed slider goes from 0.5× to 5×. At 5×, speech is incomprehensible for most users. The extreme end compresses the useful range into a small zone of the slider track.

- [x] In `src/settings-panels/playback.ts`, find the speed slider `<input type="range">`.
- [x] Change the `max` attribute from `5` to `2.5` (or `3` at most). The range 0.5×–2.5× covers all realistic use cases for TTS narration.
- [x] If the current persisted value for any user is above the new max, clamp it to the new max on load. This should be handled in the render path: `Math.min(value, 2.5)`.
- [x] Update `src/styles.css` if any slider track calculations depend on the range.
- [x] Update render tests that assert on slider `max` or value display.

### ✅ DONE 4.3 Remove or visually distinguish read-only profile detail cards

**Problem:** The local/remote profile detail cards in TTS and ASR settings look similar to editable form fields. Users may try to click or edit them, not realizing they are read-only reference information.

- [x] In `src/styles.css`, add a distinct visual style for `.settings-panel-details` or equivalent read-only data grid elements. Use a muted background, no border focus ring, and a subtly different font weight compared to editable inputs.
- [x] In `src/settings-panels/tts.ts` and `src/settings-panels/asr.ts`, ensure the read-only detail cards use a `<dl>` or `<table>` element (not `<form>` or `<input>`) so their semantics are clearly informational rather than interactive.
- [x] Add a small "Read-only" or "Your current configuration" label at the top of each detail card to set expectations.

### ✅ DONE 4.4 ASR settings: add a visual indication when Local model is missing

**Problem:** The ASR local profile detail card shows model name and path, but gives no clear signal if the model hasn't been downloaded yet. Users may not realize they need to visit the Runtime > Model management section to download it.

- [x] In `src/settings-panels/asr.ts`, check whether the local ASR model state includes a `downloaded` or `available` flag (it should, from model management state).
- [x] If the model is not downloaded, show an inline warning with text like: "Model not downloaded yet. Go to Advanced settings → Model management to download it." Include a button or link that navigates to the runtime settings view.
- [x] Update relevant render tests.

### ✅ DONE 4.5 TTS settings: same model-missing warning as 4.4

- [x] Apply the same pattern as task 4.4 to `src/settings-panels/tts.ts` for the local TTS model detail card.

---

## Phase 5 — Progress and feedback for long-running operations

Users currently have no feedback during downloads, model loading, or API testing. This phase adds visual progress signals.

### ✅ DONE 5.1 Add a spinner to the API key "Test" button

**Problem:** When a user clicks "Test API key", the button changes its label to "Testing..." but there is no spinner or progress indicator. Users don't know if the test is running or frozen, especially since it involves a network call.

- [x] In `src/confirmation-panel-helpers.ts`, find `renderSecretEntryCard` and the test button rendering logic.
- [x] When `isTesting` is true, add a CSS spinner inside or alongside the "Testing..." button label. A simple CSS `@keyframes` spin on a small `<span>` element is sufficient — no new dependencies needed.
- [x] In `src/styles.css`, add the spinner animation and the `.btn-spinner` class.
- [x] Ensure the spinner respects `prefers-reduced-motion` — when reduced motion is preferred, show only the text state change without animation.
- [x] Update render tests that assert on the test button state.

### ✅ DONE 5.2 Add a spinner to the planner "Load models" button

**Problem:** When loading models from an endpoint, the button changes to "Loading models..." but no visual progress signal is shown.

- [x] In `src/settings-panels/planner.ts`, find the load-models button and its `isLoadingModels` state.
- [x] Apply the same spinner pattern as task 5.1 to the "Loading models..." state.
- [x] Ensure the spinner uses the same CSS animation class introduced in 5.1 for consistency.

### ✅ DONE 5.3 Add a progress or status indicator to model downloads in Runtime settings

**Problem:** Model download buttons show "Downloading..." but provide no estimate, progress bar, or confirmation that the download is happening. Large Whisper or TTS models can take several minutes.

- [x] In `src/settings-panels/runtime.ts`, find the model download button rendering for TTS and ASR models.
- [x] When `isDownloading` is true for a model, show a spinner (same CSS as 5.1) alongside the "Downloading..." label.
- [x] If the backend provides download progress as a percentage, expose that value through the runtime panel state and display a simple text progress indicator (e.g., "Downloading... 42%"). If the backend does not provide progress, a spinner alone is sufficient.
- [x] When download completes, update the button area to show a success state ("Downloaded ✓") briefly before reverting to the normal idle state.
- [x] Update render tests for the download button states.

### ✅ DONE 5.4 Add an indeterminate progress indicator to the planner settings "Save settings" button

**Problem:** Saving planner endpoint/model settings is asynchronous but the button only changes its label. No spinner is shown.

- [x] In `src/settings-panels/planner.ts`, find the save-settings button.
- [x] Apply the spinner pattern from 5.1 when `isSaving` is true.

---

## Phase 6 — Accessibility polish

### ✅ DONE 6.1 Add a text label beside the model freshness dot in planner settings

**Problem:** The model freshness indicator in planner settings is a colored dot (green = fresh, red = stale). Colorblind users cannot distinguish the states. The dot has an `aria-label` but sighted users who don't check ARIA labels get color only.

- [x] In `src/settings-panels/planner.ts`, find the model status indicator (the dot element with `aria-label`).
- [x] Add a short visible text label next to the dot. For example: a green dot + "Up to date" or a red dot + "Reload needed".
- [x] Move the state information into the visible text; the color can remain as a secondary reinforcement.
- [x] In `src/styles.css`, style the text label to appear beside the dot inline.
- [x] Update any render tests that assert on the model status indicator.

### ✅ DONE 6.2 Deduplicate PTT hint text and aria-label

**Problem:** In all four push-to-talk states, the visible hint text below the button and the button's `aria-label` contain essentially the same message. Screen reader users hear the message twice.

- [x] In `src/confirmation-panels/push-to-talk.ts`, review each of the four states (idle, holding, listening-busy, hands-free):
  - Keep the `aria-label` on the button to describe the button's current action.
  - Update the hint text below the button to provide **complementary** information, not a restatement. For example:
    - Idle state: `aria-label` = "Hold to talk"; hint = "Say a URL or command"
    - Holding state: `aria-label` = "Release to send"; hint = "Listening..."
    - Processing state: `aria-label` = "Processing"; hint = "Working on your command"
    - Hands-free active: `aria-label` = "Voice input active"; hint = "Say 'stop listening' to end"
- [x] Update render tests that assert on the hint text or aria-label for each state.

### ✅ DONE 6.3 Add aria-live to confirmation panel error display

**Problem:** When a confirmation submission fails, an error appears in the confirmation panel but there is no `aria-live` region to announce it. Screen reader users may miss the error entirely.

- [x] In `src/confirmation-panels/confirmation.ts`, find the error display element (the block that shows transport errors and backend tool errors).
- [x] Add `aria-live="assertive"` to the error container, since confirmation submission errors are high-priority information the user must act on.
- [x] Ensure the error container is always present in the DOM (even when empty) so that the `aria-live` region is registered before errors appear. Use an empty element that becomes populated rather than a conditionally rendered element.
- [x] Update tests to verify the error container has the correct `aria-live` attribute.

### ✅ DONE 6.4 Improve disabled state contrast on the push-to-talk button

**Problem:** When the push-to-talk button is disabled, it uses `opacity: 0.56`. The visual difference between disabled and active states is too subtle, especially for low-vision users.

- [x] In `src/styles.css`, find `.push-to-talk-button:disabled` (or the equivalent disabled styling).
- [x] Instead of only reducing opacity, also change the background color to a clearly distinct muted state (e.g., a gray or the warm neutral `#c0b49a`), so the disabled state is distinguishable by shape and hue, not only opacity.
- [x] Ensure the disabled cursor (`cursor: not-allowed`) is present.
- [x] Verify the change does not interfere with the active/listening state styles.

### ✅ DONE 6.5 Add visible focus indicators on custom button elements

**Problem:** The shell uses MUI `IconButton` for the settings gear and back arrow. These have MUI's default focus ring, which may be overridden by the custom `CssBaseline`. Confirm visible focus rings are present for keyboard users.

- [x] In `src/styles.css`, confirm that `.shell-toolbar-action:focus-visible` and `.settings-subpage-back:focus-visible` have a clearly visible outline.
- [x] If the MUI `CssBaseline` removes the default browser focus ring, add an explicit `:focus-visible` outline rule for all interactive toolbar elements.
- [x] Also check `.settings-subpage-card:focus-visible` (the overview navigation cards) for a visible focus ring.
- [x] Visually verify focus rings appear when tabbing through the app's toolbar and settings overview.

---

## Phase 7 — Final validation

### 7.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] Fix any lint errors or test failures before committing.

### 7.2 Walk the first-time user flow

- [ ] Open the app with no providers configured.
- [ ] Confirm the setup-required banner appears and the Talk button shows a visible error.
- [ ] Follow the banner to settings. Confirm status indicators on the overview cards indicate which sections need attention.
- [ ] Configure ASR and TTS (use a test API key or local model path).
- [ ] Return to workspace. Confirm the banner is gone and the Talk button is enabled.

### 7.3 Walk the settings navigation flow

- [ ] Open settings. Confirm overview uses plain-language headings.
- [ ] Navigate to Voice input setup (ASR). Note the current `settingsView`.
- [ ] Switch to workspace. Switch back to settings. Confirm you land on Voice input setup, not the overview.
- [ ] Navigate back to overview via the back button. Confirm the back button is present and works.

### 7.4 Walk the confirmation flow

- [ ] Trigger a confirmation (say a command that requires approval).
- [ ] Confirm the prompt text is human-readable, no internal metadata is shown, and the approve/reject buttons are prominent.
- [ ] Approve the action. Confirm the panel disappears cleanly.
- [ ] Trigger a confirmation and force a submission error (disconnect backend if possible). Confirm the error is visible and announced by screen readers.

### 7.5 Check all jargon replacements

- [ ] Open each settings subpage and confirm no user-facing heading or label uses "TTS", "ASR", "Planner", "Region", "Keyring", or raw millisecond values.
- [ ] Open the Runtime settings. Confirm the "Provider Failover" section is gone.
- [ ] Open the planner settings. Confirm the model dropdown placeholder is non-selectable.
- [ ] Check the playback settings. Confirm the speed slider max is 2.5× (or 3× at most).

### 7.6 Update memory.md

- [ ] After completing all phases, run `date -u +"%Y-%m-%dT%H:%M:%SZ"` to get the current timestamp.
- [ ] Add an entry to `memory.md` summarizing which phases were completed, the final commit hash, and validation status.

---

## Suggested commit sequence

### Commit 1
Phase 1: critical functional fixes — PTT error visibility, reset confirmation, failover section removal, settings navigation context preservation.

### Commit 2
Phase 2: first-run and unconfigured state — setup-required banner, settings overview status indicators.

### Commit 3
Phase 3: plain-language copy pass — subpage headings, abbreviations, timeout formatting, technical field removal, "Region" → "Section", keyring copy, badge copy.

### Commit 4
Phase 4: settings UX improvements — non-selectable placeholder option, speed slider range, read-only card styling, model-missing warnings.

### Commit 5
Phase 5: progress indicators — spinners on API key test, load models, model download, save settings.

### Commit 6
Phase 6: accessibility polish — model freshness dot label, PTT hint deduplication, confirmation error aria-live, disabled state contrast, focus indicators.

### Commit 7
Phase 7: full validation pass, memory.md update.
