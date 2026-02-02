"use client";

import { useState } from "react";

interface Tab {
    id: number;
    title: string;
    url: string;
    active: boolean;
}

interface SidebarProps {
    tabs: Tab[];
    onNavigate: (url: string) => void;
}

export default function Sidebar({ tabs, onNavigate }: SidebarProps) {
    const [urlInput, setUrlInput] = useState("");

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if(e.key === "Enter" && urlInput.trim()) {
            onNavigate(urlInput.trim());
        }
    };

    return (
        <aside className="w-[260px] h-full bg-gray-950 border-r border-gray-800 flex flex-col">
            {/* Header */}
            <header className="flex items-center justify-between p-4 border-b border-gray-800/50">
                <div className="flex items-center gap-2">
                    <div className="w-6 h-6 rounded-md bg-gradient-to-br from-violet-500 to-pink-500 flex items-center justify-center text-xs font-bold">
                        ☄
                    </div>
                    <span className="text-sm font-semibold">Comet</span>
                </div>
                <button className="w-7 h-7 rounded-md hover:bg-gray-800 flex items-center justify-center text-gray-400 hover:text-white">
                    ⚙
                </button>
            </header>

            {/* URL Bar */}
            <div className="p-4">
                <div className="flex items-center h-9 bg-gray-900 border border-gray-800 rounded-full focus-within:border-violet-500 focus-within:ring-2 focus-within:ring-violet-500/20 transition-all">
                    <span className="pl-3 text-green-500 text-xs">🔒</span>
                    <input
                        type="text"
                        value={urlInput}
                        onChange={(e) => setUrlInput(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Search or enter URL"
                        className="flex-1 bg-transparent border-none px-3 text-sm text-white placeholder:text-gray-500 focus:outline-none"
                    />
                </div>
            </div>

            {/* Nav Buttons */}
            <div className="flex gap-2 px-4 pb-4">
                <button className="flex-1 h-8 rounded-lg bg-gray-900 border border-gray-800 text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                    ←
                </button>
                <button className="flex-1 h-8 rounded-lg bg-gray-900 border border-gray-800 text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                    →
                </button>
                <button className="flex-1 h-8 rounded-lg bg-gray-900 border border-gray-800 text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                    ↻
                </button>
                <button className="flex-1 h-8 rounded-lg bg-gray-900 border border-gray-800 text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                    ⌂
                </button>
            </div>

            {/* Tabs Header */}
            <div className="flex items-center justify-between px-4 py-2">
                <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">Tabs</span>
                <span className="text-xs text-gray-500 bg-gray-800 px-2 py-0.5 rounded-full">{tabs.length}</span>
            </div>

            {/* Tabs List */}
            <div className="flex-1 overflow-y-auto px-2">
                {tabs.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-8 text-center">
                        <span className="text-3xl opacity-30 mb-2">📭</span>
                        <p className="text-xs text-gray-500">No tabs open</p>
                    </div>
                ) : (
                    tabs.map((tab) => (
                        <div
                            key={tab.id}
                            className={`flex items-center gap-3 px-3 py-2 my-0.5 rounded-lg cursor-pointer ${tab.active
                                    ? "bg-violet-500/10 border-l-2 border-violet-500"
                                    : "hover:bg-gray-800/50"
                                }`}
                        >
                            <div className="w-5 h-5 rounded-md bg-gray-800 flex items-center justify-center text-[10px]">
                                ✨
                            </div>
                            <div className="flex-1 min-w-0">
                                <div className="text-xs font-medium truncate">{tab.title || "New Tab"}</div>
                                <div className="text-[10px] text-gray-500 truncate">
                                    {tab.url || "about:blank"}
                                </div>
                            </div>
                            <button className="w-4 h-4 rounded opacity-0 group-hover:opacity-100 hover:bg-red-500 hover:text-white flex items-center justify-center text-gray-500">
                                ×
                            </button>
                        </div>
                    ))
                )}
            </div>

            {/* Footer */}
            <footer className="flex gap-2 p-4 border-t border-gray-800/50">
                <button className="flex-1 h-9 rounded-lg bg-violet-600 hover:bg-violet-500 text-white font-medium text-xs flex items-center justify-center gap-2 shadow-lg shadow-violet-500/20">
                    + New Tab
                </button>
                <button className="flex-1 h-9 rounded-lg bg-gray-900 border border-gray-800 hover:bg-gray-800 text-gray-300 font-medium text-xs flex items-center justify-center gap-2">
                    ✨ Lyra
                </button>
            </footer>
        </aside>
    );
}
