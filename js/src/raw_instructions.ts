// This file is auto-generated. DO NOT EDIT
import { serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}
export class editRegistrarInstruction {
  tag: number;
  newAuthority: Uint8Array | null;
  newMint: Uint8Array | null;
  newFeeAccount: Uint8Array | null;
  newPriceSchedule: number[] | null;
  newMaxNftMint: number | null;
  static schema = {
    struct: {
      tag: "u8",
      newAuthority: { option: { array: { type: "u8", len: 32 } } },
      newMint: { option: { array: { type: "u8", len: 32 } } },
      newFeeAccount: { option: { array: { type: "u8", len: 32 } } },
      newPriceSchedule: { option: { array: { type: "u8" } } },
      newMaxNftMint: { option: "u8" },
    },
  };
  constructor(obj: {
    newAuthority: Uint8Array | null;
    newMint: Uint8Array | null;
    newFeeAccount: Uint8Array | null;
    newPriceSchedule: number[] | null;
    newMaxNftMint: number | null;
  }) {
    this.tag = 1;
    this.newAuthority = obj.newAuthority;
    this.newMint = obj.newMint;
    this.newFeeAccount = obj.newFeeAccount;
    this.newPriceSchedule = obj.newPriceSchedule;
    this.newMaxNftMint = obj.newMaxNftMint;
  }
  serialize(): Uint8Array {
    return serialize(editRegistrarInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    systemProgram: PublicKey,
    authority: PublicKey,
    registrar: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class adminRevokeInstruction {
  tag: number;
  static schema = {
    struct: {
      tag: "u8",
    },
  };
  constructor() {
    this.tag = 7;
  }
  serialize(): Uint8Array {
    return serialize(adminRevokeInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    registrar: PublicKey,
    subDomainAccount: PublicKey,
    subRecord: PublicKey,
    subOwner: PublicKey,
    parentDomain: PublicKey,
    authority: PublicKey,
    nameClass: PublicKey,
    splNameService: PublicKey,
    mintRecord?: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subDomainAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subOwner,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: parentDomain,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: nameClass,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splNameService,
      isSigner: false,
      isWritable: false,
    });
    if (!!mintRecord) {
      keys.push({
        pubkey: mintRecord,
        isSigner: false,
        isWritable: true,
      });
    }
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class closeRegistrarInstruction {
  tag: number;
  static schema = {
    struct: {
      tag: "u8",
    },
  };
  constructor() {
    this.tag = 4;
  }
  serialize(): Uint8Array {
    return serialize(closeRegistrarInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    systemProgram: PublicKey,
    registrar: PublicKey,
    domainNameAccount: PublicKey,
    newDomainOwner: PublicKey,
    lamportsTarget: PublicKey,
    registryAuthority: PublicKey,
    splNameProgramId: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: domainNameAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: newDomainOwner,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: lamportsTarget,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: registryAuthority,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: splNameProgramId,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class registerInstruction {
  tag: number;
  domain: string;
  static schema = {
    struct: {
      tag: "u8",
      domain: "string",
    },
  };
  constructor(obj: { domain: string }) {
    this.tag = 2;
    this.domain = obj.domain;
  }
  serialize(): Uint8Array {
    return serialize(registerInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    systemProgram: PublicKey,
    splTokenProgram: PublicKey,
    splNameService: PublicKey,
    rentSysvar: PublicKey,
    snsRegistrarProgram: PublicKey,
    rootDomain: PublicKey,
    reverseLookupClass: PublicKey,
    feeAccount: PublicKey,
    feeSource: PublicKey,
    registrar: PublicKey,
    parentDomainAccount: PublicKey,
    subDomainAccount: PublicKey,
    subReverseAccount: PublicKey,
    feePayer: PublicKey,
    bonfidaFeeAccount: PublicKey,
    subRecord: PublicKey,
    nftAccount?: PublicKey,
    nftMetadataAccount?: PublicKey,
    nftMintRecord?: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splNameService,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: rentSysvar,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: snsRegistrarProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: rootDomain,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: reverseLookupClass,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: feeAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: feeSource,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: parentDomainAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subDomainAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subReverseAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: bonfidaFeeAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subRecord,
      isSigner: false,
      isWritable: true,
    });
    if (!!nftAccount) {
      keys.push({
        pubkey: nftAccount,
        isSigner: false,
        isWritable: false,
      });
    }
    if (!!nftMetadataAccount) {
      keys.push({
        pubkey: nftMetadataAccount,
        isSigner: false,
        isWritable: false,
      });
    }
    if (!!nftMintRecord) {
      keys.push({
        pubkey: nftMintRecord,
        isSigner: false,
        isWritable: true,
      });
    }
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class createRegistrarInstruction {
  tag: number;
  mint: Uint8Array;
  feeAccount: Uint8Array;
  authority: Uint8Array;
  priceSchedule: number[];
  nftGatedCollection: Uint8Array | null;
  maxNftMint: number;
  allowRevoke: boolean;
  static schema = {
    struct: {
      tag: "u8",
      mint: { array: { type: "u8", len: 32 } },
      feeAccount: { array: { type: "u8", len: 32 } },
      authority: { array: { type: "u8", len: 32 } },
      priceSchedule: { array: { type: "u8" } },
      nftGatedCollection: { option: { array: { type: "u8", len: 32 } } },
      maxNftMint: "u8",
      allowRevoke: "bool",
    },
  };
  constructor(obj: {
    mint: Uint8Array;
    feeAccount: Uint8Array;
    authority: Uint8Array;
    priceSchedule: number[];
    nftGatedCollection: Uint8Array | null;
    maxNftMint: number;
    allowRevoke: boolean;
  }) {
    this.tag = 0;
    this.mint = obj.mint;
    this.feeAccount = obj.feeAccount;
    this.authority = obj.authority;
    this.priceSchedule = obj.priceSchedule;
    this.nftGatedCollection = obj.nftGatedCollection;
    this.maxNftMint = obj.maxNftMint;
    this.allowRevoke = obj.allowRevoke;
  }
  serialize(): Uint8Array {
    return serialize(createRegistrarInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    systemProgram: PublicKey,
    registrar: PublicKey,
    domainNameAccount: PublicKey,
    domainOwner: PublicKey,
    feePayer: PublicKey,
    splNameProgramId: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: domainNameAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: domainOwner,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: splNameProgramId,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class adminRegisterInstruction {
  tag: number;
  domain: string;
  static schema = {
    struct: {
      tag: "u8",
      domain: "string",
    },
  };
  constructor(obj: { domain: string }) {
    this.tag = 5;
    this.domain = obj.domain;
  }
  serialize(): Uint8Array {
    return serialize(adminRegisterInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    systemProgram: PublicKey,
    splTokenProgram: PublicKey,
    splNameService: PublicKey,
    rentSysvar: PublicKey,
    snsRegistrarProgram: PublicKey,
    rootDomain: PublicKey,
    reverseLookupClass: PublicKey,
    registrar: PublicKey,
    parentDomainAccount: PublicKey,
    subDomainAccount: PublicKey,
    subReverseAccount: PublicKey,
    subRecord: PublicKey,
    authority: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splNameService,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: rentSysvar,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: snsRegistrarProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: rootDomain,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: reverseLookupClass,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: parentDomainAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subDomainAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subReverseAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class nftOwnerRevokeInstruction {
  tag: number;
  static schema = {
    struct: {
      tag: "u8",
    },
  };
  constructor() {
    this.tag = 8;
  }
  serialize(): Uint8Array {
    return serialize(nftOwnerRevokeInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    registrar: PublicKey,
    subDomainAccount: PublicKey,
    subRecord: PublicKey,
    subOwner: PublicKey,
    parentDomain: PublicKey,
    nftOwner: PublicKey,
    nftAccount: PublicKey,
    nftMetadata: PublicKey,
    nftMintRecord: PublicKey,
    nameClass: PublicKey,
    splNameService: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subDomainAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subOwner,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: parentDomain,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: nftOwner,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: nftAccount,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: nftMetadata,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: nftMintRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nameClass,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splNameService,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class unregisterInstruction {
  tag: number;
  static schema = {
    struct: {
      tag: "u8",
    },
  };
  constructor() {
    this.tag = 3;
  }
  serialize(): Uint8Array {
    return serialize(unregisterInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    systemProgram: PublicKey,
    splNameService: PublicKey,
    registrar: PublicKey,
    subDomainAccount: PublicKey,
    subRecord: PublicKey,
    domainOwner: PublicKey,
    mintRecord?: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splNameService,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subDomainAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: domainOwner,
      isSigner: true,
      isWritable: true,
    });
    if (!!mintRecord) {
      keys.push({
        pubkey: mintRecord,
        isSigner: false,
        isWritable: true,
      });
    }
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class deleteSubdomainRecordInstruction {
  tag: number;
  static schema = {
    struct: {
      tag: "u8",
    },
  };
  constructor() {
    this.tag = 6;
  }
  serialize(): Uint8Array {
    return serialize(deleteSubdomainRecordInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    registrar: PublicKey,
    subDomain: PublicKey,
    subRecord: PublicKey,
    lamportsTarget: PublicKey,
    mintRecord?: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: registrar,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subDomain,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: subRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: lamportsTarget,
      isSigner: false,
      isWritable: true,
    });
    if (!!mintRecord) {
      keys.push({
        pubkey: mintRecord,
        isSigner: false,
        isWritable: true,
      });
    }
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
