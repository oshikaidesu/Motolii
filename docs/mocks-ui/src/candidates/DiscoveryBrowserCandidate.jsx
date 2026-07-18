import { createElement } from "react";
import {
  attributesToProps,
  domToReact,
} from "html-react-parser";
import "./discovery-browser-candidate.css";

function PluginCard({
  mode,
  folder,
  labels,
  search,
  thumbnail,
  kind,
  name,
  state,
  selected = false,
}) {
  return (
    <button
      className={`vism candidate-plugin-card${selected ? " on" : ""}`}
      data-mode={mode}
      data-folder={folder}
      data-labels={labels}
      data-search={search}
      data-plugin-name={name}
      data-plugin-kind={kind}
      draggable={!state}
      aria-label={`${name}${state ? ` · ${state}` : ""}`}
    >
      <span className={`plugin-thumb ${thumbnail}`}>
        <span className="candidate-kind" aria-hidden="true">{kind}</span>
        {state ? (
          <span className={`thumb-state state ${mode}`}>{state}</span>
        ) : null}
      </span>
      <span className="candidate-card-name">
        <b>{name}</b>
      </span>
    </button>
  );
}

function CandidatePluginBrowser() {
  return (
    <div
      id="vism-browser"
      className="candidate-plugin-browser"
      data-view="thumb"
      data-info="Plugin Browser|Browse visual results with the same shell as Project"
    >
      <div className="candidate-search-row">
        <input
          className="search"
          id="search"
          type="search"
          placeholder="Search"
          aria-label="Search plugins"
        />
        <button
          className="candidate-icon-button"
          id="plugin-filter-toggle"
          aria-label="Filters"
          aria-expanded="false"
        >
          ◫
        </button>
        <span className="plugin-view-toggle" aria-label="Result view">
          <button
            data-plugin-view="visual"
            aria-label="Thumbnail-only view"
          >
            ▦
          </button>
          <button
            className="on"
            data-plugin-view="thumb"
            aria-label="Thumbnail and name view"
          >
            ▤
          </button>
          <button data-plugin-view="detail" aria-label="List view">
            ☷
          </button>
        </span>
      </div>

      <div className="candidate-filter-panel" id="plugin-filter-panel" hidden>
        <button className="plugin-label on" data-plugin-label="all">All</button>
        <button className="plugin-label" data-plugin-label="effect">FX</button>
        <button className="plugin-label" data-plugin-label="generator">Gen</button>
        <button className="plugin-label" data-plugin-label="text">Text</button>
      </div>

      <div className="candidate-browser-layout">
        <nav className="candidate-browser-nav" aria-label="Plugin sources">
          <div className="candidate-nav-group">
            <button className="on" data-plugin-source="all">
              <i aria-hidden="true">▦</i><span>All</span>
            </button>
            <button data-plugin-source="installed">
              <i aria-hidden="true">◆</i><span>Installed</span>
            </button>
            <button data-plugin-source="project">
              <i aria-hidden="true">◇</i><span>Used</span>
            </button>
            <button data-plugin-source="issues">
              <i aria-hidden="true">!</i><span>Issues</span>
            </button>
            <button id="plugin-recent-toggle">
              <i aria-hidden="true">↺</i><span>Recent</span>
            </button>
          </div>
          <div className="candidate-nav-title">Collections</div>
          <div className="candidate-nav-group">
            <button data-plugin-collection="motion">
              <i aria-hidden="true">◎</i><span>Favorites</span>
            </button>
            <button data-plugin-collection="type">
              <i aria-hidden="true">Aa</i><span>Type</span>
            </button>
          </div>
          <button
            className="candidate-add-collection"
            id="add-plugin-folder"
            aria-label="Add collection"
          >
            ＋
          </button>
          <select id="plugin-folder" aria-label="Plugin collection" hidden>
            <option value="all">All</option>
            <option value="motion">Favorites</option>
            <option value="type">Type</option>
            <option value="experimental">Experimental</option>
            <option value="project">Project used</option>
          </select>
        </nav>

        <section className="candidate-results" id="plugin-results">
          <header className="candidate-results-head">
            <strong>Results</strong>
            <span id="plugin-result-count">4</span>
          </header>
          <div className="plugin-grid candidate-plugin-grid">
            <PluginCard
              mode="installed"
              folder="motion project"
              labels="goto effect glow"
              search="echo bloom light pulse glow effect installed"
              thumbnail="bloom"
              kind="FX"
              name="Echo Bloom"
              selected
            />
            <PluginCard
              mode="discover"
              folder="type"
              labels="generator text"
              search="glyph current kinetic text lyrics generator bundled available"
              thumbnail="glyph"
              kind="G"
              name="Glyph Current"
            />
            <PluginCard
              mode="blocked"
              folder="experimental"
              labels="effect space"
              search="fold field space geometry effect incompatible"
              thumbnail="fold"
              kind="FX"
              name="Fold Field"
              state="Unavailable"
            />
            <PluginCard
              mode="missing"
              folder="project"
              labels="generator array missing"
              search="ribbon array missing unavailable generator"
              thumbnail="ribbon"
              kind="G"
              name="Ribbon Array"
              state="Missing"
            />
          </div>
        </section>

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
      </div>
    </div>
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
      hidden={hidden}
      aria-label={`${name} · ${meta}`}
    >
      <span className={`asset-preview ${preview}`} />
      <span className="asset-name">{name}<small>{meta}</small></span>
    </button>
  );
}

