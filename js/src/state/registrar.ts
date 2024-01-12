import { deserialize } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Tag } from "./tag";

export class Schedule {
  length: BN;
  price: BN;

  static schema = {
    struct: { length: "u64", price: "u64" },
  };

  constructor(obj: { price: BN; length: BN }) {
    this.length = obj.length;
    this.price = obj.price;
  }
}

export class Registrar {
  static SEED = "registrar";
  tag: Tag;
  nonce: number;
  authority: PublicKey;
  feeAccount: PublicKey;
  mint: PublicKey;
  domain: PublicKey;
  totalSubCreated: BN;
  nftGatedCollection: PublicKey | null;
  maxNftMint: number;
  allowRevoke: boolean;
  priceScedule: Schedule;

  static schema = {
    struct: {
      tag: "u8",
      nonce: "u8",
      authority: { array: { type: "u8", len: 32 } },
      feeAccount: { array: { type: "u8", len: 32 } },
      mint: { array: { type: "u8", len: 32 } },
      domain: { array: { type: "u8", len: 32 } },
      totalSubCreated: "u64",
      nftGatedCollection: { option: { array: { type: "u8", len: 32 } } },
      maxNftMint: "u8",
      allowRevoke: "u8",
      priceScedule: { array: { type: Schedule.schema } },
    },
  };

  constructor(obj: {
    tag: Tag;
    nonce: number;
    authority: Uint8Array;
    feeAccount: Uint8Array;
    mint: Uint8Array;
    domain: Uint8Array;
    totalSubCreated: BN;
    nftGatedCollection: Uint8Array | null;
    maxNftMint: number;
    allowRevoke: boolean;
    priceScedule: Schedule;
  }) {
    this.tag = obj.tag;
    this.nonce = obj.nonce;
    this.authority = new PublicKey(obj.authority);
    this.feeAccount = new PublicKey(obj.feeAccount);
    this.mint = new PublicKey(obj.mint);
    this.domain = new PublicKey(obj.domain);
    this.totalSubCreated = obj.totalSubCreated;
    this.nftGatedCollection = obj.nftGatedCollection
      ? new PublicKey(obj.nftGatedCollection)
      : null;
    this.maxNftMint = obj.maxNftMint;
    this.allowRevoke = obj.allowRevoke;
    this.priceScedule = obj.priceScedule;
  }

  static deserialize(data: Buffer): Registrar {
    return new Registrar(deserialize(this.schema, data) as any);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("State account not found");
    }
    return this.deserialize(accountInfo.data);
  }
  static findKey(
    domain: PublicKey,
    authority: PublicKey,
    programId: PublicKey
  ) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(Registrar.SEED), domain.toBuffer(), authority.toBuffer()],
      programId
    );
  }
}
