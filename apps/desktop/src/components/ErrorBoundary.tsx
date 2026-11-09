import { Component, type ReactNode } from "react";
import i18n from "../i18n";

interface State {
  error: Error | null;
}

/** Beklenmeyen bir çizim hatasında boş pencere yerine açıklama gösterir. */
export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error) {
    console.error("Arayüz hatası:", error);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;
    return (
      <div className="flex h-full flex-col items-center justify-center gap-4 bg-base p-8 text-center text-primary">
        <h1 className="text-lg font-semibold">{i18n.t("crash.title")}</h1>
        <p className="max-w-lg text-secondary">{i18n.t("crash.text")}</p>
        <pre className="max-w-2xl overflow-auto rounded-md border border-border bg-surface p-3 text-left font-mono text-[11px] text-error">
          {error.message}
        </pre>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => this.setState({ error: null })}
            className="rounded-md border border-border px-3 py-1.5 hover:bg-elevated"
          >
            {i18n.t("crash.retry")}
          </button>
          <button
            type="button"
            onClick={() => location.reload()}
            className="rounded-md bg-accent px-3 py-1.5 font-medium text-[var(--bg-base)]"
          >
            {i18n.t("crash.reload")}
          </button>
        </div>
      </div>
    );
  }
}
