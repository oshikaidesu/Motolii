import {
  createContext,
  createElement,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  attributesToProps,
  domToReact,
} from "html-react-parser";
import {
  DiscoveryBrowserLayout as SharedDiscoveryBrowserLayout,
  DiscoveryResults,
  DiscoverySearchBar,
  DiscoverySection,
  DiscoverySourceRail,
  DiscoveryViewToggle,
} from "../patterns/DiscoveryBrowser.jsx";
import "./discovery-browser-candidate.css";

const BrowserHierarchyContext = createContext(null);
const EFFECT_TAGS = [
  { id: "go-to", label: "Go-to", icon: "◎" },
  { id: "atmosphere", label: "Atmosphere", icon: "◌" },
  { id: "kinetic", label: "Kinetic", icon: "⌁" },
  { id: "review", label: "Review", icon: "✓" },
];
const CREATE_TAGS = [
  { id: "layout", label: "Layout", icon: "▱" },
  { id: "brand-kit", label: "Brand kit", icon: "Aa" },
  { id: "animated", label: "Animated", icon: "⌁" },
  { id: "prototype", label: "Prototype", icon: "◇" },
];

function DiscoveryBrowserLayout(props) {
  const hierarchy = useContext(BrowserHierarchyContext);
  return (
    <SharedDiscoveryBrowserLayout
      {...props}
      hierarchyWidth={hierarchy?.width}
      onHierarchyWidthChange={hierarchy?.setWidth}
      onHierarchyRestore={hierarchy?.restore}
    />
  );
}

function BrowserThumbnailSizeSettingBridge({ onChange }) {
  useEffect(() => {
    const input = document.querySelector("#plugin-thumb-size");
    const label = input?.closest(".setting-row")?.querySelector("label");
    if (!input || !label) {
      return undefined;
    }
    const previousLabel = label.textContent;
    const previousAriaLabel = input.getAttribute("aria-label");
    const sync = () => onChange(Number(input.value));
    label.textContent = "Browser thumbnail size";
    input.setAttribute("aria-label", "Browser thumbnail size");
    input.addEventListener("input", sync);
    const frame = window.requestAnimationFrame(sync);
    return () => {
      window.cancelAnimationFrame(frame);
      input.removeEventListener("input", sync);
      label.textContent = previousLabel;
      if (previousAriaLabel) {
        input.setAttribute("aria-label", previousAriaLabel);
      } else {
        input.removeAttribute("aria-label");
      }
    };
  }, [onChange]);
  return null;
}

function showFixtureStatus(title, body) {
  const titleNode = document.querySelector("#status-title");
  const bodyNode = document.querySelector("#status-body");
  const keyNode = document.querySelector("#status-key");
  if (titleNode) titleNode.textContent = title;
  if (bodyNode) bodyNode.textContent = body;
  if (keyNode) keyNode.textContent = "";
}

function setFixtureResultView(rootSelector, controlSelector, view, copy) {
  const root = document.querySelector(rootSelector);
  if (root) root.dataset.view = view;
  document.querySelectorAll(controlSelector).forEach((control) => {
    control.classList.toggle(
      "on",
      control.dataset.elementView === view,
    );
  });
  showFixtureStatus("Create", copy);
}

function useCandidateTags(itemIds, initialAssignments) {
  const [activeTag, setActiveTag] = useState("");
  const [assignments, setAssignments] = useState(initialAssignments);

  const assign = (itemId, tagId) => {
    if (!itemIds.includes(itemId)) {
      return;
    }
    setAssignments((current) => ({
      ...current,
      [itemId]: Array.from(
        new Set([...(current[itemId] ?? []), tagId]),
      ),
    }));
  };

  return {
    activeTag,
    setActiveTag,
    assign,
    tagsFor(itemId) {
      return assignments[itemId] ?? [];
    },
    count(tagId) {
      return itemIds.filter((itemId) =>
        (assignments[itemId] ?? []).includes(tagId),
      ).length;
    },
    isVisible(itemId) {
      return (
        !activeTag ||
        (assignments[itemId] ?? []).includes(activeTag)
      );
    },
  };
}

function CandidateTagSection({ scope, tags, tagging }) {
  return (
    <DiscoverySection title="Tags">
      <div
        className="candidate-nav-group candidate-organization-tags"
        role="group"
        aria-label={`${scope} tags`}
      >
        {tags.map((tag) => (
          <button
            type="button"
            key={tag.id}
            className={tagging.activeTag === tag.id ? "on" : ""}
            data-candidate-tag={tag.id}
            aria-pressed={tagging.activeTag === tag.id}
            title={`Filter by ${tag.label}; drop an item to classify`}
            onClick={() =>
              tagging.setActiveTag(
                tagging.activeTag === tag.id ? "" : tag.id,
              )
            }
            onDragOver={(event) => {
              if (
                event.dataTransfer.types.includes(
                  "application/x-motolii-browser-item",
                )
              ) {
                event.preventDefault();
                event.dataTransfer.dropEffect = "copy";
                event.currentTarget.classList.add("drop-ready");
              }
            }}
            onDragLeave={(event) =>
              event.currentTarget.classList.remove("drop-ready")
            }
            onDrop={(event) => {
              event.preventDefault();
              event.currentTarget.classList.remove("drop-ready");
              tagging.assign(
                event.dataTransfer.getData(
                  "application/x-motolii-browser-item",
                ),
                tag.id,
              );
            }}
          >
            <i aria-hidden="true">{tag.icon}</i>
            <span>{tag.label}</span>
            <b>{tagging.count(tag.id)}</b>
          </button>
        ))}
      </div>
    </DiscoverySection>
  );
}

