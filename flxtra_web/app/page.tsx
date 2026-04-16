"use client";

import { type KeyboardEvent, useEffect, useMemo, useState } from "react";
import Sidebar from "@/components/Sidebar";
import BrowserFrame from "@/components/BrowserFrame";

interface BrowserTab {
  id: number;
  url: string;
  title: string;
  favicon?: string;
  history: string[];
  historyIndex: number;
  loading: boolean;
  progress: number;
  mode: "browse" | "agent";
  suspended: boolean;
}

const initialTab: BrowserTab = {
  id: 1,
  url: "",
  title: "New Tab",
  favicon: "",
  history: [],
  historyIndex: -1,
  loading: false,
  progress: 0,
  mode: "browse",
  suspended: false,
};

const normalizeBrowserInput = (input: string) => {
  const trimmed = input.trim();
  const lower = trimmed.toLowerCase();

  if (lower.startsWith("ai ") || lower.startsWith("ask ") || lower.startsWith("agent ")) {
    return { url: trimmed, mode: "agent" as const };
  }

  const urlPattern = /^(https?:\/\/|www\.)[\w-.]+\.[a-z]{2,}(|\/.*)$/i;
  const hostnamePattern = /^[\w-]+\.[a-z]{2,}(\/.*)?$/i;

  if (urlPattern.test(trimmed) || hostnamePattern.test(trimmed)) {
    const url = trimmed.startsWith("http") ? trimmed : `https://${trimmed}`;
    return { url, mode: "navigate" as const };
  }

  return {
    url: `https://duckduckgo.com/?q=${encodeURIComponent(trimmed)}`,
    mode: "search" as const,
  };
};

