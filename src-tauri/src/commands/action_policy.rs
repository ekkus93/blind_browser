use super::*;

pub(crate) const CLICK_AUTH_TOKEN_ARG: &str = "_runtime_click_authorization";
pub(crate) const CLICK_AUTH_CONFIDENCE_ARG: &str = "_runtime_click_confidence_bps";
pub(crate) const CLICK_AUTH_AMBIGUOUS_ARG: &str = "_runtime_click_ambiguous";
pub(crate) const CLICK_AUTH_DESTRUCTIVE_ARG: &str = "_runtime_click_potentially_destructive";
pub(crate) const CLICK_AUTH_GENERATION_ARG: &str = "_runtime_click_page_generation";
pub(crate) const RUNTIME_TARGET_LABEL_ARG: &str = "_runtime_target_label";
pub(crate) const RUNTIME_FORM_LABEL_ARG: &str = "_runtime_form_label";
pub(crate) const RUNTIME_FORM_DESTINATION_ARG: &str = "_runtime_form_destination";
pub(crate) const RUNTIME_FORM_FIELDS_ARG: &str = "_runtime_form_fields";

/// Deterministic classification of every planner-visible tool. This match is
/// intentionally exhaustive: adding a new `ToolName` fails compilation until
/// its security class and minimum confirmation requirement are chosen.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ActionClass {
    ReadOnly,
    ReversibleLocalStateChange,
    BrowserNavigation,
    PageInteraction,
    DataEntry,
    FormSubmission,
    ArbitraryScriptExecution,
    CredentialOperation,
    ModelDownload,
    OtherSideEffect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ConfirmationRequirement {
    NoConfirmation,
    ConfirmationRequired,
    Prohibited,
}

impl ConfirmationRequirement {
    fn severity(self) -> u8 {
        match self {
            Self::NoConfirmation => 0,
            Self::ConfirmationRequired => 1,
            Self::Prohibited => 2,
        }
    }

