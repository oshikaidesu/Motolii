import "./primitives.css";

function classes(...names) {
  return names.filter(Boolean).join(" ");
}

export function Button({
  variant = "default",
  active = false,
  pressed,
  className,
  type = "button",
  children,
  ...buttonProps
}) {
  return (
    <button
      {...buttonProps}
      type={type}
      className={classes(
        "mock-button",
        variant !== "default" && `mock-button--${variant}`,
        className,
      )}
      data-active={active ? "true" : undefined}
      aria-pressed={pressed}
    >
      {children}
    </button>
  );
}

export function Icon({ glyph, label, className, ...spanProps }) {
  const accessibilityProps = label
    ? { role: "img", "aria-label": label }
    : { "aria-hidden": true };

  return (
    <span
      {...spanProps}
      {...accessibilityProps}
      className={classes("mock-icon", className)}
    >
      {glyph}
    </span>
  );
}

export function IconButton({
  label,
  glyph,
  active = false,
  pressed,
  className,
  type = "button",
  children,
  ...buttonProps
}) {
  return (
    <button
      {...buttonProps}
      type={type}
      className={classes("mock-icon-button", className)}
      data-active={active ? "true" : undefined}
      aria-label={label}
      aria-pressed={pressed}
    >
      {children ?? <Icon glyph={glyph} />}
    </button>
  );
}

export function TabList({ label, className, children, ...listProps }) {
  return (
    <div
      {...listProps}
      className={classes("mock-tabs", className)}
      role="tablist"
      aria-label={label}
    >
      {children}
    </div>
  );
}

export function Tab({
  selected = false,
  className,
  type = "button",
  children,
  ...buttonProps
}) {
  return (
    <button
      {...buttonProps}
      type={type}
      className={classes("mock-tab", className)}
      role="tab"
      aria-selected={selected}
      tabIndex={selected ? 0 : -1}
    >
      {children}
    </button>
  );
}

const panelWayfindingRoles = {
  project: "var(--mock-role-way-project)",
  files: "var(--mock-role-way-files)",
  plugins: "var(--mock-role-way-plugins)",
  stage: "var(--mock-role-way-stage)",
  inspector: "var(--mock-role-way-inspector)",
  timeline: "var(--mock-role-way-timeline)",
};

export function PanelHeader({
  title,
  detail,
  wayfinding = "plugins",
  actions,
  headingLevel = 2,
  className,
  children,
  ...headerProps
}) {
  const Heading = `h${headingLevel}`;
  const markerColor =
    panelWayfindingRoles[wayfinding] ?? panelWayfindingRoles.plugins;

  return (
    <header
      {...headerProps}
      className={classes("mock-panel-header", className)}
      style={{
        "--mock-panel-wayfinding": markerColor,
        ...headerProps.style,
      }}
    >
      <span className="mock-panel-header__marker" aria-hidden="true" />
      <Heading
        style={{ margin: 0, font: "inherit", fontWeight: "inherit" }}
      >
        {title}
      </Heading>
      {detail && <small className="mock-panel-header__detail">{detail}</small>}
      {children}
      {actions && (
        <span className="mock-panel-header__actions">{actions}</span>
      )}
    </header>
  );
}

export function Field({
  label,
  hint,
  inline = false,
  className,
  id,
  ...inputProps
}) {
  return (
    <label
      className={classes(
        "mock-field-label",
        inline && "mock-field-label--inline",
      )}
    >
      <span>{label}</span>
      <input
        {...inputProps}
        id={id}
        className={classes("mock-field", className)}
      />
      {hint && <small className="mock-field__hint">{hint}</small>}
    </label>
  );
}
