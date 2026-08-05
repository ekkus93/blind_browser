import { classifyInvokeFailure } from "./api/errors.ts";
import {
  createRuntimeRefreshHandlers,
  type RuntimeRefreshDependencies,
} from "./runtime-refresh.ts";
import {
  getRemotePlannerPrivacyStatus,
  type RemotePlannerPrivacyStatus,
} from "./tauri-api.ts";

export interface RemotePlannerPrivacyRefreshDependencies {
  createRequestId: (prefix: string) => string;
  markRemotePlannerPrivacyRefreshStarted: () => void;
  applyRemotePlannerPrivacyRefreshSuccess: (
    status: RemotePlannerPrivacyStatus,
  ) => void;
  applyRemotePlannerPrivacyRefreshFailure: (message: string) => void;
}

export interface RuntimeRefreshWithPrivacyDependencies
  extends RuntimeRefreshDependencies {
  markRemotePlannerPrivacyRefreshStarted: () => void;
  applyRemotePlannerPrivacyRefreshSuccess: (
    status: RemotePlannerPrivacyStatus,
  ) => void;
  applyRemotePlannerPrivacyRefreshFailure: (message: string) => void;
}

export type RemotePlannerPrivacyStatusLoader = typeof getRemotePlannerPrivacyStatus;

export function describeRemotePlannerPrivacyRefreshFailure(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  const message = failure.kind === "tool-error"
    ? failure.toolError.message
    : failure.message;
  return `Remote planner privacy status could not be refreshed. ${message}`;
}

export async function refreshRemotePlannerPrivacyStatus(
  dependencies: RemotePlannerPrivacyRefreshDependencies,
  loadStatus: RemotePlannerPrivacyStatusLoader = getRemotePlannerPrivacyStatus,
): Promise<void> {
  dependencies.markRemotePlannerPrivacyRefreshStarted();
  try {
    const status = await loadStatus({
      requestId: dependencies.createRequestId("remote-planner-privacy-status"),
    });
    dependencies.applyRemotePlannerPrivacyRefreshSuccess(status);
  } catch (error) {
    dependencies.applyRemotePlannerPrivacyRefreshFailure(
      describeRemotePlannerPrivacyRefreshFailure(error),
    );
  }
}

export function createRuntimeRefreshHandlersWithPrivacy(
  dependencies: RuntimeRefreshWithPrivacyDependencies,
  loadStatus: RemotePlannerPrivacyStatusLoader = getRemotePlannerPrivacyStatus,
) {
  const {
    markRemotePlannerPrivacyRefreshStarted,
    applyRemotePlannerPrivacyRefreshSuccess,
    applyRemotePlannerPrivacyRefreshFailure,
    ...runtimeDependencies
  } = dependencies;
  const runtimeHandlers = createRuntimeRefreshHandlers(runtimeDependencies);
  const privacyDependencies: RemotePlannerPrivacyRefreshDependencies = {
    createRequestId: dependencies.createRequestId,
    markRemotePlannerPrivacyRefreshStarted,
    applyRemotePlannerPrivacyRefreshSuccess,
    applyRemotePlannerPrivacyRefreshFailure,
  };

  return {
    ...runtimeHandlers,
    refreshRuntimePanelsFromRuntime: async (): Promise<void> => {
      const [runtimeResult, privacyResult] = await Promise.allSettled([
        runtimeHandlers.refreshRuntimePanelsFromRuntime(),
        refreshRemotePlannerPrivacyStatus(privacyDependencies, loadStatus),
      ]);

      if (runtimeResult.status === "rejected") {
        throw runtimeResult.reason;
      }
      if (privacyResult.status === "rejected") {
        throw privacyResult.reason;
      }
    },
  };
}
