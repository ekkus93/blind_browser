# ChatGPT-Readable CI Status Bridge

**Repository:** `ekkus93/blind_browser`  
**Default branch:** `master`  
**Status:** Installed  

## Monitored target

| Field | Value |
|---|---|
| Workflow | `CI` |
| Workflow file | `.github/workflows/ci.yml` |
| Branch | `master` |
| Status issue | `#1` — `CI Status: Hosted Quality Gates — master` |
| Publisher | `.github/workflows/publish-ci-status.yml` |

`CI` is the repository's authoritative hosted quality gate. It runs Rust formatting, default-feature checking, Clippy with warnings denied, Rust tests with all features, frontend lint, UI tests, and the production frontend build.

## Publisher behavior

The publisher listens for `requested`, `in_progress`, and `completed` `workflow_run` events from `CI`.

Before changing issue #1, it deterministically verifies:

1. the triggering workflow name is exactly `CI`;
2. the triggering branch is exactly `master`;
3. the triggering run ID is still the latest run for the `CI` workflow on `master`;
4. issue #1 begins with the expected automation ownership marker.

If the event is stale, the publisher exits successfully without updating the issue. A stale publisher step writes `publish=false`, and all metadata-fetch and issue-update steps are explicitly conditioned on `publish=true`; therefore a stale event cannot continue into the write path.

## Security properties

The publisher uses only:

```yaml
permissions:
  actions: read
  contents: read
  issues: write
```

It does not check out repository code, execute code from the triggering commit, download or execute artifacts, publish raw logs, print environment variables, or use a personal access token.

Workflow, branch, job, step, and artifact names are treated as untrusted display strings. Human-readable values are normalized before being placed in inline Markdown. Raw workflow logs and environment variables are never copied into the status issue.

## Published data

Issue #1 contains a concise Markdown summary and a parseable JSON document with:

- schema version and publisher identity;
- status issue number and monitored branch;
- workflow name and ID;
- run ID, attempt, URL, event, status, and conclusion;
- exact head branch and full head SHA;
- timestamps;
- job IDs, job state, runner metadata, and every available step state;
- all abnormal job or step conclusions;
- artifact IDs, names, sizes, expiry state, and timestamps after completion;
- explicit jobs/artifacts availability state;
- explicit issue-body compaction state.

Job and artifact API responses are fetched with pagination. The generated issue body is bounded to 60,000 UTF-8 bytes. If needed, successful step details are compacted first while preserving workflow metadata, every job ID and conclusion, and all abnormal-step details. Invalid or still-oversized output fails closed rather than publishing malformed JSON.

## ChatGPT operating procedure

For each candidate commit during a Ralph loop:

1. Record the exact candidate SHA.
2. Read issue #1.
3. Parse the fenced JSON block.
4. Compare `workflow.head_sha` with the candidate SHA.
5. Ignore status for any different SHA.
6. Use `workflow.run_id` to retrieve the run's jobs when needed.
7. Use the exact failed `jobs[].id` to retrieve the relevant job log.
8. Fix the first meaningful failure and push a new candidate.
9. Repeat until issue #1 reports `completed` / `success` for the exact candidate SHA.

The issue is a run-discovery and indexing bridge. GitHub Actions job logs remain the source for detailed command output, and a hosted CI success does not substitute for device, platform, accessibility, or manual acceptance evidence not covered by the workflow.

## Ownership marker

The publisher will only overwrite an issue whose body begins with:

```html
<!-- maintained by .github/workflows/publish-ci-status.yml -->
```

Removing or changing that marker intentionally disables issue updates until ownership is restored.
