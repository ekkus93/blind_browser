# SKILLS.md

## v1 Built-in Skills

This file defines the bundled default skill metadata used by the planner for v1.
Each built-in skill has:
- a stable `name`
- one or more `intent_tags`
- optional `allowed_tools`
- a `requires_confirmation` default
- a short `description`

The metadata here is the authoritative source for bundled default skills shipped with the application.

### 1. Navigation Skills

#### open_url
- intent_tags: `open-url`, `navigation`, `intent:OpenUrl`
- allowed_tools: `open_url`
- requires_confirmation: `false`
- description: Open a URL and navigate to the requested page.

#### go_back
- intent_tags: `go-back`, `navigation-history`, `intent:GoBack`
- allowed_tools: `go_back`
- requires_confirmation: `false`
- description: Navigate back in browser history.

#### go_forward
- intent_tags: `go-forward`, `navigation-history`, `intent:GoForward`
- allowed_tools: `go_forward`
- requires_confirmation: `false`
- description: Navigate forward in browser history.

#### reload_page
- intent_tags: `reload-page`, `navigation`, `intent:ReloadPage`
- allowed_tools: `reload_page`
- requires_confirmation: `false`
- description: Reload the current page.

#### scroll_page
- intent_tags: `scroll-page`, `navigation`, `intent:Scroll`
- allowed_tools: `scroll_page`
- requires_confirmation: `false`
- description: Scroll the current page by a bounded amount in the requested direction.

#### get_current_url
- intent_tags: `get-current-url`, `status-query`, `navigation`, `intent:GetCurrentUrl`
- allowed_tools: `get_agent_state`, `report_result`
- requires_confirmation: `false`
- description: Report the current page URL.

### 2. Reading Skills

#### read_page
- intent_tags: `read-page`, `reading`, `intent:ReadPage`
- allowed_tools: `extract_page_model`, `read_region`, `read_next_region`
- requires_confirmation: `false`
- description: Read the current page from the title and then through readable regions.

#### read_title
- intent_tags: `read-title`, `reading`, `intent:ReadTitle`
- allowed_tools: `report_result`
- requires_confirmation: `false`
- description: Read only the current page title.

#### read_current
- intent_tags: `read-current`, `reading`
- allowed_tools: `read_region`, `get_agent_state`
- requires_confirmation: `false`
- description: Read the current narration region.

#### read_next
- intent_tags: `read-next`, `reading`, `intent:ReadNext`
- allowed_tools: `read_next_region`
- requires_confirmation: `false`
- description: Read the next narration region.

#### read_previous
- intent_tags: `read-previous`, `reading`, `intent:ReadPrevious`
- allowed_tools: `read_previous_region`
- requires_confirmation: `false`
- description: Read the previous narration region.

#### repeat
- intent_tags: `repeat-reading`, `reading`, `intent:Repeat`
- allowed_tools: `read_region`, `get_agent_state`
- requires_confirmation: `false`
- description: Repeat the current narration region.

#### stop_reading
- intent_tags: `stop-reading`, `reading`, `intent:Stop`
- allowed_tools: `stop_speaking`
- requires_confirmation: `false`
- description: Stop current speech output.

#### pause_reading
- intent_tags: `pause-reading`, `reading`
- allowed_tools: `stop_speaking`
- requires_confirmation: `false`
- description: Pause current reading output.

#### resume_reading
- intent_tags: `resume-reading`, `reading`
- allowed_tools: `read_region`, `get_agent_state`
- requires_confirmation: `false`
- description: Resume reading from the current narration position.

### 3. Link Interaction Skills

#### read_links
- intent_tags: `read-links`, `links`, `navigation`
- allowed_tools: `list_interactive_elements`
- requires_confirmation: `false`
- description: Read available visible links from the current page context.

#### open_link_by_index
- intent_tags: `open-link-by-index`, `links`, `intent:ClickElement`
- allowed_tools: `list_interactive_elements`, `click_element`
- requires_confirmation: `false`
- description: Open a visible link by its spoken index in the current link list.

#### open_link_by_text
- intent_tags: `open-link-by-text`, `links`, `intent:FindElement`, `intent:ClickElement`
- allowed_tools: `find_element`, `click_element`
- requires_confirmation: `false`
- description: Find a link by its text and open it.

### 4. Form Interaction Skills

