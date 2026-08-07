import { type ReactNode } from "react";

const FOCUS_RING = "focus-visible:[outline:var(--focus-ring)] focus-visible:[outline-offset:var(--focus-offset)]";

// Structural anatomy shared by every `.settings-control-card` variant, with
// no background of its own -- each variant below appends exactly one
// background utility so two never end up stacked on the same element
// fighting over which wins (the bug that shipped: renderReadOnlyCard used to
// append a `bg-[color-mix(...)]` on top of CONTROL_CARD, which already set
// `bg-[var(--color-surface-inner)]`).
const CONTROL_CARD_STRUCTURE = "flex flex-col p-[16px_18px] rounded-[18px] border border-[rgba(41,88,63,0.14)]";
const CONTROL_CARD_BASE = `${CONTROL_CARD_STRUCTURE} bg-[var(--color-surface-inner)]`;
export const CONTROL_CARD = `${CONTROL_CARD_BASE} gap-2`;
// Read-only variant: same anatomy, its own (translucent) background instead
// of CONTROL_CARD's, complete and self-contained rather than layered on top.
// Exported so the cascade regression test in tailwind-cascade.test.mjs can
// compile this exact string rather than a hand-copied duplicate.
export const CONTROL_CARD_READONLY_CLASS = `${CONTROL_CARD_STRUCTURE} gap-2 bg-[color-mix(in_srgb,var(--color-surface-card)_72%,transparent)] pointer-events-none select-text`;
export const CONTROL_LABEL = "text-[0.84rem] uppercase tracking-[0.08em] text-[var(--color-text-label)] font-bold";
export const CONTROL_VALUE = "text-[1.1rem] font-bold text-[var(--color-text-primary)]";

// `.settings-secret-entry-card` / `.settings-planner-connection-card` in the old
// CSS were just `.settings-control-card` with a wider `gap` — same card anatomy,
// different spacing between the label/value/control rows.
export const SETTINGS_SECRET_ENTRY_CARD_CLASS = `${CONTROL_CARD_BASE} gap-3`;
export const SETTINGS_PLANNER_CONNECTION_CARD_CLASS = `${CONTROL_CARD_BASE} gap-[14px]`;

export const SETTINGS_PANEL_DESCRIPTION_CLASS = "leading-[1.55]";
// `.settings-panel-description.settings-panel-warning` combo — used wherever
// a warning-tier inline note appears inside a settings panel.
export const SETTINGS_PANEL_WARNING_CLASS = `${SETTINGS_PANEL_DESCRIPTION_CLASS} mt-[6px] text-[var(--color-amber-primary)] text-[0.84rem]`;

export const SETTINGS_CONTROL_SELECT_CLASS = `w-full p-[12px_14px] rounded-[14px] border border-[rgba(41,88,63,0.18)] bg-[var(--color-surface-inner)] text-[var(--color-text-primary)] [font:inherit] focus-visible:border-[rgba(41,88,63,0.3)] disabled:cursor-progress disabled:opacity-60 ${FOCUS_RING}`;

export const SETTINGS_CONTROL_INPUT_CLASS = "w-full accent-[var(--color-green-primary)] disabled:cursor-progress disabled:opacity-60";

// Shared anatomy for every `.settings-control-button` variant. The hover/focus
// box-shadow and translate are defined on the base class in the old CSS and
// never overridden per-variant (even the "danger" button gets the same
// green-tinted hover shadow) — kept here rather than duplicated per variant.
// Each exported button class below is a full, self-contained string (base +
// its own background/color/border) rather than base-plus-delta, so two
// variants' conflicting `bg-gradient-to-br`/`from-*`/`to-*` utilities never
// end up stacked on the same element fighting over the same CSS custom
// properties.
const CONTROL_BUTTON_BASE = `w-fit py-[10px] px-4 rounded-full border-none [font:inherit] font-bold cursor-pointer transition-[transform,box-shadow,opacity] duration-[120ms] enabled:hover:-translate-y-px enabled:hover:shadow-[0_10px_18px_rgba(41,88,63,0.2)] focus-visible:-translate-y-px focus-visible:shadow-[0_10px_18px_rgba(41,88,63,0.2)] ${FOCUS_RING} disabled:cursor-progress disabled:opacity-60 disabled:shadow-none`;
export const SETTINGS_CONTROL_BUTTON_CLASS = `${CONTROL_BUTTON_BASE} bg-gradient-to-br from-[var(--color-green-primary)] to-[#347f55] text-[#f6f2eb]`;
export const SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS = `${CONTROL_BUTTON_BASE} [background:var(--color-surface-inner)] text-[var(--color-text-primary)] [border:var(--inner-card-border)]`;
export const SETTINGS_CONTROL_BUTTON_DANGER_CLASS = `${CONTROL_BUTTON_BASE} bg-gradient-to-br from-[var(--color-error-primary)] to-[#b84436] text-[#fffdf8] border border-[rgba(139,52,42,0.3)] enabled:hover:from-[#a03d32] enabled:hover:to-[#c94e40] focus-visible:from-[#a03d32] focus-visible:to-[#c94e40]`;

