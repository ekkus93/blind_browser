use crate::commands::{
    FindElementData, FindElementInput, ListInteractiveElementsData, ListInteractiveElementsInput,
    ToolError, ToolName, ToolResult,
};
use crate::app_core::element_scoring::{
    build_find_element_query, determine_find_element_resolution, filter_interactive_elements,
    rank_find_element_candidates,
};

impl super::super::AppCore {
    pub fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "list_interactive_elements requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Interactive elements could not be listed because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "list_interactive_elements requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Interactive elements could not be listed because the runtime page model is missing.",
                )],
            );
        };

        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visibility_filter.visible_only(),
            input.roles.as_deref(),
        );
        let visible_count = elements.iter().filter(|element| element.visible).count();

        let mut observations = vec![String::from(
            "Listed deterministic interactive elements from the current runtime page state.",
        )];
        if input.visibility_filter.visible_only() {
            observations.push(String::from(
                "Results were filtered to elements currently marked visible in runtime state.",
            ));
        }
        if let Some(roles) = input.roles.as_ref() {
            observations.push(format!(
                "Results were filtered to {} requested role(s).",
                roles.len()
            ));
        }

        ToolResult::success(
            ToolName::ListInteractiveElements,
            input.request_id,
            ListInteractiveElementsData {
                page_id,
                elements,
                visible_count,
            },
            observations,
        )
    }

    pub fn execute_find_element(
        &mut self,
        input: FindElementInput,
    ) -> ToolResult<FindElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("find_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Element search could not run because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "find_element requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Element search could not run because the runtime page model is missing.",
                )],
            );
        };

        let search_query = match build_find_element_query(&input) {
            Ok(search_query) => search_query,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::FindElement,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Element search was rejected because the search criteria were empty.",
                    )],
                )
            }
        };

        let candidate_limit = input
            .max_candidates
            .unwrap_or(super::super::DEFAULT_FIND_ELEMENT_MAX_CANDIDATES)
            .clamp(1, super::super::MAX_FIND_ELEMENT_CANDIDATES);
        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visibility_filter.visible_only(),
            input.role.as_ref().map(std::slice::from_ref),
        );
        let ranked_candidates =
            rank_find_element_candidates(&elements, &search_query, candidate_limit);
        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(
                &ranked_candidates,
                self.config.safety.confirmation_confidence_threshold,
            );

        let mut observations = vec![format!(
            "Searched {} interactive element(s) from the current runtime page state.",
            elements.len()
        )];
        if input.visibility_filter.visible_only() {
            observations.push(String::from(
                "Search was limited to elements currently marked visible in runtime state.",
            ));
        }
        if input
            .max_candidates
            .is_some_and(|value| value > super::super::MAX_FIND_ELEMENT_CANDIDATES)
        {
            observations.push(format!(
                "Candidate count was clamped to the supported maximum of {}.",
                super::super::MAX_FIND_ELEMENT_CANDIDATES
            ));
        }
        if ranked_candidates.is_empty() {
            observations.push(String::from(
                "No interactive elements produced a positive match score for the requested search criteria.",
            ));
        } else if requires_confirmation {
            observations.push(String::from(
                "Top candidates are too close to choose deterministically, so planner clarification is required before any side effect.",
            ));
        } else {
            observations.push(String::from(
                "A single strongest candidate was identified from the filtered interactive elements.",
            ));
        }

        ToolResult::success(
            ToolName::FindElement,
            input.request_id,
            FindElementData {
                query_summary: search_query.summary,
                chosen_element_id,
                chosen_confidence,
                candidates: ranked_candidates,
                requires_confirmation,
            },
            observations,
        )
    }
}
