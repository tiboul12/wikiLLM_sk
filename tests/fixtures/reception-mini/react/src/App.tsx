export function App() {
  return (
    <Routes>
      <Route path="/panier" element={<Panier />} />
    </Routes>
  );
}

/** Panier. */
export function Panier() {
  return <div />;
}
