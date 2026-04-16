"use client";

interface TabInfo {
  id: number;
  title: string;
  url: string;
  mode?: "browse" | "agent";
}

interface SidebarProps {
  tabs: TabInfo[];
  activeTabUrl: string;
  activeTabMode: "browse" | "agent";
  agentPaneOpen: boolean;
  agentStatus: string;
  agentPlan: string | null;
  onAskAI: () => void;
  onToggleAgentPane: () => void;
}

export default function Sidebar({
  tabs,
  activeTabUrl,
  activeTabMode,
  agentPaneOpen,
  agentStatus,
  agentPlan,
  onAskAI,
  onToggleAgentPane,
}: SidebarProps) {
  return (
    <aside className="w-[340px] min-w-[340px] border-l border-white/10 bg-[#060607]/95 backdrop-blur-2xl">
      <div className="flex h-full flex-col p-5">
        <div className="flex items-start justify-between gap-4 rounded-[32px] border border-white/10 bg-[#13131a]/95 p-5 shadow-[0_25px_80px_-46px_rgba(239,68,68,0.45)]">
          <div>
            <div className="text-base font-semibold text-white">Flextra AI</div>
            <div className="mt-1 text-xs uppercase tracking-[0.3em] text-[#8a8a8a]">Premium agent assistant</div>
          </div>
          <button
            onClick={onToggleAgentPane}
            className={`rounded-full px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] transition ${agentPaneOpen ? "bg-white text-black" : "bg-[#1c1c21] text-[#f1f1f1] hover:bg-white/10"}`}
          >
            {agentPaneOpen ? "Active" : "Open"}
          </button>
        </div>

        <div className="mt-5 rounded-[30px] border border-white/10 bg-[#111117]/90 p-5 shadow-[0_26px_90px_-48px_rgba(239,68,68,0.35)]">
          <div className="flex items-center justify-between text-xs uppercase tracking-[0.2em] text-[#8a8a8a]">
            <span>Page Intelligence</span>
            <span className="rounded-full bg-[#19191f] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-[#f8f8f8]">{activeTabMode === "agent" ? "Agent" : "Classic"}</span>
          </div>
          <div className="mt-4 space-y-3 text-sm text-[#d4d4d8]">
            <div className="rounded-[24px] bg-[#0f0f14]/90 px-4 py-4">
              <div className="text-[11px] uppercase tracking-[0.18em] text-[#8f8f8f]">URL</div>
              <div className="mt-2 truncate text-sm text-white">{activeTabUrl || "No page loaded"}</div>
            </div>
            <div className="rounded-[24px] bg-[#0f0f14]/90 px-4 py-4">
              <div className="text-[11px] uppercase tracking-[0.18em] text-[#8f8f8f]">Status</div>
              <div className="mt-2 text-sm text-[#efefef]">{agentStatus}</div>
            </div>
          </div>

          <button
            onClick={onAskAI}
            className="mt-4 w-full rounded-2xl bg-[#ef4444] px-4 py-3 text-sm font-semibold text-white transition hover:bg-[#ff6666]"
          >
            Ask AI about this page
          </button>
        </div>

        <div className="mt-4 rounded-[30px] border border-white/10 bg-[#111117]/90 p-5 text-xs uppercase tracking-[0.16em] text-[#8a8a8a]">
          <div className="mb-3 font-semibold text-white">Privacy & Safety</div>
          <ul className="space-y-3 text-[13px] text-[#c7c7c7]">
            <li>• Browser state stays local; no telemetry.</li>
            <li>• AI actions are visible and reversible.</li>
            <li>• Tab content is isolated, never shared externally.</li>
          </ul>
        </div>

        <div className="mt-4 flex-1 overflow-hidden rounded-[30px] border border-white/10 bg-[#111117]/90 p-5">
          <div className="text-xs uppercase tracking-[0.24em] text-[#7c7c7c]">Agent Plan</div>
          <div className="mt-4 min-h-[170px] rounded-[26px] border border-white/10 bg-[#0b0b0d]/95 p-4 text-sm text-[#c9c9c9]">
            {agentPlan ? (
              <pre className="whitespace-pre-wrap break-words text-sm leading-6">{agentPlan}</pre>
            ) : (
              <div className="text-[#7d7d7d]">Create a plan by asking AI from the omnibox or page actions.</div>
            )}
          </div>
        </div>

        <div className="mt-4 rounded-[30px] border border-white/10 bg-[#111117]/90 p-5 text-sm text-[#d1d1d1]">
          <div className="mb-3 text-xs uppercase tracking-[0.18em] text-[#8a8a8a]">Open tabs</div>
          <div className="space-y-3">
            {tabs.map((tab) => (
              <div
                key={tab.id}
                className={`rounded-[24px] border px-3 py-3 transition ${
                  tab.url === activeTabUrl ? "border-[#ef4444] bg-[#17171e]" : "border-white/10 bg-[#0f0f14]/90"
                }`}
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="text-sm font-semibold text-white truncate">{tab.title || "New Tab"}</div>
                  <span className={`rounded-full px-2 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] ${tab.mode === "agent" ? "bg-[#ef4444]/15 text-[#fee2e2]" : "bg-white/5 text-[#c7c7d1]"}`}>
                    {tab.mode === "agent" ? "Agent" : "Browse"}
                  </span>
                </div>
                <div className="mt-2 text-[11px] text-[#8d8d8d] truncate">{tab.url || "about:blank"}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </aside>
  );
}
