import { useRequest } from "ahooks";
import { useFavouriteDomain } from "../useFavoriteDomain";
import { useConnection } from "@solana/wallet-adapter-react";
import { NameRegistryState } from "@bonfida/spl-name-service";
import { Connection, PublicKey } from "@solana/web3.js";
import { getDomainKey } from "@bonfida/spl-name-service";

const record = async (connection: Connection, key: PublicKey) => {
  try {
    const { registry } = await NameRegistryState.retrieve(connection, key);
    return registry.data;
  } catch {}
};

export const useProfilePic = (address: string | undefined | null) => {
  const { connection } = useConnection();
  const { data: fav } = useFavouriteDomain(address);

  const fn = async () => {
    if (!fav || !address) return;
    try {
      let data: Buffer | undefined = undefined;

      const { pubkey } = await getDomainKey("pic." + fav);
      const { pubkey: pubkeyRecord } = await getDomainKey("pic." + fav, true);

      const registryData = await record(connection, pubkey);
      const recordData = await record(connection, pubkeyRecord);

      // Prioritize the record
      if (recordData) {
        data = recordData;
      } else if (registryData) {
        data = registryData;
      }

      if (!data) return;

      return data.toString("utf-8");
    } catch {
      return undefined;
    }
  };

  return useRequest(fn, {
    refreshDeps: [address, fav],
    cacheKey: `profile-pic-${address}`,
  });
};