export const SETTINGS_FIELD_GROUP_CLASS = "flex flex-col gap-2";
export const SETTINGS_INLINE_LABEL_ROW_CLASS = "flex items-center gap-2";

export const SETTINGS_STATUS_LIGHT_CLASS = "w-3 h-3 rounded-full flex-none shadow-[0_0_0_1px_rgba(31,37,39,0.14),0_0_10px_rgba(31,37,39,0.12)]";
export const SETTINGS_STATUS_LIGHT_FRESH_CLASS = "bg-[var(--color-green-fresh)]";
export const SETTINGS_STATUS_LIGHT_STALE_CLASS = "bg-[#c84c3a]";

export const SETTINGS_MODEL_FRESHNESS_INDICATOR_CLASS = "inline-flex items-center gap-[5px]";
export const SETTINGS_MODEL_FRESHNESS_LABEL_CLASS = "text-[0.78rem] font-medium text-[var(--eyebrow-color)]";

export const SETTINGS_INLINE_CONTROL_ROW_CLASS = "flex items-center gap-[10px]";
export const SETTINGS_INLINE_CONTROL_ROW_WRAP_CLASS = "flex items-center gap-[10px] flex-wrap";
export const SETTINGS_INLINE_CONTROL_FILL_CLASS = "flex-[1_1_320px] min-w-[min(320px,100%)]";

export const SETTINGS_INLINE_LOADING_CLASS = "inline-flex items-center gap-2 mt-2 text-[var(--color-text-muted)] text-[0.9rem] font-semibold";

export const SETTINGS_BUTTON_ROW_CLASS = "flex gap-[10px] items-center";
export const SETTINGS_BUTTON_ROW_WRAP_CLASS = "flex gap-[10px] items-center flex-wrap";
// `.settings-reset-confirm-row` replaces (not extends) the row's flex-direction
// and alignment, so it's written as one self-contained string rather than a
// `${SETTINGS_BUTTON_ROW_CLASS} ...` composition.
export const SETTINGS_RESET_CONFIRM_ROW_CLASS = "flex flex-col items-start gap-[10px] bg-[rgba(139,52,42,0.06)] border border-[rgba(139,52,42,0.18)] rounded-xl p-[12px_14px]";
export const SETTINGS_RESET_CONFIRM_MESSAGE_CLASS = "m-0 text-[0.9rem] font-semibold text-[var(--color-error-primary)]";
// Note: the old CSS also had a `.settings-reset-confirm-row .settings-button-row
// { flex-wrap: wrap; }` descendant rule, but no markup anywhere in the app
// actually nests a `.settings-button-row` inside a `.settings-reset-confirm-row`
// (they're always combined on the same element) — that rule was dead, so it's
// dropped here rather than ported.

export const SETTINGS_API_KEY_INLINE_ACTIONS_CLASS = "flex flex-wrap items-center gap-[10px]";
export const SETTINGS_API_KEY_INPUT_CLASS = `${SETTINGS_CONTROL_SELECT_CLASS} min-w-[min(320px,100%)] flex-[1_1_320px]`;
export const SETTINGS_API_KEY_BUTTON_ROW_CLASS = "flex flex-wrap gap-[10px] items-center";
export const SETTINGS_API_KEY_TEST_STATUS_CLASS = "flex flex-col gap-[6px] p-[12px_14px] rounded-[14px] bg-[rgba(31,127,92,0.09)] border border-[rgba(31,127,92,0.2)]";
export const SETTINGS_API_KEY_TEST_STATUS_LABEL_CLASS = "m-0 text-[0.76rem] uppercase tracking-[0.08em] text-[var(--color-green-primary)] font-bold";
export const SETTINGS_API_KEY_TEST_STATUS_MESSAGE_CLASS = "m-0 leading-[1.55] text-[var(--color-text-primary)] font-semibold";

