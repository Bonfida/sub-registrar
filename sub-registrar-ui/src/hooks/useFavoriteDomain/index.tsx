import { useRequest } from "ahooks";
import { FavouriteDomain, NAME_OFFERS_ID } from "@bonfida/name-offers";
import { PublicKey } from "@solana/web3.js";
import { useConnection } from "@solana/wallet-adapter-react";
import { performReverseLookup } from "@bonfida/spl-name-service";
import { NameRegistryState } from "@bonfida/spl-name-service";

export const useFavouriteDomain = (publicKey: string | null | undefined) => {
  const { connection } = useConnection();
  const fn = async () => {
    if (!publicKey) return;
    try {
      const [favKey] = await FavouriteDomain.getKey(
        NAME_OFFERS_ID,
        new PublicKey(publicKey)
      );
      const favourite = await FavouriteDomain.retrieve(connection, favKey);
      const { registry, nftOwner } = await NameRegistryState.retrieve(
        connection,
        favourite.nameAccount
      );

      // Domain is wraped
      if (!!nftOwner && nftOwner.toBase58() !== publicKey) {
        return;
      }
      // Domain is not wrapped
      if (!nftOwner && !!registry && registry.owner.toBase58() !== publicKey) {
        return;
      }

      const reverse = await performReverseLookup(
        connection,
        favourite.nameAccount
      );
      return reverse;
    } catch (err) {}
  };
  return useRequest(fn, {
    refreshDeps: [publicKey],
    cacheKey: `useFavouriteDomain-${publicKey}`,
  });
};
