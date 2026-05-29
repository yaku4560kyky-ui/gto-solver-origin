"use client";

import { useState } from "react";

type WaitlistState = "idle" | "saving" | "joined" | "error";

export default function Home() {
  const [result, setResult] = useState<unknown>(null);
  const [loading, setLoading] = useState(false);
  const [waitlistState, setWaitlistState] = useState<WaitlistState>("idle");

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

  async function joinWaitlist(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setWaitlistState("saving");
    const form = new FormData(event.currentTarget);

    const response = await fetch("/api/v1/beta/waitlist", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: form.get("name"),
        email: form.get("email"),
        company: form.get("company") || null,
        intended_use: form.get("intended_use"),
      }),
    });

    setWaitlistState(response.ok ? "joined" : "error");
    if (response.ok) {
      event.currentTarget.reset();
    }
  }

  const plans = [
    {
      name: "Starter",
      price: "$19",
      features: ["Kuhn CFR demo", "Hand evaluator", "Community support"],
    },
    {
      name: "Pro",
      price: "$49",
      features: ["CFR+ experiments", "Preflop abstraction", "Priority beta access"],
    },
    {
      name: "Team",
      price: "$149",
      features: ["Shared workspaces", "Webhook billing", "Launch support"],
    },
  ];

  return (
    <main style={{ fontFamily: "Inter, system-ui, sans-serif", color: "#101828", background: "#f8fafc" }}>
      <section style={{ maxWidth: 1120, margin: "0 auto", padding: "56px 24px 32px" }}>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: 32, alignItems: "center" }}>
          <div>
            <p style={{ margin: "0 0 12px", color: "#0f766e", fontWeight: 700 }}>Private beta</p>
            <h1 style={{ margin: 0, fontSize: 48, lineHeight: 1.05, letterSpacing: 0 }}>GTO Poker Solver</h1>
            <p style={{ fontSize: 18, lineHeight: 1.6, color: "#475467", marginTop: 20 }}>
              Train compact CFR strategies, inspect hand strength, and prototype poker decisions from a focused web interface.
            </p>
          </div>

          <form onSubmit={joinWaitlist} style={{ background: "#ffffff", border: "1px solid #d0d5dd", borderRadius: 8, padding: 24, display: "grid", gap: 12 }}>
            <h2 style={{ margin: "0 0 4px", fontSize: 22 }}>Join the waitlist</h2>
            <input name="name" required placeholder="Name" style={inputStyle} />
            <input name="email" required type="email" placeholder="Email" style={inputStyle} />
            <input name="company" placeholder="Company" style={inputStyle} />
            <textarea name="intended_use" required placeholder="How will you use the solver?" rows={4} style={inputStyle} />
            <button type="submit" disabled={waitlistState === "saving"} style={primaryButtonStyle}>
              {waitlistState === "saving" ? "Joining..." : "Request access"}
            </button>
            <p style={{ minHeight: 20, margin: 0, color: waitlistState === "error" ? "#b42318" : "#027a48" }}>
              {waitlistState === "joined" ? "You are on the list." : waitlistState === "error" ? "Could not join. Check your details." : ""}
            </p>
          </form>
        </div>
      </section>

      <section style={{ maxWidth: 1120, margin: "0 auto", padding: "24px" }}>
        <div style={{ background: "#ffffff", border: "1px solid #d0d5dd", borderRadius: 8, padding: 24 }}>
          <div style={{ display: "flex", justifyContent: "space-between", gap: 16, alignItems: "center", flexWrap: "wrap" }}>
            <div>
              <h2 style={{ margin: 0, fontSize: 28 }}>Solver demo</h2>
              <p style={{ margin: "8px 0 0", color: "#667085" }}>Run a 1,000-iteration Kuhn CFR solve against the API.</p>
            </div>
            <button onClick={solveKuhn} disabled={loading} style={primaryButtonStyle}>
              {loading ? "Solving..." : "Run demo"}
            </button>
          </div>
          <pre style={{ marginTop: 20, minHeight: 180, padding: 16, background: "#111827", color: "#e5e7eb", borderRadius: 8, overflow: "auto" }}>
            {result ? JSON.stringify(result, null, 2) : "No result yet"}
          </pre>
        </div>
      </section>

      <section style={{ maxWidth: 1120, margin: "0 auto", padding: "24px 24px 56px" }}>
        <h2 style={{ margin: "0 0 16px", fontSize: 28 }}>Pricing</h2>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))", gap: 16 }}>
          {plans.map((plan) => (
            <article key={plan.name} style={{ background: "#ffffff", border: "1px solid #d0d5dd", borderRadius: 8, padding: 20 }}>
              <h3 style={{ margin: 0, fontSize: 20 }}>{plan.name}</h3>
              <p style={{ margin: "12px 0", fontSize: 32, fontWeight: 800 }}>{plan.price}<span style={{ fontSize: 16, color: "#667085" }}>/mo</span></p>
              <ul style={{ margin: 0, paddingLeft: 20, color: "#475467", lineHeight: 1.8 }}>
                {plan.features.map((feature) => (
                  <li key={feature}>{feature}</li>
                ))}
              </ul>
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%",
  boxSizing: "border-box",
  border: "1px solid #d0d5dd",
  borderRadius: 6,
  padding: "10px 12px",
  fontSize: 15,
};

const primaryButtonStyle: React.CSSProperties = {
  border: "1px solid #0f766e",
  borderRadius: 6,
  background: "#0f766e",
  color: "#ffffff",
  cursor: "pointer",
  fontWeight: 700,
  padding: "10px 14px",
};
