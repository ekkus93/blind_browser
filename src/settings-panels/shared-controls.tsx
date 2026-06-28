import { type ReactNode } from "react";

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
    <div className="settings-control-card settings-control-card-readonly">
      <span className="settings-control-label">{label}</span>
      <span className="settings-control-value">{renderReadOnlySettingText(value)}</span>
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
  sectionClassName = "settings-panel",
}: SettingsPanelSectionOptions): ReactNode {
  const childNodes = Array.isArray(children) ? children : [children];

  return (
    <section className={sectionClassName} aria-labelledby={titleId}>
      <div className="settings-panel-copy">
        <p className="settings-panel-eyebrow">{eyebrow}</p>
        <h2 id={titleId}>{title}</h2>
        <p className="settings-panel-description">{description}</p>
        {error ? (
          <p className="settings-panel-error" role="alert">
            {error}
            {onRetry ? (
              <button type="button" className="panel-error-dismiss" onClick={onRetry} aria-label="Try again">
                Try again
              </button>
            ) : null}
            {onDismissError ? (
              <button type="button" className="panel-error-dismiss" onClick={onDismissError} aria-label="Dismiss error">
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
    <label className="settings-control-card" htmlFor={id}>
      <span className="settings-control-label">{label}</span>
      <span className="settings-control-value">{valueText}</span>
      <select
        id={id}
        className="settings-control-select"
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
    <label className="settings-control-card" htmlFor={id}>
      <span className="settings-control-label">{label}</span>
      <span className="settings-control-value">{valueText}</span>
      <input
        id={id}
        className="settings-control-input"
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
