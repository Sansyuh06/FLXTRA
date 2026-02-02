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
            {/* URL Display Bar */}
            <div className="h-10 bg-gray-900 border-b border-gray-800 flex items-center px-4 gap-3">
                <div className="flex items-center gap-2 flex-1 min-w-0">
                    <span className="text-green-500 text-sm">🔒</span>
                    <span className="text-xs text-gray-400 truncate">{url || "about:blank"}</span>
                </div>
                <span className="text-[10px] text-amber-500 bg-amber-500/10 px-2 py-1 rounded">
                    Web Preview
                </span>
            </div>

            {/* Content Area */}
            <div className="flex-1 relative bg-gray-950">
                {url ? (
                    canEmbed ? (
                        <iframe
                            src={url}
                            className="w-full h-full border-0"
                            sandbox="allow-scripts allow-same-origin allow-forms"
                            title="Web Content"
                        />
                    ) : (
                        <div className="absolute inset-0 flex flex-col items-center justify-center text-center p-8">
                            <div className="text-6xl mb-4">🚧</div>
                            <h2 className="text-xl font-semibold mb-2">Cannot Preview This Site</h2>
                            <p className="text-gray-400 text-sm max-w-md mb-4">
                                This website blocks embedding. In the full version of Comet Web,
                                we use Remote Browser Isolation (RBI) to stream any website securely.
                            </p>
                            <a
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="px-4 py-2 bg-violet-600 hover:bg-violet-500 rounded-lg text-sm font-medium transition-colors"
                            >
                                Open in New Tab →
                            </a>
                        </div>
                    )
                ) : (
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                        <div className="text-8xl mb-6 opacity-20">☄️</div>
                        <h1 className="text-2xl font-bold mb-2">Welcome to Comet</h1>
                        <p className="text-gray-500 text-sm">Enter a URL in the sidebar to get started</p>
                    </div>
                )}
            </div>
        </div>
    );
}