function PluginCard({
  itemId,
  mode,
  folder,
  labels,
  search,
  thumbnail,
  kind,
  name,
  category,
  subtype,
  state,
  pack,
  identity,
  impact,
  motion = false,
  selected = false,
  tags = [],
  tagVisible = true,
  onSelect,
}) {
  return (
    <div
      className={`vism candidate-plugin-card${selected ? " is-selected" : ""}`}
      data-browser-item={itemId}
      data-tags={tags.join(" ")}
      data-tag-visible={tagVisible}
      data-mode={mode}
      data-folder={folder}
      data-labels={labels}
      data-search={search}
      data-plugin-name={name}
      data-plugin-kind={kind}
      data-pack={pack}
      data-item-identity={identity}
      data-preview={motion ? "motion" : "poster"}
      draggable
      onDragStart={(event) =>
        event.dataTransfer.setData(
          "application/x-motolii-browser-item",
          itemId,
        )
      }
    >
      <button
        className="candidate-plugin-card-main"
        aria-label={`${name}${state ? ` · ${state}` : ""}`}
        aria-pressed={selected}
        onClick={onSelect}
      >
        <span className={`plugin-thumb ${thumbnail}`}>
          <span className="candidate-kind" aria-hidden="true">{kind}</span>
          {state ? (
            <span className={`thumb-state state ${mode}`}>{state}</span>
          ) : null}
          {impact ? <span className="candidate-impact">{impact}</span> : null}
          {motion ? <span className="candidate-motion-mark" aria-hidden="true">▶</span> : null}
        </span>
        <span className="candidate-card-name">
          <b>{name}</b>
          {tags.length ? (
            <small className="candidate-item-tags">
              {tags.map((tag) => `#${tag}`).join(" ")}
            </small>
          ) : null}
        </span>
      </button>
      <nav
        className="candidate-card-taxonomy"
        aria-label={`${name} type`}
      >
        <button data-plugin-type={category.value}>{category.label}</button>
        <span aria-hidden="true">›</span>
        <button data-plugin-type={subtype.value}>{subtype.label}</button>
      </nav>
    </div>
  );
}