#### focus_field
- intent_tags: `focus-field`, `forms`, `intent:FillInput`
- allowed_tools: `find_element`, `focus_element`
- requires_confirmation: `false`
- description: Find an input field by label or description and move focus to it.

#### fill_field_by_label
- intent_tags: `fill-field-by-label`, `forms`, `intent:FillInput`
- allowed_tools: `find_element`, `focus_element`, `type_into_element`
- requires_confirmation: `false`
- description: Find a form field by its label, placeholder, or nearby text and enter the requested value.

#### fill_focused_field
- intent_tags: `fill-focused-field`, `forms`, `intent:FillInput`
- allowed_tools: `type_into_element`
- requires_confirmation: `false`
- description: Enter or replace text in the currently focused field.

#### submit_form
- intent_tags: `submit-form`, `forms`, `intent:SubmitForm`
- allowed_tools: `submit_active_form`, `confirm_action`
- requires_confirmation: `true`
- description: Submit the active form after explicit confirmation.

#### fill_and_submit_form
- intent_tags: `fill-and-submit-form`, `forms`, `intent:FillInput`, `intent:SubmitForm`
- allowed_tools: `find_element`, `focus_element`, `type_into_element`, `submit_active_form`, `confirm_action`
- requires_confirmation: `true`
- description: Complete one or more form fields and submit the form, with confirmation before submission.

### 5. Structure Navigation Skills

#### next_heading
- intent_tags: `next-heading`, `structure-navigation`
- allowed_tools: `extract_page_model`, `read_next_region`
- requires_confirmation: `false`
- description: Move to and read the next heading-oriented region.

#### previous_heading
- intent_tags: `previous-heading`, `structure-navigation`
- allowed_tools: `extract_page_model`, `read_previous_region`
- requires_confirmation: `false`
- description: Move to and read the previous heading-oriented region.

#### skip_navigation
- intent_tags: `skip-navigation`, `structure-navigation`
- allowed_tools: `extract_page_model`, `read_region`
- requires_confirmation: `false`
- description: Skip repetitive navigation content and jump to primary page content.

#### jump_to_content
- intent_tags: `jump-to-content`, `structure-navigation`
- allowed_tools: `extract_page_model`, `read_region`
- requires_confirmation: `false`
- description: Jump directly to the main readable content region.

### 6. OCR Fallback Skills

#### read_visible_text
- intent_tags: `read-visible-text`, `ocr-fallback`, `intent:OcrRecovery`
- allowed_tools: `get_page_snapshot`, `extract_page_model`, `run_ocr`, `merge_ocr_into_page_model`, `read_region`
- requires_confirmation: `false`
- description: Read visible page text using OCR fallback when extraction is insufficient.

#### ocr_current_region
- intent_tags: `ocr-current-region`, `ocr-fallback`, `intent:OcrRecovery`
- allowed_tools: `capture_screenshot`, `run_ocr`, `merge_ocr_into_page_model`
- requires_confirmation: `false`
- description: Run OCR on the current or targeted region.

#### describe_unreadable_region
- intent_tags: `describe-unreadable-region`, `ocr-fallback`, `intent:OcrRecovery`
- allowed_tools: `capture_screenshot`, `run_ocr`, `report_result`
- requires_confirmation: `false`
- description: Attempt to describe a region that could not be read through DOM extraction.

### 7. Voice Input Skills

#### start_listening
- intent_tags: `start-listening`, `voice-input`, `intent:StartListening`
- allowed_tools: `start_listening`
- requires_confirmation: `false`
- description: Start listening for voice input.

#### stop_listening
- intent_tags: `stop-listening`, `voice-input`, `intent:StopListening`
- allowed_tools: `stop_listening`
- requires_confirmation: `false`
- description: Stop listening for voice input.

#### transcribe_command
- intent_tags: `transcribe-command`, `voice-input`, `intent:TranscribeCommand`
- allowed_tools: `transcribe_command`
- requires_confirmation: `false`
- description: Capture and transcribe a short spoken command.

### 8. System Skills

#### set_tts_voice
- intent_tags: `set-tts-voice`, `settings`, `tts`, `intent:SetTtsVoice`
- allowed_tools: `set_tts_voice`, `report_result`
- requires_confirmation: `false`
- description: Change the active TTS voice setting.

