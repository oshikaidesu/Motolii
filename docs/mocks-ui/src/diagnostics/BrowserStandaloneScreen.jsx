import { DiscoveryBrowserCandidate } from "@motolii/motolii-web";
import "./browser-standalone-screen.css";

const rectangleIdentity = {
  scope_ref: "create",
  item_id: "rectangle",
};

export function BrowserStandaloneScreen() {
  return (
    <main className="browser-standalone-screen">
      <DiscoveryBrowserCandidate rectangleIdentity={rectangleIdentity} />
    </main>
  );
}