function CandidatePluginBrowser() {
  const itemIds = ["echo-bloom", "type-pulse", "fold-field"];
  const tagging = useCandidateTags(itemIds, {
    "echo-bloom": ["go-to", "atmosphere"],
    "type-pulse": ["kinetic"],
    "fold-field": ["review"],
  });
  const [selectedItem, setSelectedItem] = useState("echo-bloom");

  return (
    <div
      id="vism-browser"
      className="candidate-plugin-browser"
      data-view="thumb"
      style={{
        "--plugin-thumb-size":
          "var(--browser-thumbnail-size, 80px)",
      }}
      data-info="Effects Browser|Apply visual results to an existing object or Timeline bar"
    >
      <DiscoverySearchBar
        inputId="search"
        inputLabel="Search effects"
        actions={(
          <DiscoveryViewToggle
            label="Result view"
            options={[
              {
                "data-plugin-view": "visual",
                "aria-label": "Thumbnail-only view",
                children: "▦",
              },
              {
                active: true,
                "data-plugin-view": "thumb",
                "aria-label": "Thumbnail and name view",
                children: "▤",
              },
              {
                "data-plugin-view": "detail",
                "aria-label": "List view",
                children: "☷",
              },
            ]}
          />
        )}
      />

      <DiscoveryBrowserLayout>
        <DiscoverySourceRail label="Effect sources">
          <div className="candidate-nav-group">
            <button className="on" data-plugin-source="all">
              <i aria-hidden="true">▦</i><span>All</span>
            </button>
            <button data-plugin-source="project">
              <i aria-hidden="true">◇</i><span>Used</span>
            </button>
            <button id="plugin-recent-toggle">
              <i aria-hidden="true">↺</i><span>Recent</span>
            </button>
          </div>
          <DiscoverySection title="Collections">
            <div className="candidate-nav-group">
              <button data-plugin-collection="motion">
                <i aria-hidden="true">◎</i><span>Favorites</span>
              </button>
              <button data-plugin-collection="type">
                <i aria-hidden="true">Aa</i><span>Type</span>
              </button>
            </div>
          </DiscoverySection>
          <CandidateTagSection
            scope="Effect"
            tags={EFFECT_TAGS}
            tagging={tagging}
          />
          <DiscoverySection title="Packs">
            <div className="candidate-nav-group">
              <button data-pack-select="motion-kit-alpha">
                <i aria-hidden="true">▤</i><span>Motion Kit α</span>
              </button>
            </div>
          </DiscoverySection>
          <button
            className="candidate-save-effect"
            id="save-effect-item"
          >
            ＋ Save current…
          </button>
          <select id="plugin-folder" aria-label="Plugin collection" hidden>
            <option value="all">All</option>
            <option value="motion">Favorites</option>
            <option value="type">Type</option>
            <option value="experimental">Experimental</option>
            <option value="project">Project used</option>
          </select>
        </DiscoverySourceRail>

        <DiscoveryResults
          id="plugin-results"
          title="Results"
          count="3"
          countId="plugin-result-count"
          headerChildren={(
            <button
              className="candidate-taxonomy-clear"
              id="plugin-taxonomy-clear"
              aria-label="Clear effect type filter"
              hidden
            />
          )}
        >
          <div className="plugin-grid candidate-plugin-grid">
            <PluginCard
              itemId="echo-bloom"
              mode="installed"
              folder="motion project"
              labels="goto effect glow"
              search="echo bloom light pulse glow effect installed"
              thumbnail="bloom"
              kind="FX"
              name="Echo Bloom"
              category={{ value: "effect", label: "Effect" }}
              subtype={{ value: "glow", label: "Light" }}
              pack="motion-kit-alpha"
              motion
              selected={selectedItem === "echo-bloom"}
              tags={tagging.tagsFor("echo-bloom")}
              tagVisible={tagging.isVisible("echo-bloom")}
              onSelect={() => setSelectedItem("echo-bloom")}
            />
            <PluginCard
              itemId="type-pulse"
              mode="installed"
              folder="type"
              labels="effect text motion"
              search="type pulse kinetic text motion effect"
              thumbnail="glyph"
              kind="FX"
              name="Type Pulse"
              category={{ value: "effect", label: "Effect" }}
              subtype={{ value: "text", label: "Typography" }}
              pack="motion-kit-alpha"
              identity="motion-kit.type-pulse"
              impact="◆ 12 KEYS"
              motion
              selected={selectedItem === "type-pulse"}
              tags={tagging.tagsFor("type-pulse")}
              tagVisible={tagging.isVisible("type-pulse")}
              onSelect={() => setSelectedItem("type-pulse")}
            />
            <PluginCard
              itemId="fold-field"
              mode="blocked"
              folder="experimental"
              labels="effect space"
              search="fold field space geometry effect incompatible"
              thumbnail="fold"
              kind="FX"
              name="Fold Field"
              category={{ value: "effect", label: "Effect" }}
              subtype={{ value: "space", label: "Spatial" }}
              state="Unavailable"
              selected={selectedItem === "fold-field"}
              tags={tagging.tagsFor("fold-field")}
              tagVisible={tagging.isVisible("fold-field")}
              onSelect={() => setSelectedItem("fold-field")}
            />
          </div>
        </DiscoveryResults>

        <section
          className="candidate-recent"
          id="plugin-recent-panel"
          aria-label="Recent plugins"
          hidden
        >
          <header className="candidate-results-head">
            <strong>Recent</strong>
          </header>
          <div className="plugin-history-items" id="plugin-history" />
        </section>
      </DiscoveryBrowserLayout>
    </div>
  );
}

function ElementCard({
  itemId,
  element,
  name,
  type,
  provider,
  glyph,
  thumbnail,
  pack,
  state,
  identity,
  impact,
  motion = false,
  selected = false,
  tags = [],
  tagVisible = true,
  onSelect,
}) {
  return (
    <button
      className={`candidate-element-card${selected ? " on" : ""}`}
      data-browser-item={itemId}
      data-tags={tags.join(" ")}
      data-tag-visible={tagVisible}
      data-element={element}
      data-element-name={name}
      data-element-type={type.toLowerCase()}
      data-element-provider={provider.toLowerCase().replaceAll(" ", "-")}
      data-element-state={state}
      data-pack={pack}
      data-item-identity={identity}
      data-preview={motion ? "motion" : "poster"}
      data-search={`${name} ${type} ${provider}`.toLowerCase()}
      draggable
      aria-label={`${name} · ${type} · ${provider}`}
      onClick={onSelect}
      onDragStart={(event) =>
        event.dataTransfer.setData(
          "application/x-motolii-browser-item",
          itemId,
        )
      }
    >
      <span className={`candidate-element-preview ${thumbnail}`}>
        <i aria-hidden="true">{glyph}</i>
        <small>{provider}</small>
        {state ? <em className="thumb-state state missing">{state}</em> : null}
        {impact ? <em className="candidate-impact">{impact}</em> : null}
        {motion ? <em className="candidate-motion-mark" aria-hidden="true">▶</em> : null}
      </span>
      <span className="candidate-element-name">
        <b>{name}</b>
        <small>{type}</small>
        {tags.length ? (
          <small className="candidate-item-tags">
            {tags.map((tag) => `#${tag}`).join(" ")}
          </small>
        ) : null}
      </span>
    </button>
  );
}

