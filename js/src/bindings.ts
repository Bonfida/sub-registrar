import {
  getDomainKey,
  getReverseKey,
  NAME_PROGRAM_ID,
  performReverseLookup,
  REVERSE_LOOKUP_CLASS,
  ROOT_DOMAIN_ACCOUNT,
} from "@bonfida/spl-name-service";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
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
import {
  formatSchedule,
  MintRecord,
  Registrar,
  Schedule,
  SubRecord,
} from "./state";
import { Metaplex } from "@metaplex-foundation/js";

/**
 * Mainnet program ID
 */
export const SUB_REGISTER_ID = new PublicKey(""); //TODO

const NAME_AUCTIONING_ID = PublicKey.default;

const FEE_OWNER = PublicKey.default;

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
    TOKEN_PROGRAM_ID
  );
  return [ix];
};

export const editRegistrar = async (
  connection: Connection,
  registrar: PublicKey,
  newAuthority: PublicKey | undefined,
  newMint: PublicKey | undefined,
  newFeeAccount: PublicKey | undefined,
  newPriceSchedule: Schedule | undefined,
  newMaxNftMint: number | undefined
) => {
  const obj = await Registrar.retrieve(connection, registrar);
  const ix = new editRegistrarInstruction({
    newAuthority: newAuthority ? newAuthority.toBuffer() : null,
    newMint: newMint ? newMint.toBuffer() : null,
    newFeeAccount: newFeeAccount ? newFeeAccount.toBuffer() : null,
    newPriceSchedule: newPriceSchedule
      ? formatSchedule(newPriceSchedule)
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
  subDomain: string
) => {
  const metaplex = new Metaplex(connection);
  const obj = await Registrar.retrieve(connection, registrar);
  const parent = await performReverseLookup(connection, obj.domain);

  const { pubkey } = await getDomainKey(subDomain + "." + parent);
  const reverseKey = await getReverseKey(subDomain + "." + parent, true);

  let nftAccount: PublicKey | undefined = undefined;
  let nftMetadata: PublicKey | undefined = undefined;
  let nftMintRecord: PublicKey | undefined = undefined;
  if (obj.nftGatedCollection) {
    // TODO
    let nfts = await metaplex.nfts().findAllByOwner({ owner: buyer });
  }

  const [subRecord] = SubRecord.findKey(pubkey, SUB_REGISTER_ID);

  const feeSource = getAssociatedTokenAddressSync(obj.mint, buyer, true);
  const bonfidaFee = getAssociatedTokenAddressSync(obj.mint, FEE_OWNER);

  const ix = new registerInstruction({ domain: subDomain }).getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    TOKEN_PROGRAM_ID,
    NAME_PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    NAME_AUCTIONING_ID,
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

  return [ix];
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

  const ix = new deleteSubrecordInstruction().getInstruction(
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
  const parent = await performReverseLookup(connection, obj.domain);
  const { pubkey } = await getDomainKey(subDomain + "." + parent);
  const [subRecord] = SubRecord.findKey(obj.domain, SUB_REGISTER_ID);

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
  const { pubkey } = await getDomainKey(subDomain + "." + parent);
  const reverse = await getReverseKey(subDomain + "." + parent, true);
  const [subRecord] = SubRecord.findKey(pubkey, SUB_REGISTER_ID);

  const ix = new adminRegisterInstruction({
    domain: subDomain,
  }).getInstruction(
    SUB_REGISTER_ID,
    SystemProgram.programId,
    TOKEN_PROGRAM_ID,
    NAME_PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    NAME_AUCTIONING_ID,
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
  const metaplex = new Metaplex(connection);
  const obj = await Registrar.retrieve(connection, registrar);
  const [subRecord] = SubRecord.findKey(subDomainAccount, SUB_REGISTER_ID);
  const subRecordObj = await SubRecord.retrieve(connection, subRecord);

  // TODO

  const ix = new nftOwnerRevokeInstruction().getInstruction(
    SUB_REGISTER_ID,
    registrar,
    subDomainAccount,
    subRecord,
    subOwner,
    obj.domain,
    nftOwner,
    nftAccount,
    nftMetadata,
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
  const parent = await performReverseLookup(connection, obj.domain);
  const { pubkey } = await getDomainKey(subDomain + "." + parent);
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