#### set_playback_speed
- intent_tags: `set-playback-speed`, `settings`, `audio-speed`, `intent:SetPlaybackSpeed`
- allowed_tools: `set_playback_speed`, `report_result`
- requires_confirmation: `false`
- description: Set playback speed to a specific value.

#### increase_playback_speed
- intent_tags: `increase-playback-speed`, `settings`, `audio-speed`, `intent:SetPlaybackSpeed`
- allowed_tools: `set_playback_speed`, `report_result`
- requires_confirmation: `false`
- description: Increase playback speed by a normalized step.

#### decrease_playback_speed
- intent_tags: `decrease-playback-speed`, `settings`, `audio-speed`, `intent:SetPlaybackSpeed`
- allowed_tools: `set_playback_speed`, `report_result`
- requires_confirmation: `false`
- description: Decrease playback speed by a normalized step.

#### get_playback_speed
- intent_tags: `get-playback-speed`, `settings-query`, `audio-speed`, `intent:GetPlaybackSpeed`
- allowed_tools: `get_runtime_status`, `report_result`
- requires_confirmation: `false`
- description: Report the current playback speed.

#### set_volume
- intent_tags: `set-volume`, `settings`, `audio-volume`, `intent:SetPlaybackVolume`
- allowed_tools: `set_playback_volume`, `report_result`
- requires_confirmation: `false`
- description: Set playback volume to a specific value.

#### increase_volume
- intent_tags: `increase-volume`, `settings`, `audio-volume`, `intent:SetPlaybackVolume`
- allowed_tools: `set_playback_volume`, `report_result`
- requires_confirmation: `false`
- description: Increase playback volume by a normalized step.

#### decrease_volume
- intent_tags: `decrease-volume`, `settings`, `audio-volume`, `intent:SetPlaybackVolume`
- allowed_tools: `set_playback_volume`, `report_result`
- requires_confirmation: `false`
- description: Decrease playback volume by a normalized step.

#### mute_volume
- intent_tags: `mute-volume`, `settings`, `audio-volume`, `intent:SetPlaybackVolume`
- allowed_tools: `set_playback_volume`, `report_result`
- requires_confirmation: `false`
- description: Mute playback volume.

#### get_volume
- intent_tags: `get-volume`, `settings-query`, `audio-volume`, `intent:GetPlaybackVolume`
- allowed_tools: `get_runtime_status`, `report_result`
- requires_confirmation: `false`
- description: Report the current playback volume.

#### toggle_browser_visibility
- intent_tags: `toggle-browser-visibility`, `settings`, `browser-visibility`, `intent:SetBrowserVisibility`
- allowed_tools: `get_agent_state`, `set_browser_visibility`, `report_result`
- requires_confirmation: `false`
- description: Toggle the browser between visible and headless-style modes when supported.

#### get_status
- intent_tags: `get-status`, `status-query`, `intent:GetStatus`
- allowed_tools: `get_runtime_status`, `report_result`
- requires_confirmation: `false`
- description: Report the current application status.

### 9. Feedback Skills

#### confirm_action
- intent_tags: `confirm-action`, `feedback`
- allowed_tools: `confirm_action`
- requires_confirmation: `false`
- description: Ask the user for explicit confirmation before protected actions.

#### announce_state
- intent_tags: `announce-state`, `feedback`, `status-query`, `intent:GetStatus`
- allowed_tools: `get_runtime_status`, `get_agent_state`, `report_result`
- requires_confirmation: `false`
- description: Announce relevant current state to the user.

#### error_feedback
- intent_tags: `error-feedback`, `feedback`
- allowed_tools: `report_result`
- requires_confirmation: `false`
- description: Deliver short spoken error or recovery feedback.

---

## Planner Notes

- Bundled skills are advisory workflow guides, not executable tools.
- Bundled v1 skills should map to registered deterministic tools without relying on implied capabilities.
- If a bundled skill needs a tool that is not registered, the skill metadata should be updated together with the deterministic tool catalog before release.

---

## Design Rules

- Deterministic behavior
- Operate on PageModel where applicable
- Interruptible
- Clear success/failure
- Callable from UI and voice
- Short spoken confirmations by default

---

## v2 (Deferred)

- LLM-based action resolution (e.g., "press the red button")
- Semantic navigation
- Summarization
- Vision-based interaction
