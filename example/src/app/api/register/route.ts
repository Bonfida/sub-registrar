import { NextRequest, NextResponse } from "next/server";
import {
  getDomainKeySync,
  NAME_PROGRAM_ID,
  transferInstruction,
} from "@bonfida/spl-name-service";
import { adminRegister, Registrar } from "@bonfida/sub-register";
import {
  Connection,
  Keypair,
  PublicKey,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import bs58 from "bs58";
import { isValidSubdomain } from "@/utils/string";

const ADMIN_KEYPAIR = Keypair.fromSecretKey(
  new Uint8Array(bs58.decode(process.env.PRIVATE_KEY!))
);
const CONNECTION = new Connection(process.env.NEXT_PUBLIC_RPC!, "processed");

const getSubRegistrar = async () => {
  const registrars = await Registrar.findForDomain(
    CONNECTION,
    getDomainKeySync(process.env.NEXT_PUBLIC_DOMAIN_NAME!).pubkey
  );
  if (registrars.length === 0) {
    throw new Error("Subdomain registrar not found");
  }
  return registrars[0];
};

const sendAndConfirmTxWithRetry = async ({
  instructions,
  maxRetry,
  retryDelay,
}: {
  instructions: TransactionInstruction[];
  maxRetry: number;
  retryDelay: number;
}) => {
  for (let attempt = 0; attempt < maxRetry; attempt++) {
    try {
      const latestBlockhash = await CONNECTION.getLatestBlockhash();

      const messageV0 = new TransactionMessage({
        payerKey: ADMIN_KEYPAIR.publicKey,
        recentBlockhash: latestBlockhash.blockhash,
        instructions,
      }).compileToV0Message();

      const versionedTransaction = new VersionedTransaction(messageV0);
      versionedTransaction.sign([ADMIN_KEYPAIR]);

      const txid = await CONNECTION.sendTransaction(versionedTransaction);
      const confirmation = await CONNECTION.confirmTransaction({
        signature: txid,
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      });

      if (confirmation.value.err) {
        throw new Error("Transaction not confirmed");
      } else {
        return txid;
      }
    } catch {
      await new Promise((resolve) => setTimeout(resolve, retryDelay));
    }
  }
  throw new Error(`Transaction failed after ${maxRetry} retries`);
};

/**
 * Registers a subdomain and transfer it to the target account
 * @api {post} /api/register
 * @apiBody {String} publicKey    Target account public key
 * @apiBody {String} subdomain    Subdomain to be registered
 * @apiSuccess {String} txid      Transaction ID
 * @apiError {String} error       Error detail
 */
export const POST = async (request: NextRequest) => {
  const { publicKey: targetPublicKey, subdomain } = await request.json();

  // TODO: Implement additional validation/filtering as needed, such as
  // - signature validation
  // - publicKey filtering
  // - IP filtering
  // - language checks
  // - admin account balance check
  // - rate limiting

  if (!isValidSubdomain(subdomain)) {
    return NextResponse.json(
      { success: false, error: "Invalid subdomain" },
      { status: 400 }
    );
  }

  try {
    const subRegistrar = await getSubRegistrar();
    const domainKey = getDomainKeySync(
      `${subdomain}.${process.env.NEXT_PUBLIC_DOMAIN_NAME}`
    );
    const instructions = [
      // Register subdomain as admin
      ...(await adminRegister(
        CONNECTION,
        subRegistrar.pubkey,
        subdomain,
        ADMIN_KEYPAIR.publicKey
      )),
      // Sends registered subdomain to target account
      transferInstruction(
        NAME_PROGRAM_ID,
        domainKey.pubkey,
        new PublicKey(targetPublicKey),
        ADMIN_KEYPAIR.publicKey,
        undefined,
        domainKey.parent,
        ADMIN_KEYPAIR.publicKey
      ),
    ];

    const txid = await sendAndConfirmTxWithRetry({
      instructions,
      maxRetry: 3,
      retryDelay: 2500,
    });

    return NextResponse.json({ success: true, txid });
  } catch (e) {
    return NextResponse.json(
      { success: false, error: (e as Error).message },
      { status: 400 }
    );
  }
};