export default function Home() {
  const [tabs, setTabs] = useState<BrowserTab[]>([initialTab]);
  const [activeTabId, setActiveTabId] = useState(1);
  const [omnibox, setOmnibox] = useState("");
  const [agentMode, setAgentMode] = useState(false);
  const [viewMode, setViewMode] = useState<"classic" | "agent">("classic");
  const [agentPaneOpen, setAgentPaneOpen] = useState(false);
  const [agentStatus, setAgentStatus] = useState("Ready");
  const [agentPlan, setAgentPlan] = useState<string | null>(null);
  const [omniboxFocused, setOmniboxFocused] = useState(false);

  const activeTab = useMemo(
    () => tabs.find((tab) => tab.id === activeTabId) ?? tabs[0],
    [tabs, activeTabId]
  );

  useEffect(() => {
    setOmnibox(activeTab.url);
  }, [activeTab.id, activeTab.url]);

  type Suggestion = {
    label: string;
    value: string;
    subtitle: string;
  };

  const knownSites: Suggestion[] = [
    { label: "GitHub", value: "https://github.com", subtitle: "github.com" },
    { label: "YouTube", value: "https://www.youtube.com", subtitle: "youtube.com" },
    { label: "Twitter", value: "https://x.com", subtitle: "x.com" },
    { label: "Reddit", value: "https://www.reddit.com", subtitle: "reddit.com" },
    { label: "Wikipedia", value: "https://en.wikipedia.org", subtitle: "wikipedia.org" },
    { label: "Stack Overflow", value: "https://stackoverflow.com", subtitle: "stackoverflow.com" },
    { label: "Amazon", value: "https://www.amazon.com", subtitle: "amazon.com" },
    { label: "DuckDuckGo", value: "https://duckduckgo.com", subtitle: "duckduckgo.com" },
  ];

  const suggestions = useMemo<Suggestion[]>(() => {
    const query = omnibox.trim();
    if (!query) return [];

    const normalized = query.toLowerCase();
    const seen = new Set<string>();
    const results: Suggestion[] = [];

    const addSuggestion = (item: Suggestion) => {
      if (!seen.has(item.value)) {
        seen.add(item.value);
        results.push(item);
      }
    };

    if (query.match(/^(https?:\/\/|www\.)/i) || query.match(/^[\w-]+\.[a-z]{2,}/i)) {
      const maybeUrl = query.startsWith("http") ? query : `https://${query}`;
      addSuggestion({ label: `Go to ${maybeUrl}`, value: maybeUrl, subtitle: maybeUrl });
    }

    if (viewMode !== "agent") {
      addSuggestion({
        label: `Search DuckDuckGo for \"${query}\"`,
        value: `https://duckduckgo.com/?q=${encodeURIComponent(query)}`,
        subtitle: "Search",
      });
      addSuggestion({
        label: `Search Google for \"${query}\"`,
        value: `https://www.google.com/search?q=${encodeURIComponent(query)}`,
        subtitle: "Search",
      });
    } else {
      addSuggestion({
        label: `Ask AI to: \"${query}\"`,
        value: query,
        subtitle: "Agent query",
      });
    }

    knownSites.forEach((site) => {
      if (site.label.toLowerCase().includes(normalized) || site.subtitle.toLowerCase().includes(normalized)) {
        addSuggestion(site);
      }
    });

    tabs.forEach((tab) => {
      if (!tab.url) return;
      const urlText = tab.url.toLowerCase();
      if (urlText.includes(normalized) || tab.title.toLowerCase().includes(normalized)) {
        addSuggestion({
          label: `Open ${tab.title || tab.url}`,
          value: tab.url,
          subtitle: tab.url.replace(/^https?:\/\//, ""),
        });
      }
    });

    return results.slice(0, 6);
  }, [omnibox, tabs, viewMode]);

  const handleSuggestionSelect = (value: string) => {
    setOmnibox(value);
    handleOmniboxSubmit(value);
  };

  const setActiveTab = (id: number) => {
    if (id === activeTabId) return;
    setActiveTabId(id);
  };

  const createTab = () => {
    const nextId = tabs.length ? Math.max(...tabs.map((tab) => tab.id)) + 1 : 1;
    const newTab: BrowserTab = {
      ...initialTab,
      id: nextId,
      title: "New Tab",
      mode: "browse",
      suspended: false,
    };
    setTabs((current) => current.map((tab) => ({ ...tab, loading: false })).concat(newTab));
    setActiveTabId(nextId);
    setOmnibox("");
  };

  const closeTab = (id: number) => {
    setTabs((current) => {
      const next = current.filter((tab) => tab.id !== id);
      if (!next.length) {
        setActiveTabId(1);
        return [initialTab];
      }
      if (id === activeTabId) {
        const index = current.findIndex((tab) => tab.id === id);
        const neighbor = next[Math.max(0, Math.min(index - 1, next.length - 1))];
        setActiveTabId(neighbor.id);
      }
      return next;
    });
  };

  const suspendTab = (id: number) => {
    setTabs((current) =>
      current.map((tab) =>
        tab.id !== id
          ? tab
          : {
              ...tab,
              suspended: !tab.suspended,
              loading: false,
              progress: tab.suspended ? 100 : tab.progress,
            }
      )
    );
  };

  const updateTab = (id: number, url: string) => {
    setTabs((current) =>
      current.map((tab) => {
        if (tab.id !== id) return tab;
        const history = tab.history.slice(0, tab.historyIndex + 1);
        return {
          ...tab,
          url,
          mode: "browse",
          suspended: false,
          title: url.replace(/^https?:\/\//, ""),
          history: [...history, url],
          historyIndex: history.length,
          loading: true,
          progress: 15,
        };
      })
    );
  };

  const handleOmniboxSubmit = (value: string) => {
    const trimmed = value.trim();
    if (!trimmed) return;

    const parsed = normalizeBrowserInput(trimmed);
    const isAgentIntent = viewMode === "agent" || parsed.mode === "agent";

    if (isAgentIntent) {
      setAgentMode(true);
      setAgentPaneOpen(true);
      setAgentStatus("Planning your AI action...");
      setAgentPlan(`Plan: analyze page, evaluate intent, and execute for '${trimmed}'`);
      return;
    }

    updateTab(activeTab.id, parsed.url);
    setOmnibox(parsed.url);
  };

  const handleOmniboxKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      handleOmniboxSubmit(omnibox);
    }
  };

  const handleGoBack = () => {
    setTabs((current) =>
      current.map((tab) => {
        if (tab.id !== activeTabId || tab.historyIndex <= 0) return tab;
        const prevIndex = tab.historyIndex - 1;
        return {
          ...tab,
          url: tab.history[prevIndex],
          historyIndex: prevIndex,
          loading: true,
          progress: 15,
        };
      })
    );
  };

  const handleGoForward = () => {
    setTabs((current) =>
      current.map((tab) => {
        if (tab.id !== activeTabId || tab.historyIndex >= tab.history.length - 1) return tab;
        const nextIndex = tab.historyIndex + 1;
        return {
          ...tab,
          url: tab.history[nextIndex],
          historyIndex: nextIndex,
          loading: true,
          progress: 15,
        };
      })
    );
  };

  const handleReload = () => {
    setTabs((current) =>
      current.map((tab) =>
        tab.id !== activeTabId
          ? tab
          : {
              ...tab,
              loading: true,
              progress: 15,
            }
      )
    );
  };

  const handleFrameLoaded = () => {
    setTabs((current) =>
      current.map((tab) =>
        tab.id !== activeTabId
          ? tab
          : {
              ...tab,
              loading: false,
              progress: 100,
            }
      )
    );
  };

  const toggleAgentPane = () => {
    setAgentPaneOpen((open) => !open);
    if (!agentPaneOpen) {
      setAgentMode(true);
      setViewMode("agent");
    }
  };

  const onAskAI = () => {
    setAgentPaneOpen(true);
    setAgentMode(true);
    setAgentStatus("Ready to analyze this page");
    setAgentPlan("Ask AI to summarize or interact with the current page.");
  };

  const title = activeTab.title || (activeTab.url ? activeTab.url.replace(/^https?:\/\//, "") : "New Tab");

  return (
    <div className="relative flex h-screen min-h-0 overflow-hidden text-white">
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,_rgba(239,68,68,0.18),_transparent_18%),radial-gradient(circle_at_80%_10%,_rgba(255,255,255,0.06),_transparent_25%),linear-gradient(180deg,_#040406_0%,_#08080b_100%)]" />
      <div className="relative flex flex-1 flex-col min-h-0">
        <div className="sticky top-0 z-40 border-b border-white/10 bg-[#040408]/90 backdrop-blur-xl shadow-[0_24px_80px_-48px_rgba(0,0,0,0.55)]">
          <div className="flex h-16 items-center gap-3 px-4 py-2">
            <div className="flex items-center gap-3 rounded-3xl border border-white/10 bg-[#0e0e12]/80 px-4 py-3 shadow-[inset_0_0_0_1px_rgba(255,255,255,0.04)]">
              <div className="flex h-9 w-9 items-center justify-center rounded-2xl bg-[#17171d]/90 text-sm font-bold text-[#ef4444]">F</div>
              <div>
                <div className="text-sm font-semibold text-white">Flextra</div>
                <div className="text-[11px] uppercase tracking-[0.32em] text-[#8d8d8d]">AI-NATIVE BROWSER</div>
              </div>
            </div>

            <div className="flex-1">
              <div className="relative">
                <div className="absolute left-4 top-1/2 z-10 -translate-y-1/2 text-sm text-[#9a9a9a]">�</div>
                <input
                  value={omnibox}
                  onFocus={() => setOmniboxFocused(true)}
                  onBlur={() => setTimeout(() => setOmniboxFocused(false), 150)}
                  onChange={(event) => setOmnibox(event.target.value)}
                  onKeyDown={handleOmniboxKeyDown}
                  placeholder={viewMode === "agent" ? "Ask AI or give an action" : "Search or enter address"}
                  className="w-full rounded-full border border-white/10 bg-[#111118]/95 py-3 pl-14 pr-32 text-sm text-white outline-none transition focus:border-[#ef4444] focus:ring-2 focus:ring-[#ef4444]/20"
                />
                {omniboxFocused && suggestions.length > 0 && (
                  <div className="absolute left-0 right-0 top-full z-20 mt-2 overflow-hidden rounded-[28px] border border-white/10 bg-[#09090f]/98 shadow-[0_36px_120px_-50px_rgba(0,0,0,0.75)] backdrop-blur-xl">
                    <div className="max-h-72 overflow-auto">
                      {suggestions.map((suggestion, index) => (
                        <button
                          key={index}
                          type="button"
                          onMouseDown={() => handleSuggestionSelect(suggestion.value)}
                          className="flex w-full flex-col gap-1 border-b border-white/5 px-4 py-4 text-left text-sm text-white transition hover:bg-white/5"
                        >
                          <span className="truncate font-semibold">{suggestion.label}</span>
                          <span className="truncate text-[11px] text-[#9a9a9a]">{suggestion.subtitle}</span>
                        </button>
                      ))}
                    </div>
                  </div>
                )}
                <div className="absolute right-3 top-1/2 z-10 flex items-center gap-2 -translate-y-1/2">
                  <button
                    onClick={() => handleOmniboxSubmit(omnibox)}
                    className="rounded-full bg-[#ef4444] px-4 py-2 text-xs font-semibold uppercase tracking-[0.18em] text-white transition hover:bg-[#ff6666]"
                  >
                    Go
                  </button>
                  <button
                    onClick={toggleAgentPane}
                    className={`rounded-full px-4 py-2 text-xs font-semibold uppercase tracking-[0.18em] transition ${agentPaneOpen ? "bg-white text-black" : "bg-[#1c1c1c] text-[#f1f1f1] hover:bg-white/10"}`}
                  >
                    {agentPaneOpen ? "Agent" : "AI"}
                  </button>
                </div>
              </div>
            </div>

            <div className="hidden items-center gap-2 rounded-3xl bg-[#0e0e12]/80 px-3 py-2 lg:flex">
              <button
                onClick={() => setViewMode("classic")}
                className={`rounded-full px-3 py-2 text-xs font-semibold uppercase tracking-[0.16em] transition ${viewMode === "classic" ? "bg-[#ef4444] text-white" : "bg-[#16161d] text-[#adafb4] hover:bg-white/10"}`}
              >
                Classic
              </button>
              <button
                onClick={() => {
                  setViewMode("agent");
                  setAgentPaneOpen(true);
                  setAgentMode(true);
                }}
                className={`rounded-full px-3 py-2 text-xs font-semibold uppercase tracking-[0.16em] transition ${viewMode === "agent" ? "bg-white text-black" : "bg-[#16161d] text-[#adafb4] hover:bg-white/10"}`}
              >
                Agent
              </button>
            </div>
          </div>

          <div className="flex min-h-[54px] items-center gap-3 overflow-x-auto px-4 pb-2">
            {tabs.map((tab) => (
              <div
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`relative inline-flex min-w-[150px] cursor-pointer items-center gap-3 rounded-[24px] border px-3 py-2 text-sm transition ${
                  tab.id === activeTabId
                    ? "border-[#ef4444] bg-[#18181f] text-white shadow-[0_16px_50px_-28px_rgba(239,68,68,0.65)]"
                    : "border-white/10 bg-[#101018] text-[#d2d2d2] hover:border-white/20 hover:bg-[#16161f]"
                }`}
              >
                <div className="min-w-0 flex-1">
                  <div className="truncate font-semibold">
                    {tab.title || "New Tab"}
                  </div>
                  <div className="truncate text-[11px] text-[#9a9a9a]">{tab.url ? tab.url.replace(/^https?:\/\//, "") : "Start page"}</div>
                </div>
                <span className={`rounded-full px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] ${tab.mode === "agent" ? "bg-[#ef4444]/15 text-[#fee2e2]" : "bg-white/5 text-[#c7c7d1]"}`}>
                  {tab.mode === "agent" ? "Agent" : "Browse"}
                </span>
              </div>
            ))}
            <button
              onClick={createTab}
              className="inline-flex min-w-[120px] items-center justify-center rounded-2xl border border-dashed border-white/15 bg-[#0f0f0f] px-4 py-2 text-sm text-[#d1d1d1] transition hover:border-white/30 hover:bg-[#161617]"
            >
              + New Tab
            </button>
          </div>
        </div>

        <div className="flex flex-1 min-h-0 overflow-hidden">
          <main className="flex-1 min-h-0 px-4 py-4">
            <BrowserFrame
              url={activeTab.url}
              loading={activeTab.loading}
              progress={activeTab.progress}
              suspended={activeTab.suspended}
              onAskAI={onAskAI}
              onFrameLoaded={handleFrameLoaded}
              onResume={() => suspendTab(activeTab.id)}
            />
          </main>

          <Sidebar
            tabs={tabs.map((tab) => ({ id: tab.id, title: tab.title, url: tab.url, mode: tab.mode }))}
            activeTabUrl={activeTab.url}
            activeTabMode={activeTab.mode}
            agentPaneOpen={agentPaneOpen}
            agentStatus={agentStatus}
            agentPlan={agentPlan}
            onAskAI={onAskAI}
            onToggleAgentPane={toggleAgentPane}
          />
        </div>
      </div>
    </div>
  );
}
