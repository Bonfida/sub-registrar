import { deserialize, Schema } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Tag } from "./tag";

export class MintRecord {
  static SEED = "nft_mint_record";
  tag: Tag;
  count: number;

  static schema: Schema = new Map([
    [
      MintRecord,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["nonce", "u8"],
        ],
      },
    ],
  ]);

  constructor(obj: { tag: BN; nonce: number; owner: Uint8Array }) {
    this.tag = obj.tag.toNumber() as Tag;
    this.count = obj.nonce;
  }

  static deserialize(data: Buffer): MintRecord {
    return deserialize(this.schema, MintRecord, data);
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
