import { createRoot } from "react-dom/client";
import { BrowserStandaloneScreen } from "./BrowserStandaloneScreen.jsx";
import "../tokens/mock-candidates.css";

createRoot(document.getElementById("root")).render(<BrowserStandaloneScreen />);
