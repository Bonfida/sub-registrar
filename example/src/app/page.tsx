"use client";

import Image from "next/image";
import { useWallet } from "@solana/wallet-adapter-react";
import { ConnectButton } from "@/components/buttons/ConnectButton";
import { StartButton } from "@/components/buttons/StartButton";
import { SubdomainSearch } from "@/components/SubdomainSearch";

export default function Home() {
  const { connected } = useWallet();

  return (
    <main className="flex min-h-screen flex-col">
      <div className="flex justify-end px-4 py-3">
        <ConnectButton />
      </div>
      <div className="m-auto flex max-w-[90vw] grow flex-col items-center justify-center pb-16">
        <Image
          className="relative mb-6 size-20"
          src="/images/sns.svg"
          alt="SNS Logo"
          width={33}
          height={38}
          priority
        />
        <h1 className="mb-4 text-center text-2xl">
          SNS Subdomain Registrar Demo
        </h1>
        {connected ? <SubdomainSearch /> : <StartButton />}
      </div>
    </main>
  );
}
