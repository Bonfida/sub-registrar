import { PublicKey } from "@solana/web3.js";
import { Buffer } from "buffer";

const PREFIX = Buffer.from("metadata");
const PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

export const getMetadataKeyFromMint = (mint: PublicKey) => {
  const [key] = PublicKey.findProgramAddressSync(
    [PREFIX, PROGRAM_ID.toBuffer(), mint.toBuffer()],
    PROGRAM_ID
  );
  return key;
};