function CandidateCreateBrowser() {
  const itemIds = [
    "rectangle",
    "ellipse",
    "text",
    "solid",
    "glyph-current",
    "type-pulse",
    "ribbon-array",
    "particle-field",
  ];
  const tagging = useCandidateTags(itemIds, {
    rectangle: ["layout"],
    ellipse: [],
    text: ["brand-kit"],
    solid: ["brand-kit"],
    "glyph-current": ["animated"],
    "type-pulse": ["brand-kit", "animated"],
    "ribbon-array": ["prototype"],
    "particle-field": ["animated"],
  });
  const [selectedItem, setSelectedItem] = useState("rectangle");
  const elementProps = (itemId) => ({
    itemId,
    selected: selectedItem === itemId,
    tags: tagging.tagsFor(itemId),
    tagVisible: tagging.isVisible(itemId),
    onSelect: () => setSelectedItem(itemId),
  });

  return (
    <div
      className="candidate-elements-browser"
      id="elements-browser"
      data-view="grid"
      hidden
      data-info="Create Browser|Browse every registered item that creates a Stage or Timeline object"
    >
      <DiscoverySearchBar
        inputId="element-search"
        inputLabel="Search create items"
        actions={(
          <DiscoveryViewToggle
            label="Create result view"
            options={[
              {
                "data-element-view": "visual",
                "aria-label": "Create thumbnail-only view",
                children: "▦",
                onClick: () =>
                  setFixtureResultView(
                    "#elements-browser",
                    "[data-element-view]",
                    "visual",
                    "Thumbnail only",
                  ),
              },
              {
                active: true,
                "data-element-view": "grid",
                "aria-label": "Create thumbnail and name view",
                children: "▤",
                onClick: () =>
                  setFixtureResultView(
                    "#elements-browser",
                    "[data-element-view]",
                    "grid",
                    "Thumbnail + name",
                  ),
              },
              {
                "data-element-view": "list",
                "aria-label": "Create list view",
                children: "☷",
                onClick: () =>
                  setFixtureResultView(
                    "#elements-browser",
                    "[data-element-view]",
                    "list",
                    "List",
                  ),
              },
            ]}
          />
        )}
      />

      <DiscoveryBrowserLayout>
        <DiscoverySourceRail label="Create sources">
          <div className="candidate-nav-group">
            <button className="on" data-element-filter="all">
              <i aria-hidden="true">▦</i><span>All</span>
            </button>
            <button data-element-filter="recent">
              <i aria-hidden="true">↺</i><span>Recent</span>
            </button>
          </div>
          <DiscoverySection title="Type">
            <div className="candidate-nav-group">
              <button data-element-type-filter="shape">
                <i aria-hidden="true">○</i><span>Shapes</span>
              </button>
              <button data-element-type-filter="layer">
                <i aria-hidden="true">▱</i><span>Layers</span>
              </button>
              <button data-element-type-filter="generator">
                <i aria-hidden="true">✣</i><span>Generators</span>
              </button>
            </div>
          </DiscoverySection>
          <DiscoverySection title="Provider">
            <div className="candidate-nav-group candidate-provider-list">
              <button data-element-provider-filter="built-in">
                <i aria-hidden="true">M</i><span>Built-in</span>
              </button>
              <button data-element-provider-filter="orbit-forge">
                <i aria-hidden="true">O</i><span>Orbit Forge</span>
              </button>
            </div>
          </DiscoverySection>
          <CandidateTagSection
            scope="Create"
            tags={CREATE_TAGS}
            tagging={tagging}
          />
          <DiscoverySection title="Packs">
            <div className="candidate-nav-group">
              <button data-pack-select="motion-kit-alpha">
                <i aria-hidden="true">▤</i><span>Motion Kit α</span>
              </button>
            </div>
          </DiscoverySection>
        </DiscoverySourceRail>

        <DiscoveryResults
          id="element-results"
          title="All Create items"
          titleId="element-result-title"
          scope="REGISTERED PROVIDERS"
          scopeId="element-result-scope"
          count="8"
          countId="element-result-count"
        >
          <div className="candidate-element-grid">
            <ElementCard {...elementProps("rectangle")} element="rectangle" name="Rectangle" type="Shape" provider="Built-in" glyph="□" thumbnail="rectangle" />
            <ElementCard {...elementProps("ellipse")} element="ellipse" name="Ellipse" type="Shape" provider="Built-in" glyph="○" thumbnail="ellipse" />
            <ElementCard {...elementProps("text")} element="text" name="Text" type="Layer" provider="Built-in" glyph="T" thumbnail="text" />
            <ElementCard {...elementProps("solid")} element="solid" name="Solid" type="Layer" provider="Built-in" glyph="■" thumbnail="solid" />
            <ElementCard {...elementProps("glyph-current")} element="glyph-current" name="Glyph Current" type="Generator" provider="Motion Kit" glyph="G" thumbnail="glyph" pack="motion-kit-alpha" motion />
            <ElementCard {...elementProps("type-pulse")} element="type-pulse" name="Type Pulse" type="Text" provider="Motion Kit" glyph="T" thumbnail="text" pack="motion-kit-alpha" identity="motion-kit.type-pulse" impact="▱ + ◆ 12" motion />
            <ElementCard {...elementProps("ribbon-array")} element="ribbon-array" name="Ribbon Array" type="Generator" provider="Motion Kit" glyph="≋" thumbnail="ribbon" pack="motion-kit-alpha" state="Missing" />
            <ElementCard {...elementProps("particle-field")} element="particle-field" name="Particle Field" type="Generator" provider="Orbit Forge" glyph="✣" thumbnail="particles" motion />
          </div>
        </DiscoveryResults>
      </DiscoveryBrowserLayout>

      <footer className="candidate-elements-foot">
        <span><b>8</b> registered</span>
        <span><b>3</b> providers</span>
        <span>D&amp;D or double-click</span>
        <button id="save-create-item">Save…</button>
      </footer>

      <menu
        className="candidate-context-menu"
        id="element-context-menu"
        role="menu"
        aria-label="Create item commands"
        hidden
      >
        <button data-element-context-command="add">Add to Stage</button>
        <button data-element-context-command="timeline">Add at playhead</button>
        <hr />
        <button data-element-context-command="favorite">Add to Favorites</button>
        <button data-element-context-command="provider">Show provider</button>
      </menu>
    </div>
  );
}

