import { sendRawTransaction } from "@bonfida/hooks";
import {
  Transaction,
  Connection,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import chunk from "lodash/chunk";
import { Toast } from "@bonfida/components";
import { getAllConnections } from "./make-tx";

export const sendBatch = async (
  connection: Connection,
  publicKey: PublicKey,
  toast: Toast,
  instructions: TransactionInstruction[],
  txs: Transaction[],
  signAllTransactions: (transaction: Transaction[]) => Promise<Transaction[]>,
  chunkSize = 5
) => {
  try {
    toast.processing();

    const { blockhash } = await connection.getLatestBlockhash();
    let transactions = chunk(instructions, chunkSize).map((e) =>
      new Transaction().add(...e)
    );

    transactions.push(...txs);

    transactions.forEach((e) => {
      e.recentBlockhash = blockhash;
      e.feePayer = publicKey;
    });

    let idx = 0;
    const len = transactions.length;

    const signed = await signAllTransactions(transactions);

    const connections = [connection, ...getAllConnections([])];

    for (let signedTransaction of signed) {
      idx += 1;
      console.log(`Sending ${idx}/${len}`);
      const txid = await sendRawTransaction(connections, signedTransaction);
      await connection.confirmTransaction(txid, "processed");
    }

    toast.success("all");
  } catch (err) {
    console.log(err);
    // @ts-expect-error
    toast.error(err.message);
  }
};
