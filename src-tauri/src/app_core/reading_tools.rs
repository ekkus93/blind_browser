use super::narration::{narration_consent_required_error, NarrationAttempt};
use crate::commands::{
    ReadNextRegionData, ReadNextRegionInput, ReadPreviousRegionData, ReadPreviousRegionInput,
    ReadRegionData, ReadRegionInput, StopSpeakingData, StopSpeakingInput, ToolError, ToolName,
    ToolResult,
};
use crate::narration::{next_region_index, previous_region_index};

impl super::AppCore {
    pub fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        self.sync_narration_playback_state();
        let region_id = input.region_id.trim().to_string();
        if region_id.is_empty() {
            return ToolResult::failure(
                ToolName::ReadRegion,
                input.request_id,
                ToolError {
                    code: String::from("invalid_region_id"),
                    message: String::from("read_region requires a non-empty region_id"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Narration request was rejected because the region_id was empty.",
                )],
            );
        }

        let (region_index, region) = match self.region_by_id(&region_id) {
            Ok(region) => region,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration request could not resolve the requested region in the current page model.",
                    )],
                )
            }
        };

        let interrupted_region_id = match self.begin_region_narration(
            region_index,
            &region,
            input.interruption_mode.interrupts_current_playback(),
            &input.request_id,
        ) {
            Ok(NarrationAttempt::Completed(interrupted_region_id)) => interrupted_region_id,
            Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                return ToolResult::failure(
                    ToolName::ReadRegion,
                    input.request_id,
                    narration_consent_required_error(&challenge),
                    vec![String::from(
                        "Narration was paused because sending this page's text to remote narration requires your permission first.",
                    )],
                )
            }
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration request could not start playback for the requested region.",
                    )],
                )
            }
        };

        let mut observations = vec![format!(
            "Moved the narration cursor to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before starting the new region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadRegion,
            input.request_id,
            ReadRegionData {
                region_id: region.region_id,
                region_index,
                text_length: region.text.chars().count(),
                speech_started: true,
            },
            observations,
        )
    }

    pub fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        self.sync_narration_playback_state();
        let regions = match self.readable_regions() {
            Ok(regions) => regions,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                    "Narration could not advance because the current page has no readable regions.",
                )],
                )
            }
        };

        // Bounds-checked regardless of what next_region_index guarantees: a
        // narration path must never index the region list unconditionally, so
        // a `None` index and an out-of-range index take the same "no next
        // region" path rather than panicking.
        let Some((region_index, region)) =
            next_region_index(&self.state.narration_cursor, regions.len())
                .and_then(|index| regions.get(index).map(|region| (index, region.clone())))
        else {
            return ToolResult::success(
                ToolName::ReadNextRegion,
                input.request_id,
                ReadNextRegionData {
                    cursor: self.state.narration_cursor.clone(),
                    region_id: None,
                    speech_started: false,
                    boundary: crate::commands::NarrationBoundary::End,
                },
                vec![String::from(
                    "Narration is already at the end of the readable region list.",
                )],
            );
        };
        let interrupted_region_id = match self.begin_region_narration(
            region_index,
            &region,
            input.interruption_mode.interrupts_current_playback(),
            &input.request_id,
        ) {
            Ok(NarrationAttempt::Completed(interrupted_region_id)) => interrupted_region_id,
            Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                return ToolResult::failure(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    narration_consent_required_error(&challenge),
                    vec![String::from(
                        "Narration was paused because sending this page's text to remote narration requires your permission first.",
                    )],
                )
            }
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not advance to the next region for playback.",
                    )],
                )
            }
        };

        let mut observations = vec![format!(
            "Advanced narration to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before reading the next region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadNextRegion,
            input.request_id,
            ReadNextRegionData {
                cursor: self.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                boundary: crate::commands::NarrationBoundary::None,
            },
            observations,
        )
    }

    pub fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        self.sync_narration_playback_state();
        let regions = match self.readable_regions() {
            Ok(regions) => regions,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not move backward because the current page has no readable regions.",
                    )],
                )
            }
        };

        // Bounds-checked regardless of what previous_region_index guarantees:
        // a narration path must never index the region list unconditionally,
        // so a `None` index and an out-of-range index take the same "no
        // previous region" path rather than panicking.
        let Some((region_index, region)) =
            previous_region_index(&self.state.narration_cursor, regions.len())
                .and_then(|index| regions.get(index).map(|region| (index, region.clone())))
        else {
            return ToolResult::success(
                ToolName::ReadPreviousRegion,
                input.request_id,
                ReadPreviousRegionData {
                    cursor: self.state.narration_cursor.clone(),
                    region_id: None,
                    speech_started: false,
                    boundary: crate::commands::NarrationBoundary::Start,
                },
                vec![String::from(
                    "Narration is already at the start of the readable region list.",
                )],
            );
        };
        let interrupted_region_id = match self.begin_region_narration(
            region_index,
            &region,
            input.interruption_mode.interrupts_current_playback(),
            &input.request_id,
        ) {
            Ok(NarrationAttempt::Completed(interrupted_region_id)) => interrupted_region_id,
            Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                return ToolResult::failure(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    narration_consent_required_error(&challenge),
                    vec![String::from(
                        "Narration was paused because sending this page's text to remote narration requires your permission first.",
                    )],
                )
            }
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not move backward to the previous region for playback.",
                    )],
                )
            }
        };

        let mut observations = vec![format!(
            "Moved narration backward to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before reading the previous region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadPreviousRegion,
            input.request_id,
            ReadPreviousRegionData {
                cursor: self.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                boundary: crate::commands::NarrationBoundary::None,
            },
            observations,
        )
    }

    pub fn execute_stop_speaking(
        &mut self,
        input: StopSpeakingInput,
    ) -> ToolResult<StopSpeakingData> {
        self.sync_narration_playback_state();
        let interrupted_region_id = self.stop_narration_playback();
        let stopped = interrupted_region_id.is_some();

        ToolResult::success(
            ToolName::StopSpeaking,
            input.request_id,
            StopSpeakingData {
                stopped,
                interrupted_region_id,
            },
            vec![if stopped {
                String::from("Stopped the active narration region.")
            } else {
                String::from("No narration was active, so there was nothing to stop.")
            }],
        )
    }
}
