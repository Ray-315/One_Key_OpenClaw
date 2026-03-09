import { Routes, Route } from "react-router-dom";
import { Sidebar } from "./components/common/Sidebar";
import { EnvCheckPage } from "./pages/EnvCheckPage";
import { LogPage } from "./pages/LogPage";

function App() {
  return (
    <div className="flex h-screen overflow-hidden bg-[var(--color-bg)]">
      <Sidebar />
      <main className="flex-1 overflow-hidden">
        <Routes>
          <Route path="/" element={<EnvCheckPage />} />
          <Route path="/log" element={<LogPage />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
