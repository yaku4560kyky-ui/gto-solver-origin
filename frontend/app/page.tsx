"use client";

import { useState } from "react";

export default function Home() {
  const [result, setResult] = useState<unknown>(null);
  const [loading, setLoading] = useState(false);

  async function solveKuhn() {
    setLoading(true);
    try {
      const response = await fetch("/api/v1/solve/kuhn-cfr", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ iterations: 1000 }),
      });
      setResult(await response.json());
    } finally {
      setLoading(false);
    }
  }

  return (
    <main style={{ maxWidth: 960, margin: "0 auto", padding: 32, fontFamily: "sans-serif" }}>
      <h1>GTO Poker Solver</h1>
      <button onClick={solveKuhn} disabled={loading}>
        {loading ? "Solving..." : "Run Kuhn CFR"}
      </button>
      <pre style={{ marginTop: 24, padding: 16, background: "#f4f4f5", overflow: "auto" }}>
        {result ? JSON.stringify(result, null, 2) : "No result yet"}
      </pre>
    </main>
  );
}