function CandidateBrowserTabs() {
  return (
    <>
      <div className="browser-tabs">
        <button className="browser-tab" data-tab="project">Media</button>
        <button className="browser-tab on" data-tab="effects">Effects</button>
        <button className="browser-tab" data-tab="create">Create</button>
      </div>
      <div className="candidate-pack-scope" id="candidate-pack-scope" hidden>
        <button className="candidate-pack-identity" id="candidate-pack-open">
          <i aria-hidden="true">▤</i>
          <span><b>Motion Kit α</b><small>ONE PACK · THREE USES</small></span>
        </button>
        <nav aria-label="Motion Kit α contents">
          <button data-pack-tab="project">Media <b>2</b></button>
          <button data-pack-tab="create">Create <b>3</b></button>
          <button data-pack-tab="effects">Effects <b>2</b></button>
        </nav>
        <button id="candidate-pack-clear" aria-label="Clear pack scope">×</button>
      </div>
      <CandidateCreateBrowser />
      <section
        className="candidate-save-sheet"
        id="candidate-save-sheet"
        role="dialog"
        aria-modal="true"
        aria-label="Save to Browser"
        hidden
      >
        <header>
          <strong>Save to Browser</strong>
          <button id="candidate-save-close" aria-label="Close save dialog">×</button>
        </header>
        <label>
          <span>Name</span>
          <input id="candidate-save-name" defaultValue="Pulse treatment" />
        </label>
        <div className="candidate-save-preview">
          <span className="candidate-save-preview-stage" id="candidate-save-preview-stage">
            <i>▶</i>
          </span>
          <div>
            <strong>Preview</strong>
            <small id="candidate-preview-copy">Auto · 2s around playhead</small>
          </div>
        </div>
        <div className="candidate-preview-options" aria-label="Preview source">
          <button className="on" data-preview-choice="auto">Auto</button>
          <button data-preview-choice="record">Record 2s</button>
          <button data-preview-choice="file">GIF / Video</button>
          <button data-preview-choice="frame">Current frame</button>
        </div>
        <section className="candidate-preview-range" id="candidate-preview-range" aria-label="Preview range">
          <header>
            <strong>Preview range</strong>
            <output id="candidate-range-duration">2.0s</output>
          </header>
          <div className="candidate-range-track">
            <span className="candidate-range-film" aria-hidden="true" />
            <span className="candidate-range-selection" aria-hidden="true" />
            <input
              id="candidate-range-in"
              type="range"
              min="0"
              max="100"
              defaultValue="25"
              aria-label="Preview in"
            />
            <input
              id="candidate-range-out"
              type="range"
              min="0"
              max="100"
              defaultValue="75"
              aria-label="Preview out"
            />
            <input
              id="candidate-range-poster"
              type="range"
              min="0"
              max="100"
              defaultValue="52"
              aria-label="Poster frame"
            />
          </div>
          <div className="candidate-range-meta">
            <span id="candidate-range-in-copy">00:53.2</span>
            <button id="candidate-range-play" aria-label="Loop preview">▶ Loop</button>
            <span id="candidate-range-out-copy">00:55.2</span>
          </div>
          <footer>
            <small>Drag In / Out · Playhead sets poster</small>
            <button id="candidate-generate-preview">Generate preview</button>
          </footer>
        </section>
        <p>Poster is generated automatically. Motion plays on hover or focus.</p>
        <footer>
          <span>Browser library · Document unchanged</span>
          <button id="candidate-save-confirm">Save</button>
        </footer>
      </section>
      <menu
        className="candidate-context-menu candidate-stage-add-menu"
        id="stage-add-menu"
        role="menu"
        aria-label="Add element"
        hidden
      >
        <strong>Add</strong>
        <button data-stage-add-element="rectangle">□ Rectangle</button>
        <button data-stage-add-element="text">T Text</button>
        <button data-stage-add-element="particle-field">✣ Particle Field</button>
        <hr />
        <button data-stage-add-element="browse">Browse Create…</button>
      </menu>
    </>
  );
}