export const BTN_SPINNER_CLASS = "inline-block w-[0.75em] h-[0.75em] mr-[0.4em] border-2 border-current border-t-transparent rounded-full [animation:spin_0.7s_linear_infinite] align-middle motion-reduce:[animation:none] motion-reduce:border-t-current motion-reduce:opacity-50";

// --- Remote planner privacy / consent UI design system ---
// Ported from the old `remote-planner-privacy.css` (used by
// remote-planner-privacy-ui.tsx and settings-panels/planner-privacy*.tsx).
// This subsystem deliberately uses OS forced-colors-mode system colors
// (Canvas, CanvasText, ButtonFace, ButtonText, Highlight) instead of the
// app's brand tokens, and CSS logical properties (margin-block,
// padding-inline-start, border-inline-start) instead of physical ones, so it
// renders correctly under Windows High Contrast Mode and RTL layouts. Block
// axis (vertical) properties are safely ported to physical mt-/mb-/pt-/pb-
// utilities since the block axis never flips for RTL; inline axis
// (horizontal) properties use Tailwind's logical ps-/border-s- utilities so
// RTL correctness is preserved.
const REMOTE_PRIVACY_CARD_BASE = "my-4 border-2 border-current rounded-xl p-4 bg-[var(--surface,Canvas)] text-[var(--text,CanvasText)]";
// The old CSS's `@media (max-width: 48rem)` breakpoint maps to Tailwind's
// `md` (48rem), not `sm` (40rem) -- kept explicit here since it's easy to
// reach for the more familiar `max-sm:` and silently narrow the breakpoint.
export const REMOTE_PRIVACY_STATUS_CLASS = `${REMOTE_PRIVACY_CARD_BASE} flex items-start justify-between gap-4 max-md:block`;
export const REMOTE_CONSENT_DIALOG_CLASS = `${REMOTE_PRIVACY_CARD_BASE} relative shadow-[0_0.5rem_2rem_rgb(0_0_0_/_0.25)] focus-visible:[outline:0.25rem_solid_Highlight] focus-visible:[outline-offset:0.2rem]`;

export const REMOTE_PRIVACY_STATUS_COPY_CLASS = "min-w-0";
export const REMOTE_PRIVACY_EYEBROW_CLASS = "mb-1 text-[0.85rem] font-bold tracking-[0.04em] uppercase";
// `.remote-privacy-status h2, .remote-consent-dialog h2, .remote-consent-dialog h3`
export const REMOTE_PRIVACY_HEADING_CLASS = "mt-1 mb-3";

export const REMOTE_PRIVACY_DETAIL_CLASS = "my-2";
export const REMOTE_PRIVACY_GUIDANCE_CLASS = "my-2";
const REMOTE_PRIVACY_INLINE_START_ACCENT = "border-s-[0.35rem] border-solid border-current ps-3";
export const REMOTE_PRIVACY_WARNING_CLASS = `my-2 ${REMOTE_PRIVACY_INLINE_START_ACCENT}`;
export const REMOTE_PRIVACY_ERROR_CLASS = `my-2 ${REMOTE_PRIVACY_INLINE_START_ACCENT}`;
export const REMOTE_PRIVACY_INLINE_STATUS_CLASS = REMOTE_PRIVACY_INLINE_START_ACCENT;

const REMOTE_PRIVACY_ACTION_BUTTON_BASE = "min-h-11 border-2 border-current rounded-lg p-[0.65rem_0.9rem] [font:inherit] font-bold bg-[ButtonFace] text-[ButtonText] cursor-pointer focus-visible:[outline:0.25rem_solid_Highlight] focus-visible:[outline-offset:0.2rem]";
export const REMOTE_PRIVACY_SETTINGS_BUTTON_CLASS = `${REMOTE_PRIVACY_ACTION_BUTTON_BASE} max-md:w-full max-md:mt-3`;
export const REMOTE_CONSENT_ACTION_BUTTON_CLASS = `${REMOTE_PRIVACY_ACTION_BUTTON_BASE} disabled:cursor-wait disabled:opacity-65`;
// `!important` in the old CSS beats `.remote-consent-actions button`'s higher
// specificity (a class + descendant type selector). Tailwind's trailing `!`
// modifier reproduces that unconditional win.
export const REMOTE_CONSENT_LOCAL_CANCEL_BUTTON_CLASS = "bg-[Canvas]! text-[CanvasText]!";

