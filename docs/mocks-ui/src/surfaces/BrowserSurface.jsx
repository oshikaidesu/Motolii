import { Field, PanelHeader, Tab, TabList } from "../primitives";

export function BrowserSurface({ fixture }) {
  return (
    <aside className="mock-surface mock-browser" aria-label="Browser">
      <PanelHeader title="Browser" wayfinding="project" />
      <TabList label="Browser source">
        <Tab selected={fixture.activeTab === "project"}>
          Project
        </Tab>
        <Tab selected={fixture.activeTab === "plugins"}>
          Plugins
        </Tab>
      </TabList>
      <Field
        className="mock-search"
        label="Search plugins"
        type="search"
        placeholder="Search plugins or tags"
      />
      <div className="plugin-grid">
        {fixture.items.map((item) => (
          <button className="plugin-card" key={item.name} data-state={item.state}>
            <span className="plugin-thumb" aria-hidden="true" />
            <strong>{item.name}</strong>
            <small>{item.purpose}</small>
            {item.state !== "installed" && <i>{item.state}</i>}
          </button>
        ))}
      </div>
    </aside>
  );
}
