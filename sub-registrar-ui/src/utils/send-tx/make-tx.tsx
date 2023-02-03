import {
  Connection,
  PublicKey,
  TransactionInstruction,
  Transaction,
  Signer,
} from "@solana/web3.js";
import { Toast } from "@bonfida/components";
import { sleep } from "@bonfida/hooks";
import { sendRawTransaction, retry } from "@bonfida/hooks";
import { getConnection } from "../connection/confirmed";
import { RPC_POOL, WSS_URL } from "../../settings/rpc";
import { tokenAuthFetchMiddleware } from "@strata-foundation/web3-token-auth";
import { getToken } from "@bonfida/hooks";

export const getAllConnections = (connections: Connection[]): Connection[] => {
  const token = localStorage.getItem("auth-token");
  if (token) {
    connections.push(
      new Connection(RPC_POOL, {
        wsEndpoint: WSS_URL,
        httpHeaders: { Authorization: `Bearer ${token}` },
        fetchMiddleware: tokenAuthFetchMiddleware({
          getToken,
          tokenExpiry: 2.5 * 60 * 1_000,
        }),
      })
    );
  }
  return connections;
};

export const makeTx = async (
  connection: Connection,
  feePayer: PublicKey,
  instructions: TransactionInstruction[],
  signTransaction: (tx: Transaction) => Promise<Transaction>,
  toast: Toast,
  transaction?: Transaction,
  signers?: Signer[],
  skipPreflight = false
) => {
  let sig: string | undefined = undefined;
  try {
    toast.processing();

    const { blockhash } = await connection.getLatestBlockhash();
    const tx = transaction ? transaction : new Transaction();

    if (!transaction) {
      tx.add(...instructions);
      tx.feePayer = feePayer;
      tx.recentBlockhash = blockhash;
    }

    if (signers && signers.length > 0) {
      tx.sign(...signers);
    }

    const signedTx = await signTransaction(tx);

    const connections = [connection, ...getAllConnections([])];
    // Retrying
    sig = await retry(async () => {
      sig = await sendRawTransaction(connections, signedTx, skipPreflight);
      await connection.confirmTransaction(sig, "processed");
      return sig;
    });

    toast.success(sig);
    return { success: true, signature: sig };
  } catch (err) {
    console.log(err);
    if (err instanceof Error) {
      const message = err.message;
      if (message.includes("Transaction cancelled")) {
        toast.close();
      } else if (message.includes("not found")) {
        toast.error("Solana network is unstable - Try again");
      } else if (
        message.includes("was not confirmed") ||
        message.includes("has already been processed")
      ) {
        if (sig) {
          console.log("Tx was not confirmed in 30s, refetching...");
          await sleep(1_000);
          const CONFIRMED_CONNECTION = getConnection();
          const result = await CONFIRMED_CONNECTION.getTransaction(sig);
          if (!!result?.meta && !result.meta.err) {
            toast.success(sig);
            return { success: true, signature: sig };
          }
        }

        toast.error(
          "Solana network is congested, the validator was unable to confirm that your transaction was succesful. Please inspect the transaction on the explorer"
        );
      } else if (message.includes("custom program error: 0x1")) {
        toast.error("No rewards to claim");
      } else {
        toast.error();
      }
    }
    return { success: false };
  }
};
