import { PanelHeader } from "../primitives";

export function InspectorSurface({ fixture, plugin }) {
  return (
    <aside className="mock-surface mock-inspector" aria-label="Inspector">
      <PanelHeader title="Inspector" wayfinding="inspector" />
      <section className="inspector-group">
        <small>SELECTED OBJECT</small>
        <strong>{fixture.object}</strong>
      </section>
      <section className="inspector-group">
        <small>TRANSFORM</small>
        <div className="parameter-row">
          <span>Depth Z</span>
          <output>{fixture.depth.toFixed(3)}</output>
        </div>
      </section>
      <section className="inspector-group">
        <small>APPLIED PLUGINS</small>
        <div className="applied-plugin">
          <span aria-hidden="true">◎</span>
          <strong>{plugin.name}</strong>
          <small>IN → Effect → OUT</small>
        </div>
        {fixture.parameters.map((parameter) => (
          <div className="parameter-row" key={parameter.name}>
            <span>{parameter.name}</span>
            <output>{parameter.value}</output>
            <i data-state={parameter.automation}>AUTO {parameter.automation.toUpperCase()}</i>
          </div>
        ))}
      </section>
    </aside>
  );
}
