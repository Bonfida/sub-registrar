import { getDomainKey, NAME_PROGRAM_ID } from "@bonfida/spl-name-service";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import {
  deleteSubrecordInstruction,
  editRegistrarInstruction,
  adminRegisterInstruction,
  adminRevokeInstruction,
  closeRegistrarInstruction,
  registerInstruction,
  createRegistrarInstruction,
  nftOwnerRevokeInstruction,
  unregisterInstruction,
} from "./raw_instructions";
import { formatSchedule, Registrar, Schedule } from "./state";

/**
 * Mainnet program ID
 */
export const SUB_REGISTER_ID = new PublicKey(""); //TODO

/**
 * Devnet program ID (might not have the latest version deployed!)
 */
export const SUB_REGISTER_ID_DEVNET = new PublicKey(""); //TODO

export const createRegistrar = async (
  domain: string,
  domainOwner: PublicKey,
  feePayer: PublicKey,
  mint: PublicKey,
  authority: PublicKey,
  schedule: Schedule,
  feeAccount: PublicKey,
  nftGatedCollection: PublicKey | null,
  maxNftMint: number | null,
  allowRevoke: boolean
) => {
  const { pubkey } = await getDomainKey(domain);
  const [registrar] = Registrar.findKey(pubkey, authority, SUB_REGISTER_ID);
  const ix = new createRegistrarInstruction({
    mint: mint.toBuffer(),
    feeAccount: feeAccount.toBuffer(),
    authority: authority.toBuffer(),
    nftGatedCollection: nftGatedCollection
      ? nftGatedCollection.toBuffer()
      : null,
    maxNftMint: maxNftMint || 0,
    allowRevoke: allowRevoke ? 1 : 0,
    priceSchedule: formatSchedule(schedule),
  }).getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    registrar,
    pubkey,
    domainOwner,
    feePayer,
    NAME_PROGRAM_ID
  );
  return [ix];
};
