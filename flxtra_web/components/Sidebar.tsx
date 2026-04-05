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
        <aside className="w-[280px] h-full bg-[#090909] border-r border-[#3a0f12] flex flex-col">
            {/* Header */}
            <header className="flex items-center justify-between p-4 border-b border-[#3a0f12]">
                <div className="flex items-center gap-2">
                    <div className="w-7 h-7 rounded-lg bg-[#3a0f12] border border-[#830f14] flex items-center justify-center text-sm font-bold text-white">
                        F
                    </div>
                    <div>
                        <div className="text-sm font-semibold text-white">Flextra</div>
                        <div className="text-[10px] text-[#a8a8a8]">Black·Red·White</div>
                    </div>
                </div>
                <button className="w-8 h-8 rounded-md hover:bg-[#1f1214] flex items-center justify-center text-[#d4d4d8] hover:text-white">
                    ⚙
                </button>
            </header>

            {/* URL Bar */}
            <div className="p-4">
                <div className="flex items-center h-10 bg-[#121212] border border-[#3a0f12] rounded-full focus-within:border-[#ef4444] focus-within:ring-2 focus-within:ring-[#ef4444]/20 transition-all">
                    <span className="pl-3 text-[#ef4444] text-xs">🔒</span>
                    <input
                        type="text"
                        value={urlInput}
                        onChange={(e) => setUrlInput(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Search or enter URL"
                        className="flex-1 bg-transparent border-none px-3 text-sm text-white placeholder:text-[#7f7f7f] focus:outline-none"
                    />
                </div>
            </div>

            {/* Nav Buttons */}
            <div className="flex gap-2 px-4 pb-4">
                <button className="flex-1 h-8 rounded-lg bg-[#121212] border border-[#3a0f12] text-[#c4c4c4] hover:text-white hover:bg-[#1f1214] transition-colors">
                    ←
                </button>
                <button className="flex-1 h-8 rounded-lg bg-[#121212] border border-[#3a0f12] text-[#c4c4c4] hover:text-white hover:bg-[#1f1214] transition-colors">
                    →
                </button>
                <button className="flex-1 h-8 rounded-lg bg-[#121212] border border-[#3a0f12] text-[#c4c4c4] hover:text-white hover:bg-[#1f1214] transition-colors">
                    ↻
                </button>
                <button className="flex-1 h-8 rounded-lg bg-[#121212] border border-[#3a0f12] text-[#c4c4c4] hover:text-white hover:bg-[#1f1214] transition-colors">
                    ⌂
                </button>
            </div>

            {/* Tabs Header */}
            <div className="flex items-center justify-between px-4 py-2">
                <span className="text-xs font-medium text-[#9a9a9a] uppercase tracking-wide">Tabs</span>
                <span className="text-xs text-white bg-[#111111] px-2 py-0.5 rounded-full">{tabs.length}</span>
            </div>

            {/* Tabs List */}
            <div className="flex-1 overflow-y-auto px-2">
                {tabs.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-8 text-center">
                        <span className="text-3xl opacity-30 mb-2">📭</span>
                        <p className="text-xs text-[#8a8a8a]">No tabs open</p>
                    </div>
                ) : (
                    tabs.map((tab) => (
                        <div
                            key={tab.id}
                            className={`group flex items-center gap-3 px-3 py-2 my-0.5 rounded-lg cursor-pointer ${tab.active
                                    ? "bg-[#2f0b0f] border-l-2 border-[#ef4444]"
                                    : "hover:bg-[#151214]"
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
            <footer className="flex gap-2 p-4 border-t border-[#3a0f12]">
                <button className="flex-1 h-9 rounded-lg bg-[#ef4444] hover:bg-[#fb7185] text-white font-medium text-xs flex items-center justify-center gap-2 shadow-lg shadow-[#ef4444]/20">
                    + New Tab
                </button>
                <button className="flex-1 h-9 rounded-lg bg-[#121212] border border-[#3a0f12] hover:bg-[#1f1214] text-[#e5e5e5] font-medium text-xs flex items-center justify-center gap-2">
                    ✨ AI Hub
                </button>
            </footer>
        </aside>
    );
}
