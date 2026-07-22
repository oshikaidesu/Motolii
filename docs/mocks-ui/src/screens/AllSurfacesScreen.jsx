import { allSurfacesFixture } from "../fixtures/allSurfaces";
import { Button } from "../primitives";
import { BrowserSurface } from "../surfaces/BrowserSurface";
import { InspectorSurface } from "../surfaces/InspectorSurface";
import { StageSurface } from "../surfaces/StageSurface";
import { TimelineSurface } from "../surfaces/TimelineSurface";
import "./all-surfaces.css";

export function AllSurfacesScreen() {
  const fixture = allSurfacesFixture;
  return (
    <div className="motolii-mock-app">
      <header className="mock-titlebar">
        <strong className="wordmark">MOTOLII</strong>
        <span>{fixture.project}</span>
        <span className="mock-grow" />
        <Button variant="quiet">Settings</Button>
        <Button>Export</Button>
      </header>
      <nav className="mock-commandbar" aria-label="Command bar">
        <Button pressed>SELECT</Button>
        <Button variant="quiet">CAMERA</Button>
        <Button variant="quiet">HAND</Button>
        <span>{fixture.composition} / <strong>{fixture.plugin.name}</strong></span>
      </nav>
      <main className="mock-workspace">
        <BrowserSurface fixture={fixture.browser} />
        <StageSurface composition={fixture.composition} plugin={fixture.plugin} />
        <InspectorSurface fixture={fixture.inspector} plugin={fixture.plugin} />
      </main>
      <TimelineSurface fixture={fixture.timeline} />
      <footer className="mock-status">
        <strong>{fixture.plugin.name}</strong>
        <span>Selected object · Host standard Inspector</span>
      </footer>
    </div>
  );
}
