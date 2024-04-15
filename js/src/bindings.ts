import {
  getDomainKeySync,
  getReverseKeySync,
  NAME_PROGRAM_ID,
  reverseLookup,
  REVERSE_LOOKUP_CLASS,
  ROOT_DOMAIN_ACCOUNT,
  REGISTER_PROGRAM_ID,
} from "@bonfida/spl-name-service";
import {
  AccountLayout,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  deleteSubdomainRecordInstruction,
  editRegistrarInstruction,
  adminRegisterInstruction,
  adminRevokeInstruction,
  closeRegistrarInstruction,
  registerInstruction,
  createRegistrarInstruction,
  nftOwnerRevokeInstruction,
  unregisterInstruction,
} from "./raw_instructions";
import {
  MintRecord,
  Registrar,
  Schedule,
  serializePriceSchedule,
  SubRecord,
} from "./state";

import { getMetadataKeyFromMint } from "./utils";

/**
 * Mainnet program ID
 */
export const SUB_REGISTER_ID = new PublicKey(
  "2KkyPzjaAYaz2ojQZ9P3xYakLd96B5UH6a2isLaZ4Cgs"
);

const FEE_OWNER = new PublicKey("5D2zKog251d6KPCyFyLMt3KroWwXXPWSgTPyhV22K2gR");

/**
 * Creates a subdomain registrar with the provided parameters.
 * @param domain - The domain name as a string.
 * @param domainOwner - The public key of the domain owner.
 * @param feePayer - The public key of the entity paying for the transaction fees and account allocation.
 * @param mint - The public key of the mint used for subdomain issuance payment.
 * @param authority - The public key of the authority. Used for managing the registrar.
 * @param schedule - An array of `Schedule` objects defining the price schedule.
 * @param feeAccount - The public key of the fee account. Must be a token account for the given `mint`.
 * @param nftGatedCollection - The public key of the NFT gated collection, or null if not applicable.
 * @param maxNftMint - The maximum number of NFTs that can be minted, or null if not applicable.
 * @param allowRevoke - A boolean indicating whether revoking by `authority` is allowed.
 * @returns A promise that resolves to an array containing the transaction instruction.
 */
