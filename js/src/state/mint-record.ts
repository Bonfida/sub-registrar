import { deserialize } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import { Tag } from "./tag";

// MintRecords are used to keep track of how many domains were minted via a specific NFT ownership.
export class MintRecord {
  static SEED = "nft_mint_record";
  tag: Tag;
  count: number;
  mint: PublicKey;

  static schema = {
    struct: {
      tag: "u8",
      count: "u8",
      mint: { array: { type: "u8", len: 32 } },
    },
  };

  constructor(obj: { tag: bigint; count: number; mint: Uint8Array }) {
    this.tag = Number(obj.tag) as Tag;
    this.count = obj.count;
    this.mint = new PublicKey(obj.mint);
  }

  static deserialize(data: Buffer): MintRecord {
    return new MintRecord(deserialize(this.schema, data) as any);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("State account not found");
    }
    return this.deserialize(accountInfo.data);
  }
  static findKey(registrar: PublicKey, mint: PublicKey, programId: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(MintRecord.SEED), registrar.toBuffer(), mint.toBuffer()],
      programId
    );
  }
}
