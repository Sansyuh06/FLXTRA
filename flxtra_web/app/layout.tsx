import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Flextra Browser",
  description: "Flextra - AI-native browser with privacy-first browsing",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="bg-[#050505] text-white antialiased">
        {children}
      </body>
    </html>
  );
}
