import { Component, type ReactNode } from "react";

interface State {
  error: Error | null;
}

/** Se algo quebrar, mostra o erro em vez de deixar a tela preta. */
export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error) {
    console.error("ALEXANDRIA: erro de interface", error);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div style={{ padding: 40, maxWidth: 720, margin: "10vh auto", color: "#ebe8e1", font: "15px system-ui, sans-serif" }}>
        <h2 style={{ margin: "0 0 12px" }}>Algo deu errado</h2>
        <p style={{ color: "#8f8c85" }}>Tire um print desta tela e envie para corrigirmos.</p>
        <pre style={{ whiteSpace: "pre-wrap", userSelect: "text", background: "#161617", padding: 16, borderRadius: 12 }}>
          {String(this.state.error.stack ?? this.state.error.message).slice(0, 1200)}
        </pre>
        <button onClick={() => location.reload()} style={{ padding: "10px 20px", borderRadius: 999, border: 0, background: "#cbb98d", color: "#050505", fontWeight: 600, cursor: "pointer" }}>
          Recarregar
        </button>
      </div>
    );
  }
}
