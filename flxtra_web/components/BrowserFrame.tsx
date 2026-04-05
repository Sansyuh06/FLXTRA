"use client";

interface BrowserFrameProps {
    url: string;
}

export default function BrowserFrame({ url }: BrowserFrameProps) {
    // For MVP, we use an iframe with a disclaimer
    // In production, this would connect to a Remote Browser Isolation backend

    const canEmbed = url && !url.includes("google.com"); // Most sites block iframe

    return (
        <div className="w-full h-full flex flex-col">
            {/* Browser Top Bar */}
            <div className="h-14 bg-[#070707] border-b border-[#3a0f12] flex items-center px-4 gap-3">
                <div className="flex items-center gap-2 text-sm text-[#f5f5f5]">
                    <span className="w-2.5 h-2.5 rounded-full bg-[#ef4444]"></span>
                    <span className="w-2.5 h-2.5 rounded-full bg-[#ffffff] opacity-70"></span>
                    <span className="w-2.5 h-2.5 rounded-full bg-[#9f9f9f]"></span>
                </div>
                <div className="flex items-center gap-2 bg-[#0d0d0d] border border-[#3a0f12] rounded-full px-3 py-2 flex-1">
                    <button className="w-8 h-8 rounded-full text-[#c4c4c4] hover:text-white hover:bg-[#1f1214] transition">←</button>
                    <button className="w-8 h-8 rounded-full text-[#c4c4c4] hover:text-white hover:bg-[#1f1214] transition">→</button>
                    <button className="w-8 h-8 rounded-full text-[#c4c4c4] hover:text-white hover:bg-[#1f1214] transition">⟳</button>
                    <input
                        readOnly
                        value={url || "about:blank"}
                        className="flex-1 bg-transparent border-none text-xs text-white focus:outline-none truncate"
                    />
                </div>
                <button className="px-3 py-2 rounded-lg bg-[#ef4444] text-white text-xs font-semibold hover:bg-[#fb7185] transition">
                    Secure
                </button>
            </div>

            {/* Content Area */}
            <div className="flex-1 relative bg-[#050505]">
                {url ? (
                    canEmbed ? (
                        <iframe
                            src={url}
                            className="w-full h-full border-0"
                            sandbox="allow-scripts allow-same-origin allow-forms"
                            title="Web Content"
                        />
                    ) : (
                        <div className="absolute inset-0 flex flex-col items-center justify-center text-center p-8 bg-[#050505]">
                            <div className="text-6xl mb-4">🚧</div>
                            <h2 className="text-xl font-semibold mb-2 text-white">Cannot Preview This Site</h2>
                            <p className="text-[#b7b7b7] text-sm max-w-md mb-4">
                                This website blocks embedding. In the full Flextra browser,
                                we use Remote Browser Isolation (RBI) to securely stream any website.
                            </p>
                            <a
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="px-4 py-2 bg-[#ef4444] hover:bg-[#fb7185] rounded-lg text-sm font-medium transition-colors"
                            >
                                Open in New Tab →
                            </a>
                        </div>
                    )
                ) : (
                    <div className="absolute inset-0 flex flex-col items-center justify-center px-6">
                        <div className="text-8xl mb-6 opacity-20 text-[#ef4444]">✦</div>
                        <h1 className="text-3xl font-bold mb-3 text-white">Welcome to Flextra</h1>
                        <p className="text-[#b7b7b7] text-sm max-w-xl text-center">This is the browser shell. Enter an address in the sidebar or ask the AI agent to navigate, search, click, and interact for you.</p>
                    </div>
                )}
            </div>
        </div>
    );
}
