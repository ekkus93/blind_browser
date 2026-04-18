import { createElement, type ReactNode } from "react";

const h = createElement;

interface SettingsPanelSectionOptions {
  titleId: string;
  title: string;
  description: ReactNode;
  children: ReactNode | ReactNode[];
  error?: string | null;
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
}

interface CheckboxControlCardOptions {
  id: string;
  label: string;
  valueText: string;
  checked: boolean;
  disabled?: boolean;
  dataAttributes?: Record<string, string>;
}

export function renderReadOnlySettingText(value: string | number | null): string {
  if (value === null) {
    return "Not configured";
  }

  return `${value}`;
}

export function renderReadOnlyCard(label: string, value: string | number | null): ReactNode {
  return h(
    "div",
    { className: "settings-control-card" },
    h("span", { className: "settings-control-label" }, label),
    h("span", { className: "settings-control-value" }, renderReadOnlySettingText(value)),
  );
}

export function renderSettingsPanelSection({
  titleId,
  title,
  description,
  children,
  error = null,
  eyebrow = "Settings",
  sectionClassName = "settings-panel",
}: SettingsPanelSectionOptions): ReactNode {
  const childNodes = Array.isArray(children) ? children : [children];

  return h(
    "section",
    { className: sectionClassName, "aria-labelledby": titleId },
    h(
      "div",
      { className: "settings-panel-copy" },
      h("p", { className: "settings-panel-eyebrow" }, eyebrow),
      h("h2", { id: titleId }, title),
      h("p", { className: "settings-panel-description" }, description),
      error ? h("p", { className: "settings-panel-error", role: "alert" }, error) : null,
    ),
    ...childNodes,
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
}: SelectControlCardOptions): ReactNode {
  return h(
    "label",
    { className: "settings-control-card", htmlFor: id },
    h("span", { className: "settings-control-label" }, label),
    h("span", { className: "settings-control-value" }, valueText),
    h(
      "select",
      {
        id,
        className: "settings-control-select",
        value: selectedValue,
        disabled: disabled || undefined,
        "aria-disabled": disabled ? "true" : undefined,
        onChange: () => undefined,
        ...dataAttributes,
      },
      ...options.map((option) => h("option", { value: option.value, key: option.value }, option.label)),
    ),
  );
}

export function renderCheckboxControlCard({
  id,
  label,
  valueText,
  checked,
  disabled = false,
  dataAttributes = {},
}: CheckboxControlCardOptions): ReactNode {
  return h(
    "label",
    { className: "settings-control-card", htmlFor: id },
    h("span", { className: "settings-control-label" }, label),
    h("span", { className: "settings-control-value" }, valueText),
    h("input", {
      id,
      className: "settings-control-input",
      type: "checkbox",
      checked: checked || undefined,
      disabled: disabled || undefined,
      "aria-disabled": disabled ? "true" : undefined,
      readOnly: true,
      ...dataAttributes,
    }),
  );
}