export const createRegistrar = async (
  domain: string,
  domainOwner: PublicKey,
  feePayer: PublicKey,
  mint: PublicKey,
  authority: PublicKey,
  schedule: Schedule[],
  feeAccount: PublicKey,
  nftGatedCollection: PublicKey | null,
  maxNftMint: number | null,
  allowRevoke: boolean
) => {
  const { pubkey } = getDomainKeySync(domain);
  const [registrar] = Registrar.findKey(pubkey, SUB_REGISTER_ID);
  const ix = new createRegistrarInstruction({
    mint: mint.toBuffer(),
    feeAccount: feeAccount.toBuffer(),
    authority: authority.toBuffer(),
    nftGatedCollection: nftGatedCollection
      ? nftGatedCollection.toBuffer()
      : null,
    maxNftMint: maxNftMint || 0,
    allowRevoke,
    priceSchedule: Array.from(serializePriceSchedule(schedule)),
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

/**
 * Closes a subdomain registrar and transfers remaining lamports to a target account.
 * @param connection - The Solana blockchain connection object.
 * @param registrar - The public key of the registrar to close.
 * @param authority - The public key of the current authority of the registrar.
 * @param newDomainOwner - The public key of the new owner of the domain.
 * @param lamportsTarget - The public key of the account to receive the remaining lamports.
 * @returns A promise that resolves to an array containing the transaction instruction.
 */
export const closeRegistrar = async (
  connection: Connection,
  registrar: PublicKey,
  authority: PublicKey,
  newDomainOwner: PublicKey,
  lamportsTarget: PublicKey
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const ix = new closeRegistrarInstruction().getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    registrar,
    obj.domain,
    newDomainOwner,
    lamportsTarget,
    authority,
    NAME_PROGRAM_ID
  );
  return [ix];
};

/**
 * Updates the registrar with new parameters.
 * @param connection - The Solana blockchain connection object.
 * @param registrar - The public key of the registrar to update.
 * @param newAuthority - The new authority public key, if updating.
 * @param newMint - The new mint public key, if updating.
 * @param newFeeAccount - The new fee account public key, if updating. Must be a token account for the current mint.
 * @param newPriceSchedule - The new price schedule array, if updating.
 * @param newMaxNftMint - The new maximum NFT mint count, if updating.
 * @returns A promise that resolves to an array containing the transaction instruction.
 */
export const editRegistrar = async (
  connection: Connection,
  registrar: PublicKey,
  newAuthority: PublicKey | undefined,
  newMint: PublicKey | undefined,
  newFeeAccount: PublicKey | undefined,
  newPriceSchedule: Schedule[] | undefined,
  newMaxNftMint: number | undefined
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const ix = new editRegistrarInstruction({
    newAuthority: newAuthority ? newAuthority.toBuffer() : null,
    newMint: newMint ? newMint.toBuffer() : null,
    newFeeAccount: newFeeAccount ? newFeeAccount.toBuffer() : null,
    newPriceSchedule: newPriceSchedule
      ? Array.from(serializePriceSchedule(newPriceSchedule))
      : null,
    newMaxNftMint: newMaxNftMint ? newMaxNftMint : null,
  }).getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    obj.authority,
    registrar
  );
  return [ix];
};

/**
 * Registers a new subdomain.
 * @param connection - The Solana blockchain connection object.
 * @param registrar - The public key of the registrar responsible for the domain.
 * @param buyer - The public key of the buyer who is registering the subdomain.
 * @param nftAccount - The public key of the NFT account used for gated access, if applicable.
 * @param subDomain - The name of the subdomain being registered.
 * @returns A promise that resolves to an array containing the transaction instructions.
 */
export const register = async (
  connection: Connection,
  registrar: PublicKey,
  buyer: PublicKey,
  nftAccount: PublicKey,
  subDomain: string
) => {
  const ixs: TransactionInstruction[] = [];
  const obj = await Registrar.retrieve(connection, registrar);
  const parent = await reverseLookup(connection, obj.domain);

  const { pubkey } = getDomainKeySync(subDomain + "." + parent);
  const reverseKey = getReverseKeySync(subDomain + "." + parent, true);

  let nftMetadata: PublicKey | undefined = undefined;
  let nftMintRecord: PublicKey | undefined = undefined;
  if (obj.nftGatedCollection) {
    const nftInfo = await connection.getAccountInfo(nftAccount);
    if (!nftInfo) {
      throw new Error("NFT account info not found");
    }
    const des = AccountLayout.decode(nftInfo.data);
    nftMetadata = getMetadataKeyFromMint(des.mint);
    nftMintRecord = MintRecord.findKey(registrar, des.mint, SUB_REGISTER_ID)[0];
  }

  const [subRecord] = SubRecord.findKey(pubkey, SUB_REGISTER_ID);

  const feeSource = getAssociatedTokenAddressSync(obj.mint, buyer, true);
  const bonfidaFee = getAssociatedTokenAddressSync(obj.mint, FEE_OWNER, true);

  if (!(await connection.getAccountInfo(bonfidaFee))) {
    const ixCreateFee = createAssociatedTokenAccountIdempotentInstruction(
      buyer,
      bonfidaFee,
      FEE_OWNER,
      obj.mint
    );
    ixs.push(ixCreateFee);
  }

  const ix = new registerInstruction({
    domain: `\0`.concat(subDomain),
  }).getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    TOKEN_PROGRAM_ID,
    NAME_PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    REGISTER_PROGRAM_ID,
    ROOT_DOMAIN_ACCOUNT,
    REVERSE_LOOKUP_CLASS,
    obj.feeAccount,
    feeSource,
    registrar,
    obj.domain,
    pubkey,
    reverseKey,
    buyer,
    bonfidaFee,
    subRecord,
    nftAccount,
    nftMetadata,
    nftMintRecord
  );
  ixs.push(ix);

  return ixs;
};

/**
 * Deletes a subrecord associated with a given subdomain.
 *
 * @param connection - The Solana blockchain connection to use.
 * @param registrar - The public key of the registrar responsible for the subdomain.
 * @param subDomain - The public key of the subdomain to delete.
 * @param lamportsTarget - The public key where the lamports will be transferred.
 * @returns A promise that resolves to an array containing the transaction instruction for deleting the subrecord.
 */
export const deleteSubrecord = async (
  connection: Connection,
  registrar: PublicKey,
  subDomain: PublicKey,
  lamportsTarget: PublicKey
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const [subRecord] = SubRecord.findKey(obj.domain, SUB_REGISTER_ID);

  let mintRecord: PublicKey | undefined = undefined;
  if (obj.nftGatedCollection) {
    const obj = await SubRecord.retrieve(connection, subRecord);
    mintRecord = obj.mintRecord;
  }

  const ix = new deleteSubdomainRecordInstruction().getInstruction(
    SUB_REGISTER_ID,
    registrar,
    subDomain,
    subRecord,
    lamportsTarget,
    mintRecord
  );
  return [ix];
};

/**
 * Unregisters a subdomain.
 *
 * @param connection - The Solana blockchain connection to use.
 * @param registrar - The public key of the registrar responsible for the subdomain.
 * @param subDomain - The name of the subdomain to unregister.
 * @param owner - The public key of the current owner of the subdomain.
 * @returns A promise that resolves to an array containing the transaction instruction for unregistering the subdomain.
 */