    fn strongest(self, other: Self) -> Self {
        if other.severity() > self.severity() {
            other
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ActionPolicyReasonCode {
    ToolClassMinimum,
    SubmitRequiresConfirmation,
    TextEntrySubmitsForm,
    ClickRequiresConfirmationBySetting,
    ClickGroundingUnavailable,
    ClickGroundingAuthorized,
    ClickConfidenceBelowThreshold,
    ClickGroundingAmbiguous,
    ClickTargetPotentiallyDestructive,
    EvalJsProhibited,
    MalformedProtectedArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolPolicy {
    pub class: ActionClass,
    pub minimum_confirmation: ConfirmationRequirement,
    pub reason_code: ActionPolicyReasonCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ActionPolicyFinding {
    pub step_id: String,
    pub tool_name: ToolName,
    pub class: ActionClass,
    pub requirement: ConfirmationRequirement,
    pub reason_code: ActionPolicyReasonCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ActionPolicyDecision {
    pub requirement: ConfirmationRequirement,
    pub findings: Vec<ActionPolicyFinding>,
}

pub fn tool_policy(tool_name: &ToolName) -> ToolPolicy {
    let (class, minimum_confirmation, reason_code) = match tool_name {
        ToolName::OpenUrl | ToolName::GoBack | ToolName::GoForward | ToolName::ReloadPage => (
            ActionClass::BrowserNavigation,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::GetHtml
        | ToolName::GetPageSnapshot
        | ToolName::ExtractPageModel
        | ToolName::ListInteractiveElements
        | ToolName::FindElement
        | ToolName::RunOcr
        | ToolName::GetAgentState
        | ToolName::GetRuntimeStatus
        | ToolName::ReportResult => (
            ActionClass::ReadOnly,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::EvalJs => (
            ActionClass::ArbitraryScriptExecution,
            ConfirmationRequirement::Prohibited,
            ActionPolicyReasonCode::EvalJsProhibited,
        ),
        ToolName::ScrollPage | ToolName::FocusElement => (
            ActionClass::PageInteraction,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::ClickElement => (
            ActionClass::PageInteraction,
            ConfirmationRequirement::ConfirmationRequired,
            ActionPolicyReasonCode::ClickGroundingUnavailable,
        ),
        ToolName::TypeIntoElement => (
            ActionClass::DataEntry,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::SubmitActiveForm => (
            ActionClass::FormSubmission,
            ConfirmationRequirement::ConfirmationRequired,
            ActionPolicyReasonCode::SubmitRequiresConfirmation,
        ),
        ToolName::CaptureScreenshot
        | ToolName::SetBrowserVisibility
        | ToolName::ReadRegion
        | ToolName::ReadNextRegion
        | ToolName::ReadPreviousRegion
        | ToolName::StopSpeaking
        | ToolName::StartListening
        | ToolName::StopListening
        | ToolName::TranscribeCommand
        | ToolName::SetTtsVoice
        | ToolName::SetPlaybackVolume
        | ToolName::SetPlaybackSpeed
        | ToolName::MergeOcrIntoPageModel
        | ToolName::ConfirmAction => (
            ActionClass::ReversibleLocalStateChange,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
    };

    ToolPolicy {
        class,
        minimum_confirmation,
        reason_code,
    }
}

pub fn evaluate_action_policy(
    steps: &[PlannedStep],
    safety: &PlannerSafetySettings,
) -> ActionPolicyDecision {
    let mut requirement = ConfirmationRequirement::NoConfirmation;
    let mut findings = Vec::new();

    for step in steps {
        let policy = tool_policy(&step.tool_name);
        let mut step_requirement = policy.minimum_confirmation;
        let mut reason_code = policy.reason_code;

        match step.tool_name {
            ToolName::ClickElement => {
                let token_present = step
                    .arguments
                    .get(CLICK_AUTH_TOKEN_ARG)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty());
                let confidence_bps = step
                    .arguments
                    .get(CLICK_AUTH_CONFIDENCE_ARG)
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok());
                let ambiguous = step
                    .arguments
                    .get(CLICK_AUTH_AMBIGUOUS_ARG)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let potentially_destructive = step
                    .arguments
                    .get(CLICK_AUTH_DESTRUCTIVE_ARG)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let threshold_bps = (safety.confirmation_confidence_threshold.clamp(0.0, 1.0)
                    * 10_000.0)
                    .round() as u16;

                let (requirement, reason) = if !safety.allow_click_without_confirmation {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickRequiresConfirmationBySetting,
                    )
                } else if !token_present || confidence_bps.is_none() {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickGroundingUnavailable,
                    )
                } else if ambiguous {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickGroundingAmbiguous,
                    )
                } else if potentially_destructive {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickTargetPotentiallyDestructive,
                    )
                } else if confidence_bps.is_none_or(|value| value < threshold_bps) {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickConfidenceBelowThreshold,
                    )
                } else {
                    (
                        ConfirmationRequirement::NoConfirmation,
                        ActionPolicyReasonCode::ClickGroundingAuthorized,
                    )
                };
                step_requirement = requirement;
                reason_code = reason;
            }
            ToolName::SubmitActiveForm => {
                // Form submission is a runtime minimum. The legacy setting may make
                // the prompt stricter, but it can never weaken this invariant.
                let _legacy_setting = safety.always_confirm_submit;
                step_requirement = ConfirmationRequirement::ConfirmationRequired;
                reason_code = ActionPolicyReasonCode::SubmitRequiresConfirmation;
            }
            ToolName::TypeIntoElement => {
                match serde_json::from_value::<TypeIntoElementInput>(step.arguments.clone()) {
                    Ok(input) if input.submit_mode.submits_after_entry() => {
                        step_requirement = ConfirmationRequirement::ConfirmationRequired;
                        reason_code = ActionPolicyReasonCode::TextEntrySubmitsForm;
                    }
                    Ok(_) => {}
                    Err(_) => {
                        // Argument validation normally rejects this first. Executor
                        // defense in depth still treats malformed data-entry calls as
                        // protected instead of assuming they are harmless.
                        step_requirement = ConfirmationRequirement::ConfirmationRequired;
                        reason_code = ActionPolicyReasonCode::MalformedProtectedArguments;
                    }
                }
            }
            ToolName::EvalJs => {
                step_requirement = ConfirmationRequirement::Prohibited;
                reason_code = ActionPolicyReasonCode::EvalJsProhibited;
            }
            _ => {}
        }

        requirement = requirement.strongest(step_requirement);
        if step_requirement != ConfirmationRequirement::NoConfirmation {
            findings.push(ActionPolicyFinding {
                step_id: step.step_id.clone(),
                tool_name: step.tool_name.clone(),
                class: policy.class,
                requirement: step_requirement,
                reason_code,
            });
        }
    }

    ActionPolicyDecision {
        requirement,
        findings,
    }
}
