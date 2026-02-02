import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Comet Browser",
  description: "AI-Powered Browser - Web Edition",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="bg-gray-950 text-gray-50 antialiased">
        {children}
      </body>
    </html>
  );
}