export const unregister = async (
  connection: Connection,
  registrar: PublicKey,
  subDomain: string,
  owner: PublicKey
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const parent = await reverseLookup(connection, obj.domain);
  const { pubkey } = getDomainKeySync(subDomain + "." + parent);
  const [subRecord] = SubRecord.findKey(pubkey, SUB_REGISTER_ID);

  let mintRecord: PublicKey | undefined = undefined;
  if (obj.nftGatedCollection) {
    const obj = await SubRecord.retrieve(connection, subRecord);
    mintRecord = obj.mintRecord;
  }

  const ix = new unregisterInstruction().getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    NAME_PROGRAM_ID,
    registrar,
    pubkey,
    subRecord,
    owner,
    mintRecord
  );
  return [ix];
};

/**
 * Registers a subdomain with admin authority. Bypassing the price/nft holding requirements.
 *
 * @param connection - The Solana blockchain connection to use.
 * @param registrar - The public key of the registrar responsible for the domain.
 * @param subDomain - The name of the subdomain to register.
 * @param authority - The public key of the administrative authority registering the subdomain.
 * @returns A promise that resolves to an array containing the transaction instruction for registering the subdomain.
 */
export const adminRegister = async (
  connection: Connection,
  registrar: PublicKey,
  subDomain: string,
  authority: PublicKey
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const parent = await reverseLookup(connection, obj.domain);
  const { pubkey } = getDomainKeySync(subDomain + "." + parent);
  const reverse = getReverseKeySync(subDomain + "." + parent, true);
  const [subRecord] = SubRecord.findKey(pubkey, SUB_REGISTER_ID);

  const ix = new adminRegisterInstruction({
    domain: `\0`.concat(subDomain),
  }).getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    TOKEN_PROGRAM_ID,
    NAME_PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    REGISTER_PROGRAM_ID,
    ROOT_DOMAIN_ACCOUNT,
    REVERSE_LOOKUP_CLASS,
    registrar,
    obj.domain,
    pubkey,
    reverse,
    subRecord,
    authority
  );
  return [ix];
};

/**
 * Revokes a subdomain if the current owner does not fulfill the NFT holding requirements anymore.
 *
 * @param connection - The Solana blockchain connection to use.
 * @param registrar - The public key of the registrar responsible for the domain.
 * @param subOwner - The public key of the current owner of the subdomain.
 * @param nftOwner - The public key of the NFT owner.
 * @param subDomainAccount - The public key of the subdomain account.
 * @returns A promise that resolves to an array containing the transaction instruction for revoking the subdomain.
 */
export const nftOwnerRevoke = async (
  connection: Connection,
  registrar: PublicKey,
  subOwner: PublicKey,
  nftOwner: PublicKey,
  subDomainAccount: PublicKey
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const [subRecord] = SubRecord.findKey(subDomainAccount, SUB_REGISTER_ID);
  const subRecordObj = await SubRecord.retrieve(connection, subRecord);

  if (!subRecordObj.mintRecord) {
    throw new Error("Mint record not found");
  }

  const mintRecord = await MintRecord.retrieve(
    connection,
    subRecordObj.mintRecord
  );

  const ix = new nftOwnerRevokeInstruction().getInstruction(
    SUB_REGISTER_ID,
    registrar,
    subDomainAccount,
    subRecord,
    subOwner,
    obj.domain,
    nftOwner,
    getAssociatedTokenAddressSync(mintRecord.mint, nftOwner, true),
    getMetadataKeyFromMint(mintRecord.mint),
    subRecordObj.mintRecord,
    PublicKey.default,
    NAME_PROGRAM_ID
  );
  return [ix];
};

/**
 * Revokes a subdomain by an admin authority if allowed by the registrar.
 *
 * @param connection - The Solana blockchain connection to use.
 * @param registrar - The public key of the registrar responsible for the domain.
 * @param subDomain - The name of the subdomain to be revoked.
 * @param owner - The public key of the current owner of the subdomain.
 * @param authority - The public key of the admin authority performing the revoke.
 * @returns A promise that resolves to an array containing the transaction instruction for revoking the subdomain.
 */
export const adminRevoke = async (
  connection: Connection,
  registrar: PublicKey,
  subDomain: string,
  owner: PublicKey,
  authority: PublicKey
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const parent = await reverseLookup(connection, obj.domain);
  const { pubkey } = getDomainKeySync(subDomain + "." + parent);
  const [subRecord] = SubRecord.findKey(obj.domain, SUB_REGISTER_ID);

  let mintRecord: PublicKey | undefined = undefined;
  if (obj.nftGatedCollection) {
    const obj = await SubRecord.retrieve(connection, subRecord);
    mintRecord = obj.mintRecord;
  }
  const ix = new adminRevokeInstruction().getInstruction(
    SUB_REGISTER_ID,
    registrar,
    pubkey,
    subRecord,
    owner,
    obj.domain,
    authority,
    PublicKey.default,
    NAME_PROGRAM_ID,
    mintRecord
  );

  return [ix];
};