export const REMOTE_CONSENT_DESTINATION_COUNTS_CLASS = "grid gap-2 my-4";
export const REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS = "grid [grid-template-columns:minmax(8rem,0.35fr)_minmax(0,1fr)] gap-3 max-md:[grid-template-columns:1fr] max-md:gap-[0.15rem]";
export const REMOTE_CONSENT_DT_CLASS = "font-bold";
export const REMOTE_CONSENT_DD_CLASS = "m-0 [overflow-wrap:anywhere]";

export const REMOTE_CONSENT_DISCLOSURES_CLASS = "ps-6 [&>li:not(:first-child)]:mt-[0.35rem]";
export const REMOTE_CONSENT_ACTIONS_CLASS = "grid [grid-template-columns:repeat(auto-fit,minmax(13rem,1fr))] gap-3 mt-4";

export const REMOTE_CONSENT_ERROR_CLASS = `my-4 ${REMOTE_PRIVACY_INLINE_START_ACCENT}`;
export const REMOTE_CONSENT_ERROR_PARAGRAPH_CLASS = "my-[0.35rem]";

// The old markup combined `.settings-control-card` (flex, padding, radius,
// bg, border) with `.remote-privacy-settings-card` (grid, larger gap) on the
// same element — `remote-planner-privacy.css` loads after `styles.css`, so
// `display: grid`/`gap: 1rem` win over the settings-card's `flex`/`gap: 8px`
// at matching specificity. Tailwind compiles all utilities into one
// stylesheet in its own canonical order, so that source-file-load-order
// override isn't guaranteed to survive a straight class concatenation —
// this is written as one self-contained string with the already-resolved
// net result instead.
export const REMOTE_PRIVACY_SETTINGS_CARD_CLASS = "grid gap-4 p-[16px_18px] rounded-[18px] bg-[var(--color-surface-inner)] border border-[rgba(41,88,63,0.14)] [&_h3]:mt-0 [&_h4]:mt-0 [&_p]:mt-0";

export const REMOTE_PRIVACY_ROW_CLASS = "flex items-start justify-between gap-4 max-md:grid";

const REMOTE_PRIVACY_BOX_BASE = "border-2 border-current rounded-[0.65rem] p-4 bg-[Canvas] text-[CanvasText]";
export const REMOTE_PRIVACY_BOX_ACCENT_CLASS = `${REMOTE_PRIVACY_BOX_BASE} border-s-[0.45rem]`;
export const REMOTE_PRIVACY_LOOPBACK_STATUS_CLASS = REMOTE_PRIVACY_BOX_BASE;
export const REMOTE_PRIVACY_CURRENT_ORIGIN_CLASS = REMOTE_PRIVACY_BOX_BASE;
export const REMOTE_PRIVACY_RULE_MANAGEMENT_CLASS = REMOTE_PRIVACY_BOX_BASE;
export const REMOTE_PRIVACY_CLEAR_CONTROLS_CLASS = REMOTE_PRIVACY_BOX_BASE;
export const REMOTE_PRIVACY_SETTINGS_CONFIRMATION_CLASS = `${REMOTE_PRIVACY_BOX_BASE} relative shadow-[0_0.5rem_2rem_rgb(0_0_0_/_0.25)] focus-visible:[outline:0.25rem_solid_Highlight] focus-visible:[outline-offset:0.2rem]`;

export const REMOTE_PRIVACY_MODE_SELECTOR_CLASS = "grid gap-3 m-0 border-0 p-0";
export const REMOTE_PRIVACY_MODE_SELECTOR_LEGEND_CLASS = "mb-2 font-extrabold";
export const REMOTE_PRIVACY_MODE_OPTION_CLASS = "grid [grid-template-columns:auto_minmax(0,1fr)] gap-3 items-start border-2 border-current rounded-[0.65rem] p-[0.85rem] cursor-pointer focus-within:[outline:0.25rem_solid_Highlight] focus-within:[outline-offset:0.15rem]";
export const REMOTE_PRIVACY_MODE_OPTION_INPUT_CLASS = "w-5 h-5 mt-[0.15rem]";
export const REMOTE_PRIVACY_MODE_OPTION_LABEL_CLASS = "grid gap-1";