function AssetTile({
  source,
  asset,
  preview,
  name,
  meta,
  root,
  path = "",
  directory,
  origin = source,
  pack,
  hidden = false,
}) {
  return (
    <button
      className="asset-tile"
      data-asset-source-view={source}
      data-asset={asset}
      data-file-root={root}
      data-file-path={path}
      data-file-directory={directory}
      data-asset-origin={origin}
      data-pack={pack}
      data-tags=""
      data-search={`${name} ${meta}`.toLowerCase()}
      draggable
      hidden={hidden}
      aria-label={`${name} · ${meta}`}
    >
      <span className={`asset-preview ${preview}`} />
      <span className="asset-name">{name}<small>{meta}</small></span>
      <span className="asset-tags" aria-label={`${name} tags`} hidden />
    </button>
  );
}

function CandidateProjectBrowser() {
  return (
    <div
      className="project-explorer candidate-project-browser"
      id="project-browser"
      hidden
      data-view="visual"
      data-info="Media Browser|Search project assets and registered folders from one surface"
    >
      <DiscoverySearchBar
        inputId="media-search"
        inputLabel="Search media"
        actions={(
          <DiscoveryViewToggle
            label="Media result view"
            options={[
              {
                active: true,
                "data-media-view": "visual",
                "aria-label": "Media thumbnail-only view",
                children: "▦",
              },
              {
                "data-media-view": "grid",
                "aria-label": "Media thumbnail and name view",
                children: "▤",
              },
              {
                "data-media-view": "list",
                "aria-label": "Media list view",
                children: "☷",
              },
            ]}
          />
        )}
      >
        <button className="btn quiet file-nav" id="file-back" hidden aria-label="Back">‹</button>
        <button className="btn quiet file-nav" id="file-parent" hidden aria-label="Parent folder">↑</button>
      </DiscoverySearchBar>

      <div className="candidate-asset-path-row">
        <nav className="candidate-asset-path" id="asset-path" aria-label="Current folder">
          All Media
        </nav>
        <span className="candidate-file-scope" id="file-scope-toggle" hidden>
          <button className="on" data-file-scope="folders" aria-label="Browse folders" aria-pressed="true">Browse folders</button>
          <button data-file-scope="all-files" aria-label="All files view" aria-pressed="false">All files</button>
        </span>
      </div>
      <DiscoveryBrowserLayout>
        <DiscoverySourceRail label="Media sources">
          <div className="candidate-nav-group">
            <button className="on" data-asset-source="all">
              <i aria-hidden="true">▦</i><span>All Media</span>
            </button>
            <button data-asset-source="project">
              <i aria-hidden="true">◆</i><span>Project</span>
            </button>
            <button data-media-recent>
              <i aria-hidden="true">↺</i><span>Recent</span>
            </button>
          </div>
          <DiscoverySection title="Registered folders">
            <div className="candidate-nav-group candidate-file-roots">
              <button data-file-root-select="city">
                <i aria-hidden="true">▣</i><span>City Source</span>
              </button>
              <button data-file-root-select="audio">
                <i aria-hidden="true">▣</i><span>Audio Library</span>
              </button>
              <button data-file-root-select="brand">
                <i aria-hidden="true">▣</i><span>Brand Kit</span>
              </button>
            </div>
          </DiscoverySection>
          <button className="candidate-register-folder" id="add-file-root">
            ＋ Add folder
          </button>
          <DiscoverySection title="Collections">
            <div className="candidate-nav-group">
              <button data-media-collection="favorites">
                <i aria-hidden="true">◎</i><span>Favorites</span>
              </button>
              <button data-media-collection="brand">
                <i aria-hidden="true">Aa</i><span>Brand</span>
              </button>
            </div>
          </DiscoverySection>
          <DiscoverySection title="Tags">
            <nav
              className="candidate-nav-group candidate-organization-tags candidate-tag-shelf"
              aria-label="Media tags"
            >
              <button className="candidate-tag-box" data-media-tag-box="favorite">
                <i aria-hidden="true">◎</i><span>Favorite</span><b>0</b>
              </button>
              <button className="candidate-tag-box" data-media-tag-box="brand">
                <i aria-hidden="true">Aa</i><span>Brand</span><b>0</b>
              </button>
              <button className="candidate-tag-box" data-media-tag-box="review">
                <i aria-hidden="true">✓</i><span>Review</span><b>0</b>
              </button>
              <button className="candidate-tag-box" data-media-tag-box="audio">
                <i aria-hidden="true">⌁</i><span>Audio</span><b>0</b>
              </button>
              <button className="candidate-tag-new" id="add-media-tag">＋ New tag</button>
            </nav>
            <div className="candidate-new-tag" id="new-media-tag-form" hidden>
              <input id="new-media-tag-name" aria-label="New tag name" placeholder="Tag name" />
              <button id="create-media-tag">Create</button>
            </div>
          </DiscoverySection>
          <DiscoverySection title="Packs">
            <div className="candidate-nav-group">
              <button data-pack-select="motion-kit-alpha">
                <i aria-hidden="true">▤</i><span>Motion Kit α</span>
              </button>
            </div>
          </DiscoverySection>
          <div id="media-file-hierarchy" hidden>
            <DiscoverySection title="Hierarchy">
              <div
                className="candidate-file-tree"
                id="file-tree"
                aria-label="Folder hierarchy"
              />
            </DiscoverySection>
          </div>
        </DiscoverySourceRail>

        <DiscoveryResults
          className="candidate-asset-results"
          title="All Media"
          titleId="asset-source-title"
          scope="PROJECT + REGISTERED FOLDERS"
          scopeId="asset-scope-label"
          count="6 ITEMS"
          countId="asset-count"
        >
          <div className="asset-grid candidate-asset-grid">
            <AssetTile source="all" origin="project" asset="night_drive.wav" preview="audio" name="night_drive.wav" meta="PROJECT · USED" />
            <AssetTile source="all" origin="project" asset="logo.svg" preview="logo" name="logo.svg" meta="PROJECT · UNPLACED" />
            <AssetTile source="all" origin="project" asset="grain.png" preview="texture" name="grain.png" meta="PROJECT · USED" pack="motion-kit-alpha" />
            <AssetTile source="all" origin="project" asset="city_loop.mp4" preview="video" name="city_loop.mp4" meta="PROJECT · INBOX" pack="motion-kit-alpha" />
            <AssetTile source="all" origin="files" root="audio" asset="impact_04.wav" preview="audio" name="impact_04.wav" meta="AUDIO LIBRARY" />
            <AssetTile source="all" origin="files" root="brand" path="Textures" asset="paper.png" preview="texture" name="paper.png" meta="BRAND KIT" />
            <AssetTile source="project" origin="project" asset="night_drive.wav" preview="audio" name="night_drive.wav" meta="USED" hidden />
            <AssetTile source="project" origin="project" asset="logo.svg" preview="logo" name="logo.svg" meta="UNPLACED" hidden />
            <AssetTile source="project" origin="project" asset="grain.png" preview="texture" name="grain.png" meta="USED" pack="motion-kit-alpha" hidden />
            <AssetTile source="project" origin="project" asset="city_loop.mp4" preview="video" name="city_loop.mp4" meta="INBOX" pack="motion-kit-alpha" hidden />
            <AssetTile source="files" root="city" asset="MV" directory="MV" preview="folder" name="MV" meta="Folder" hidden />
            <AssetTile source="files" root="city" asset="logo.svg" preview="logo" name="logo.svg" meta="SVG" hidden />
            <AssetTile source="files" root="city" path="MV" asset="night_drive" directory="MV/night_drive" preview="folder" name="night_drive" meta="Folder" hidden />
            <AssetTile source="files" root="city" path="MV" asset="city_loop.mp4" preview="video" name="city_loop.mp4" meta="12s" hidden />
            <AssetTile source="files" root="city" path="MV/night_drive" asset="source" directory="MV/night_drive/source" preview="folder" name="source" meta="Folder" hidden />
            <AssetTile source="files" root="city" path="MV/night_drive" asset="night_drive.wav" preview="audio" name="night_drive.wav" meta="3:42" hidden />
            <AssetTile source="files" root="city" path="MV/night_drive/source" asset="city_loop.mp4" preview="video" name="city_loop.mp4" meta="12s" hidden />
            <AssetTile source="files" root="city" path="MV/night_drive/source" asset="grain.png" preview="texture" name="grain.png" meta="4K" hidden />
            <AssetTile source="files" root="city" path="MV/night_drive/source" asset="skyline.exr" preview="video" name="skyline.exr" meta="EXR" hidden />
            <AssetTile source="files" root="city" path="MV/night_drive/source" asset="notes.txt" preview="folder" name="notes.txt" meta="TXT" hidden />
            <AssetTile source="files" root="audio" asset="Hits" directory="Hits" preview="folder" name="Hits" meta="Folder" hidden />
            <AssetTile source="files" root="audio" asset="Beds" directory="Beds" preview="folder" name="Beds" meta="Folder" hidden />
            <AssetTile source="files" root="audio" asset="impact_04.wav" preview="audio" name="impact_04.wav" meta="2s" hidden />
            <AssetTile source="files" root="audio" path="Hits" asset="impact_01.wav" preview="audio" name="impact_01.wav" meta="1s" hidden />
            <AssetTile source="files" root="audio" path="Hits" asset="impact_04.wav" preview="audio" name="impact_04.wav" meta="2s" hidden />
            <AssetTile source="files" root="audio" path="Beds" asset="night_drive.wav" preview="audio" name="night_drive.wav" meta="3:42" hidden />
            <AssetTile source="files" root="brand" asset="Logos" directory="Logos" preview="folder" name="Logos" meta="Folder" hidden />
            <AssetTile source="files" root="brand" asset="Textures" directory="Textures" preview="folder" name="Textures" meta="Folder" hidden />
            <AssetTile source="files" root="brand" path="Logos" asset="logo.svg" preview="logo" name="logo.svg" meta="SVG" hidden />
            <AssetTile source="files" root="brand" path="Logos" asset="wordmark.svg" preview="logo" name="wordmark.svg" meta="SVG" hidden />
            <AssetTile source="files" root="brand" path="Textures" asset="grain.png" preview="texture" name="grain.png" meta="4K" hidden />
            <AssetTile source="files" root="brand" path="Textures" asset="paper.png" preview="texture" name="paper.png" meta="2K" hidden />
          </div>
        </DiscoveryResults>
      </DiscoveryBrowserLayout>

      <section
        className="candidate-tag-panel"
        id="media-tag-panel"
        aria-hidden="true"
        hidden
      >
        <div>
          <button data-toggle-media-tag="favorite">Favorite</button>
          <button data-toggle-media-tag="brand">Brand</button>
          <button data-toggle-media-tag="review">Review</button>
          <button data-toggle-media-tag="audio">Audio</button>
        </div>
      </section>
      <div className="surface-foot">
        <span id="asset-foot-copy" hidden>PROJECT</span>
        <span id="asset-selection-count">1 selected</span>
        <button
          className="btn quiet"
          id="media-select-mode"
          aria-pressed="false"
        >
          Select
        </button>
      </div>
    </div>
  );
}

