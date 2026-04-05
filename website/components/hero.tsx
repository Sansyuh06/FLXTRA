"use client"

import { GitHubIcon } from "@/components/github-icon"

export function Hero() {
  return (
    <main className="relative min-h-screen flex flex-col items-center justify-center bg-background px-6 overflow-hidden">
      {/* GitHub icon - top right corner */}
      <a
        href="https://github.com/Sansyuh06/FLXTRA"
        target="_blank"
        rel="noopener noreferrer"
        className="fixed top-8 right-8 z-40 text-foreground glow-icon hover:text-primary transition-colors duration-300"
        aria-label="View FLXTRA on GitHub"
      >
        <GitHubIcon className="w-8 h-8" />
      </a>

      {/* Main content - centered */}
      <div className="relative z-10 flex flex-col items-center text-center max-w-4xl w-full">
        <h1 className="font-sans leading-tight tracking-tight">
          {/* Grey text */}
          <span className="block text-3xl md:text-5xl lg:text-6xl font-bold text-muted-foreground">
            {"an Agentic AI Browser"}
          </span>
          <span className="block text-3xl md:text-5xl lg:text-6xl font-bold text-muted-foreground mt-2">
            {"that works for you."}
          </span>
          {/* White text */}
          <span className="block text-3xl md:text-5xl lg:text-6xl font-bold text-foreground mt-4 glow-text-subtle">
            {"Not on you."}
          </span>
          <span className="block mt-10 md:mt-12">
            {/* White text */}
            <span className="text-2xl md:text-4xl lg:text-5xl font-bold text-foreground glow-text-subtle">
              {"Privacy? "}
            </span>
            {/* Red FLXTRA as download button */}
            <a
              href="https://github.com/Sansyuh06/FLXTRA/releases"
              target="_blank"
              rel="noopener noreferrer"
              className="text-2xl md:text-4xl lg:text-5xl font-black text-primary glow-text hover:scale-110 inline-block transition-transform duration-300 cursor-pointer"
            >
              FLXTRA
            </a>
          </span>
        </h1>
      </div>
    </main>
  )
}
