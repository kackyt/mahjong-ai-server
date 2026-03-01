import { BrowserRouter, Route, Routes } from "react-router-dom";
import { GamePage } from "./app/pages/GamePage";
import { TitlePage } from "./app/pages/TitlePage";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<TitlePage />} />
        <Route path="/game" element={<GamePage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
