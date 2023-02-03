import { PublicKey } from "@solana/web3.js";

export const abbreviate = (
  pubkey: string | PublicKey | null | undefined,
  limit = 5
): string => {
  if (!pubkey) return "";
  if (typeof pubkey === "string") {
    if (pubkey.length <= limit) return pubkey;
    return pubkey.slice(0, limit) + "..." + pubkey.slice(-limit);
  }
  return (
    pubkey.toBase58().slice(0, limit) + "..." + pubkey.toBase58().slice(-limit)
  );
};
