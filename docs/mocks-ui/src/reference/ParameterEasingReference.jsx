import { EasingGraphCandidate } from "../candidates/EasingGraphCandidate.jsx";
import { InspectorSurface } from "../surfaces/InspectorSurface.jsx";
import { BrowserSurface } from "../surfaces/BrowserSurface.jsx";
import { Button } from "../primitives/index.jsx";
import { referenceFixtures } from "./fixtures.js";
import { loadReferenceFixtures } from "./loadReferenceFixtures.js";
import { ReferenceStateStrip } from "./ReferenceStateStrip.jsx";

export function ParameterEasingReference() {
  const fixture = loadReferenceFixtures("parameter-easing", referenceFixtures);
  return (
    <main className="motolii-mock-app" data-reference-screen="parameter-easing">
      <header className="mock-titlebar">
        <strong className="wordmark">MOTOLII</strong><span>Parameter and easing reference</span>
      </header>
      <nav className="mock-commandbar" aria-label="Parameter commands">
        <Button pressed>GRAPH</Button><span>Opacity · selected interval</span>
      </nav>
      <section className="mock-workspace">
        <BrowserSurface fixture={fixture.browser} />
        <section className="mock-surface" data-semantic-id="easing-popup">
          <EasingGraphCandidate intervalContext={fixture.intervalContext} />
        </section>
        <InspectorSurface fixture={fixture.inspector} plugin={fixture.plugin} />
      </section>
      <section className="mock-surface mock-timeline" aria-label="Parameter timeline" />
      <ReferenceStateStrip
        spacing={fixture.tokenSpacing}
        items={[{ label: "Reference states are projected into Inspector parameters" }]}
      />
    </main>
  );
}
