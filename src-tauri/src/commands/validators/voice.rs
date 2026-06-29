use super::invalid_planner_output;
use crate::commands::{ReadRegionInput, ToolError, TranscribeCommandInput};

pub(super) fn validate_read_region_input(input: &ReadRegionInput) -> Result<(), ToolError> {
    if input.region_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "read_region requires a non-empty region_id",
            None,
        ));
    }

    Ok(())
}

pub(super) fn validate_transcribe_command_input(
    input: &TranscribeCommandInput,
) -> Result<(), ToolError> {
    if matches!(input.max_duration_ms, Some(0)) {
        return Err(invalid_planner_output(
            "transcribe_command max_duration_ms must be greater than 0 when provided",
            None,
        ));
    }

    Ok(())
}