function CandidateProjectBrowser() {
  return (
    <div
      className="project-explorer candidate-project-browser"
      id="project-browser"
      hidden
      data-info="Project Explorer|Browse project assets and files with the same shell as Plugins"
    >
      <div className="candidate-search-row">
        <input
          className="search"
          type="search"
          placeholder="Search"
          aria-label="Search assets"
        />
        <button
          className="asset-source-tab on"
          data-asset-source="project"
          aria-label="Project assets"
        >
          ◇
        </button>
        <button
          className="asset-source-tab"
          data-asset-source="files"
          aria-label="External files"
        >
          ▣
        </button>
        <button className="candidate-icon-button" aria-label="List view">☷</button>
        <button className="btn quiet file-nav" id="file-back" hidden aria-label="Back">‹</button>
        <button className="btn quiet file-nav" id="file-parent" hidden aria-label="Parent folder">↑</button>
      </div>

      <nav className="candidate-asset-path" id="asset-path" aria-label="Current folder">
        night_drive / Assets
      </nav>

      <div className="candidate-browser-layout">
        <nav className="candidate-browser-nav" aria-label="Asset sources">
          <div data-asset-source-view="project">
            <div className="candidate-nav-group">
              <button className="on"><i aria-hidden="true">▦</i><span>All</span></button>
              <button><i aria-hidden="true">◆</i><span>Used</span></button>
              <button><i aria-hidden="true">◇</i><span>Unplaced</span></button>
              <button><i aria-hidden="true">↺</i><span>Recent</span></button>
            </div>
            <div className="candidate-nav-title">Collections</div>
            <div className="candidate-nav-group">
              <button><i aria-hidden="true">◎</i><span>Favorites</span></button>
              <button><i aria-hidden="true">Aa</i><span>Brand</span></button>
            </div>
          </div>
          <div data-asset-source-view="files" hidden>
            <div className="candidate-nav-title">Registered folders</div>
            <div className="candidate-nav-group candidate-file-roots">
              <button className="on" data-file-root-select="city">
                <i aria-hidden="true">▣</i><span>City Source</span>
              </button>
              <button data-file-root-select="audio">
                <i aria-hidden="true">▣</i><span>Audio Library</span>
              </button>
              <button data-file-root-select="brand">
                <i aria-hidden="true">▣</i><span>Brand Kit</span>
              </button>
            </div>
            <button className="candidate-register-folder" id="add-file-root">
              ＋ Add folder
            </button>
          </div>
        </nav>

        <section className="candidate-results candidate-asset-results">
          <header className="candidate-results-head">
            <strong id="asset-source-title">Project assets</strong>
            <span id="asset-scope-label">USED / UNPLACED</span>
            <span id="asset-count">4</span>
          </header>
          <div className="asset-grid candidate-asset-grid">
            <AssetTile source="project" asset="night_drive.wav" preview="audio" name="night_drive.wav" meta="USED" />
            <AssetTile source="project" asset="logo.svg" preview="logo" name="logo.svg" meta="UNPLACED" />
            <AssetTile source="project" asset="grain.png" preview="texture" name="grain.png" meta="USED" />
            <AssetTile source="project" asset="city_loop.mp4" preview="video" name="city_loop.mp4" meta="INBOX" />
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
        </section>
      </div>

      <div className="surface-foot">
        <span id="asset-foot-copy">PROJECT</span>
        <button className="btn primary" id="place-asset">PLACE</button>
      </div>
    </div>
  );
}

export function DiscoveryBrowserCandidate({ node, options }) {
  const candidateOptions = {
    ...options,
    replace(child) {
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

  return createElement(
    node.name,
    {
      ...attributesToProps(node.attribs, node.name),
      className: `${node.attribs.class} browser-candidate`,
    },
    domToReact(node.children ?? [], candidateOptions),
  );
}
