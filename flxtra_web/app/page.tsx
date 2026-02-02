"use client";

import { useState } from "react";
import Sidebar from "@/components/Sidebar";
import BrowserFrame from "@/components/BrowserFrame";

export default function Home() {
  const [url, setUrl] = useState("https://example.com");
  const [tabs, setTabs] = useState([
    { id: 1, title: "New Tab", url: "", active: true },
  ]);

  const navigate = (newUrl: string) => {
    // Normalize URL
    let finalUrl = newUrl;
    if(!newUrl.startsWith("http")) {
      if(newUrl.includes(".") && !newUrl.includes(" ")) {
        finalUrl = `https://${newUrl}`;
      } else {
        finalUrl = `https://duckduckgo.com/?q=${encodeURIComponent(newUrl)}`;
      }
    }
    setUrl(finalUrl);
  };

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar tabs={tabs} onNavigate={navigate} />
      <main className="flex-1 bg-gray-900">
        <BrowserFrame url={url} />
      </main>
    </div>
  );
}
