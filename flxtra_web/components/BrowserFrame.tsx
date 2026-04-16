"use client";

interface BrowserFrameProps {
  url: string;
  loading: boolean;
  progress: number;
  suspended: boolean;
  onAskAI: () => void;
  onFrameLoaded: () => void;
  onResume: () => void;
}

export default function BrowserFrame({ url, loading, progress, suspended, onAskAI, onFrameLoaded, onResume }: BrowserFrameProps) {
  const canEmbed = url && !url.includes("google.com");

  return (
    <div className="flex h-full flex-col overflow-hidden rounded-[22px] border border-white/10 bg-[#090909]/90 shadow-[0_40px_120px_-70px_rgba(239,68,68,0.55)]">
      <div className="relative h-1 overflow-hidden bg-white/5">
        <div
          className="h-full bg-[#ef4444] transition-all duration-300"
          style={{ width: `${loading ? Math.max(progress, 10) : 100}%` }}
        />
      </div>

      <div className="flex-1 min-h-0 overflow-hidden relative">
        {suspended ? (
          <div className="flex h-full flex-col items-center justify-center gap-4 bg-[#050505]/95 p-8 text-center text-white">
            <div className="rounded-3xl bg-white/5 p-6 text-5xl">💤</div>
            <div>
              <h2 className="text-2xl font-semibold">Tab Suspended</h2>
              <p className="mt-2 max-w-md text-sm text-[#b7b7b7]">
                This tab is suspended to save resources. Resume browsing to continue from the last URL.
              </p>
            </div>
            <button
              onClick={onResume}
              className="rounded-full bg-[#ef4444] px-5 py-2 text-sm font-semibold text-white transition hover:bg-[#ff6b6b]"
            >
              Resume Tab
            </button>
          </div>
        ) : url ? (
          canEmbed ? (
            <iframe
              src={url}
              className="h-full w-full border-0 bg-[#0a0a0a]"
              sandbox="allow-scripts allow-same-origin allow-forms"
              title="Web Content"
              onLoad={onFrameLoaded}
            />
          ) : (
            <div className="flex h-full flex-col items-center justify-center gap-4 p-8 text-center text-white">
              <div className="rounded-3xl bg-white/5 p-6 text-5xl">🚧</div>
              <div>
                <h2 className="text-2xl font-semibold">Cannot Preview This Site</h2>
                <p className="mt-2 max-w-md text-sm text-[#b7b7b7]">
                  This website blocks embedding. Flextra will use remote browser isolation for full support while keeping the local AI and privacy stack intact.
                </p>
              </div>
              <a
                href={url}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex rounded-full bg-[#ef4444] px-5 py-2 text-sm font-semibold text-white transition hover:bg-[#ff6b6b]"
              >
                Open in New Tab
              </a>
            </div>
          )
        ) : (
          <div className="flex h-full flex-col items-center justify-center gap-4 px-8 text-center text-white">
            <div className="text-[5rem] leading-none text-[#ef4444]/85">✦</div>
            <div className="max-w-xl">
              <h1 className="text-3xl font-bold">Welcome to Flextra</h1>
              <p className="mt-3 text-sm text-[#b7b7b7]">
                A privacy-first browser shell with AI as a power layer. Type a URL, search term, or ask AI to navigate for you.
              </p>
            </div>
            <button
              onClick={onAskAI}
              className="rounded-full bg-[#ef4444] px-6 py-3 text-sm font-semibold text-white transition hover:bg-[#ff6b6b]"
            >
              Ask AI about this page
            </button>
          </div>
        )}
      </div>

      {url && (
        <div className="flex items-center justify-between gap-4 border-t border-white/10 bg-[#080808]/95 px-4 py-3 text-xs text-[#d4d4d8]">
          <span>Browsing <strong>{url.replace(/^https?:\/\//, "")}</strong></span>
          <button
            onClick={onAskAI}
            className="rounded-full border border-white/10 bg-white/5 px-4 py-2 text-xs font-semibold text-white transition hover:border-[#ef4444] hover:bg-[#ef4444]/10"
          >
            Ask AI about this page
          </button>
        </div>
      )}
    </div>
  );
}