export const REMOTE_PRIVACY_RULE_LIST_CLASS = "grid gap-3 m-0 p-0 list-none";
export const REMOTE_PRIVACY_RULE_ITEM_CLASS = `${REMOTE_PRIVACY_ROW_CLASS} border border-current rounded-[0.6rem] p-[0.85rem]`;
export const REMOTE_PRIVACY_CODE_CLASS = "[overflow-wrap:anywhere]";

export const REMOTE_PRIVACY_MANUAL_RULE_CLASS = "mt-4 border-t border-current pt-4";
export const REMOTE_PRIVACY_MANUAL_RULE_SUMMARY_CLASS = "min-h-10 font-bold cursor-pointer focus-visible:[outline:0.25rem_solid_Highlight] focus-visible:[outline-offset:0.2rem]";

interface SettingsPanelSectionOptions {
  titleId: string;
  title: string;
  description: ReactNode;
  children: ReactNode | ReactNode[];
  error?: string | null;
  onDismissError?: () => void;
  onRetry?: () => void;
  eyebrow?: string;
  sectionClassName?: string;
}

interface SelectControlCardOptions {
  id: string;
  label: string;
  valueText: string;
  selectedValue: string;
  disabled?: boolean;
  dataAttributes: Record<string, string>;
  options: Array<{ value: string; label: string }>;
  onChange?: (value: string) => void;
}

interface CheckboxControlCardOptions {
  id: string;
  label: string;
  valueText: string;
  checked: boolean;
  disabled?: boolean;
  dataAttributes?: Record<string, string>;
  onChange?: (checked: boolean) => void;
}

export function renderReadOnlySettingText(value: string | number | null): string {
  if (value === null) {
    return "Not configured";
  }
  return `${value}`;
}

export function renderReadOnlyCard(label: string, value: string | number | null): ReactNode {
  return (
    <div className={CONTROL_CARD_READONLY_CLASS}>
      <span className={CONTROL_LABEL}>{label}</span>
      <span className={CONTROL_VALUE}>{renderReadOnlySettingText(value)}</span>
    </div>
  );
}

// Card anatomy shared by every top-level panel (url-input-panel, status-panel,
// audio-controls-panel, settings-panel, confirmation-panel in the pre-Tailwind
// CSS): identical border-radius/border/shadow, with per-caller overrides for
// margin, padding, and background passed via sectionClassName.
const PANEL_BASE = "rounded-[22px] [border:var(--card-border)] shadow-[0_18px_36px_rgba(49,63,74,0.08)]";
export const SETTINGS_PANEL_SECTION_CLASS = `mt-[18px] p-[22px_24px] bg-[var(--color-surface-card)] ${PANEL_BASE}`;

// Card grid shared by every settings subpage (asr/tts/planner/runtime), plus
// the "single column" and "compact top margin" variants a few panels need.
// Each variant is a complete, self-contained class string (rather than an
// additive override) so Tailwind's utility-ordering can't leave two
// conflicting margin-top/grid-template-columns values fighting for the win.
export const SETTINGS_GRID_CLASS = "grid [grid-template-columns:repeat(auto-fit,minmax(220px,1fr))] gap-[14px] mt-[18px]";
export const SETTINGS_GRID_SINGLE_CLASS = "grid [grid-template-columns:1fr] gap-[14px] mt-[18px]";
export const SETTINGS_GRID_SINGLE_COMPACT_CLASS = "grid [grid-template-columns:1fr] gap-[14px] mt-[14px]";

