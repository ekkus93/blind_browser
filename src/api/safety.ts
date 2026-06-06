import { invokeCommand } from "./errors.ts";

export async function setConfirmationThreshold(input: {
  requestId: string;
  timeoutMs?: number;
  confirmationConfidenceThreshold: number;
}): Promise<{ confirmation_confidence_threshold: number; changed: boolean }> {
  return invokeCommand<{ confirmation_confidence_threshold: number; changed: boolean }>(
    "set_confirmation_threshold",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      confirmationConfidenceThreshold: input.confirmationConfidenceThreshold,
    },
  );
}

export async function setAllowClickWithoutConfirmation(input: {
  requestId: string;
  timeoutMs?: number;
  allowClickWithoutConfirmation: boolean;
}): Promise<{ allow_click_without_confirmation: boolean; changed: boolean }> {
  return invokeCommand<{ allow_click_without_confirmation: boolean; changed: boolean }>(
    "set_allow_click_without_confirmation",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      allowClickWithoutConfirmation: input.allowClickWithoutConfirmation,
    },
  );
}

export async function setOcrThresholds(input: {
  requestId: string;
  timeoutMs?: number;
  sparseTextCharThreshold: number;
  sparseTextRegionThreshold: number;
}): Promise<{
  sparse_text_char_threshold: number;
  sparse_text_region_threshold: number;
  changed: boolean;
}> {
  return invokeCommand<{
    sparse_text_char_threshold: number;
    sparse_text_region_threshold: number;
    changed: boolean;
  }>("set_ocr_thresholds", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    sparseTextCharThreshold: input.sparseTextCharThreshold,
    sparseTextRegionThreshold: input.sparseTextRegionThreshold,
  });
}
