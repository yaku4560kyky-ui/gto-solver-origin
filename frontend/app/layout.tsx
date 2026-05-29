import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "GTO Solver",
  description: "Poker solver interface",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ja">
      <body>{children}</body>
    </html>
  );
}
