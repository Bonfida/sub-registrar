import type { Metadata } from "next";
import AppWalletProvider from "@/components/providers/AppWalletProvider";
import "./globals.css";

export const metadata: Metadata = {
  title: "SNS Subregistrar Demo",
  description: "SNS Subregistrar Demo",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="bg-gradient-to-b from-black to-gray-800 text-white">
        <AppWalletProvider>{children}</AppWalletProvider>
      </body>
    </html>
  );
}
