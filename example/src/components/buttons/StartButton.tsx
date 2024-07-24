import { useWallet } from "@solana/wallet-adapter-react";
import { useWalletModal } from "@solana/wallet-adapter-react-ui";

export const StartButton = () => {
  const { setVisible } = useWalletModal();
  const { connecting } = useWallet();

  return (
    <button
      className="min-w-28 animate-pulse rounded-full p-2 text-xl"
      disabled={connecting}
      onClick={() => setVisible(true)}
    >
      {connecting ? "Connecting..." : "Start 🡒"}
    </button>
  );
};