export function renderSettingsModelMissingWarning(message: string, onOpenRuntimeSettings?: () => void): ReactNode {
  return (
    <div className="mt-3 p-[12px_16px] bg-[rgba(122,87,39,0.08)] border border-[rgba(122,87,39,0.22)] rounded-xl flex flex-col items-start gap-2" role="alert">
      <p className="m-0 text-[0.9rem] font-medium text-[var(--color-amber-active)] leading-[1.4]">{message}</p>
      <button
        type="button"
        className="py-[6px] px-[14px] rounded-lg border border-[rgba(122,87,39,0.35)] bg-[rgba(122,87,39,0.12)] text-[var(--color-amber-active)] text-[0.875rem] font-semibold cursor-pointer font-[inherit] transition-colors duration-150 hover:bg-[rgba(122,87,39,0.2)]"
        data-open-runtime-settings="true"
        onClick={onOpenRuntimeSettings}
      >
        Open Advanced settings
      </button>
    </div>
  );
}

export function renderSettingsPanelSection({
  titleId,
  title,
  description,
  children,
  error = null,
  onDismissError,
  onRetry,
  eyebrow = "Settings",
  sectionClassName = SETTINGS_PANEL_SECTION_CLASS,
}: SettingsPanelSectionOptions): ReactNode {
  const childNodes = Array.isArray(children) ? children : [children];

  return (
    <section className={sectionClassName} aria-labelledby={titleId}>
      <div className="max-w-[60ch]">
        <p className="mb-2 uppercase tracking-[0.18em] text-[0.76rem] text-[var(--eyebrow-color)]">{eyebrow}</p>
        <h2 id={titleId} className="mb-[10px] [font-family:var(--font-display)] text-[clamp(1.1rem,2vw,1.4rem)] leading-[1.05]">{title}</h2>
        <p className="leading-[1.55]">{description}</p>
        {error ? (
          <p className="mt-2 leading-[1.55] text-[var(--color-error-primary)] font-semibold" role="alert">
            {error}
            {onRetry ? (
              <button
                type="button"
                className="appearance-none bg-transparent border-none pl-2 [font:inherit] text-[0.84rem] font-bold text-inherit cursor-pointer opacity-70 underline hover:opacity-100"
                onClick={onRetry}
                aria-label="Try again"
              >
                Try again
              </button>
            ) : null}
            {onDismissError ? (
              <button
                type="button"
                className="appearance-none bg-transparent border-none pl-2 [font:inherit] text-[0.84rem] font-bold text-inherit cursor-pointer opacity-70 underline hover:opacity-100"
                onClick={onDismissError}
                aria-label="Dismiss error"
              >
                Dismiss
              </button>
            ) : null}
          </p>
        ) : null}
      </div>
      {childNodes}
    </section>
  );
}

export function renderSelectControlCard({
  id,
  label,
  valueText,
  selectedValue,
  disabled = false,
  dataAttributes,
  options,
  onChange,
}: SelectControlCardOptions): ReactNode {
  return (
    <label className={CONTROL_CARD} htmlFor={id}>
      <span className={CONTROL_LABEL}>{label}</span>
      <span className={CONTROL_VALUE}>{valueText}</span>
      <select
        id={id}
        className={`w-full py-3 px-[14px] rounded-[14px] border border-[rgba(41,88,63,0.18)] bg-[var(--color-surface-inner)] text-[var(--color-text-primary)] [font:inherit] ${FOCUS_RING} focus-visible:border-[rgba(41,88,63,0.3)] disabled:cursor-progress disabled:opacity-60`}
        value={selectedValue}
        disabled={disabled || undefined}
        aria-disabled={disabled ? "true" : undefined}
        onChange={onChange ? (event) => { onChange(event.currentTarget.value); } : () => undefined}
        {...dataAttributes}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>{option.label}</option>
        ))}
      </select>
    </label>
  );
}

export function renderCheckboxControlCard({
  id,
  label,
  valueText,
  checked,
  disabled = false,
  dataAttributes = {},
  onChange,
}: CheckboxControlCardOptions): ReactNode {
  return (
    <label className={CONTROL_CARD} htmlFor={id}>
      <span className={CONTROL_LABEL}>{label}</span>
      <span className={CONTROL_VALUE}>{valueText}</span>
      <input
        id={id}
        className="w-full accent-[var(--color-green-primary)] disabled:cursor-progress disabled:opacity-60"
        type="checkbox"
        checked={checked || undefined}
        disabled={disabled || undefined}
        aria-disabled={disabled ? "true" : undefined}
        onChange={onChange ? (event) => { onChange(event.currentTarget.checked); } : undefined}
        {...dataAttributes}
      />
    </label>
  );
}