export function DiscoveryBrowserCandidate({ node, options }) {
  const [hierarchyWidth, setHierarchyWidth] = useState(106);
  const [thumbnailSize, setThumbnailSize] = useState(80);
  const hierarchyHidden = hierarchyWidth === 0;
  const hierarchy = useMemo(
    () => ({
      width: hierarchyWidth,
      setWidth: setHierarchyWidth,
      restore: () => setHierarchyWidth(106),
    }),
    [hierarchyWidth],
  );
  const candidateOptions = {
    ...options,
    replace(child) {
      if (
        child.type === "tag" &&
        child.attribs?.class?.split(" ").includes("browser-tabs")
      ) {
        return <CandidateBrowserTabs />;
      }
      if (
        child.type === "tag" &&
        child.attribs?.class?.split(" ").includes("panel-head") &&
        child.children?.some(
          (entry) => entry.type === "text" && entry.data.includes("Browser"),
        )
      ) {
        return (
          <div className="panel-head">
            Browser
            <small>MEDIA / CREATE / EFFECTS</small>
          </div>
        );
      }
      if (
        child.type === "tag" &&
        child.attribs?.id === "vism-browser"
      ) {
        return <CandidatePluginBrowser />;
      }
      if (
        child.type === "tag" &&
        child.attribs?.id === "project-browser"
      ) {
        return <CandidateProjectBrowser />;
      }
      return options.replace?.(child);
    },
  };

  return (
    <>
      <BrowserThumbnailSizeSettingBridge onChange={setThumbnailSize} />
      <BrowserHierarchyContext.Provider value={hierarchy}>
        {createElement(
          node.name,
          {
            ...attributesToProps(node.attribs, node.name),
            className: `${node.attribs.class} browser-candidate${hierarchyHidden ? " is-hierarchy-hidden" : ""}`,
            "data-hierarchy-hidden": String(hierarchyHidden),
            "data-browser-thumbnail-size": String(thumbnailSize),
            style: {
              "--browser-thumbnail-size": `${thumbnailSize}px`,
            },
          },
          domToReact(node.children ?? [], candidateOptions),
        )}
      </BrowserHierarchyContext.Provider>
    </>
  );
}
