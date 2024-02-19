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
  formatSchedule,
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
 * Devnet program ID (might not have the latest version deployed!)
 */
export const SUB_REGISTER_ID_DEVNET = new PublicKey(
  "2KkyPzjaAYaz2ojQZ9P3xYakLd96B5UH6a2isLaZ4Cgs"
);

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
  const [registrar] = Registrar.findKey(pubkey, authority, SUB_REGISTER_ID);
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
