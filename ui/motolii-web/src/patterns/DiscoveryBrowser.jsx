import { useEffect, useRef, useState } from "react";

function classes(...names) {
  return names.filter(Boolean).join(" ");
}

export function DiscoveryViewToggle({ label, options }) {
  return (
    <span className="plugin-view-toggle" aria-label={label}>
      {options.map(({ active = false, ...option }) => (
        <button
          {...option}
          key={option["aria-label"]}
          className={classes(active && "on", option.className)}
        />
      ))}
    </span>
  );
}

export function DiscoverySearchBar({
  inputId,
  inputLabel,
  placeholder = "Search",
  actions,
  children,
}) {
  return (
    <div className="candidate-search-row">
      <input
        className="search"
        id={inputId}
        type="search"
        placeholder={placeholder}
        aria-label={inputLabel}
      />
      {actions}
      {children}
    </div>
  );
}

export function DiscoverySection({ title, children }) {
  return (
    <>
      <div className="candidate-nav-title">{title}</div>
      {children}
    </>
  );
}

export function DiscoverySourceRail({ label, children }) {
  return (
    <nav className="candidate-browser-nav" aria-label={label}>
      {children}
    </nav>
  );
}

export function DiscoveryBrowserLayout({
  rail,
  children,
  hierarchyWidth = 106,
  onHierarchyWidthChange,
  onHierarchyRestore,
}) {
  const drag = useRef(null);
  const [dragging, setDragging] = useState(false);
  const hierarchyHidden = hierarchyWidth === 0;

  const resize = (rawWidth, { deferClose = false } = {}) => {
    if (!onHierarchyWidthChange) {
      return;
    }
    const nextWidth =
      rawWidth < 48
        ? 0
        : Math.round(Math.max(72, Math.min(220, rawWidth)));
    if (drag.current) {
      drag.current.pendingClose = nextWidth === 0;
    }
    if (deferClose && nextWidth === 0) {
      onHierarchyWidthChange(72);
      return;
    }
    onHierarchyWidthChange(nextWidth);
  };

  const finish = () => {
    drag.current = null;
    setDragging(false);
  };

  useEffect(() => {
    if (!dragging) {
      return undefined;
    }
    const handlePointerMove = (event) => {
      if (!drag.current) {
        return;
      }
      resize(
        drag.current.startWidth +
          event.clientX -
          drag.current.startX,
        { deferClose: true },
      );
    };
    const handlePointerUp = () => {
      const shouldClose = drag.current?.pendingClose;
      finish();
      if (shouldClose) {
        onHierarchyWidthChange?.(0);
      }
    };
    const handlePointerCancel = () => {
      if (drag.current) {
        onHierarchyWidthChange?.(drag.current.startWidth);
      }
      finish();
    };
    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);
    window.addEventListener("pointercancel", handlePointerCancel);
    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
      window.removeEventListener("pointercancel", handlePointerCancel);
    };
  }, [dragging]);

  const handleKeyDown = (event) => {
    if (event.key === "Home") {
      event.preventDefault();
      onHierarchyWidthChange?.(106);
      return;
    }
    if (!["ArrowLeft", "ArrowRight"].includes(event.key)) {
      return;
    }
    event.preventDefault();
    const step = event.shiftKey ? 48 : 16;
    resize(
      hierarchyWidth +
        (event.key === "ArrowLeft" ? -step : step),
    );
  };

  return (
    <div
      className={`candidate-browser-layout${hierarchyHidden ? " is-hierarchy-hidden" : ""}`}
      style={{
        "--candidate-hierarchy-size": `${hierarchyWidth}px`,
      }}
    >
      {rail}
      {hierarchyHidden ? (
        <button
          type="button"
          className="candidate-rail-toggle"
          aria-label="ブラウザのフォルダ階層を表示"
          title="Show hierarchy"
          onClick={onHierarchyRestore}
        >
          <span aria-hidden="true">›</span>
          <small aria-hidden="true">HIERARCHY</small>
        </button>
      ) : onHierarchyWidthChange ? (
        <button
          type="button"
          className={`candidate-rail-resizer${dragging ? " is-dragging" : ""}`}
          role="separator"
          aria-label="ブラウザのフォルダ階層の横幅を変更"
          aria-orientation="vertical"
          aria-valuemin="0"
          aria-valuemax="220"
          aria-valuenow={hierarchyWidth}
          title="Drag left to close"
          onDoubleClick={() => onHierarchyWidthChange(106)}
          onKeyDown={handleKeyDown}
          onPointerDown={(event) => {
            event.preventDefault();
            drag.current = {
              startX: event.clientX,
              startWidth: hierarchyWidth,
              pendingClose: false,
            };
            setDragging(true);
          }}
        />
      ) : null}
      {children}
    </div>
  );
}

export function DiscoveryResults({
  id,
  className,
  title,
  titleId,
  scope,
  scopeId,
  count,
  countId,
  headerChildren,
  children,
}) {
  return (
    <section className={classes("candidate-results", className)} id={id}>
      <header className="candidate-results-head">
        <strong id={titleId}>{title}</strong>
        {scope ? <span id={scopeId}>{scope}</span> : null}
        {headerChildren}
        {count !== undefined ? <span id={countId}>{count}</span> : null}
      </header>
      {children}
    </section>
  );
}
