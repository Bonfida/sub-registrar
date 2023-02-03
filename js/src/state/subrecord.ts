import { deserialize, Schema } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Tag } from "./tag";

export class SubRecord {
  static SEED = "subrecord";
  tag: Tag;
  registrar: PublicKey;
  mintRecord: PublicKey;

  static schema: Schema = new Map([
    [
      SubRecord,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["registrar", [32]],
          ["mintRecord", { kind: "option", type: [32] }],
        ],
      },
    ],
  ]);

  constructor(obj: { tag: BN; registrar: Uint8Array; mintRecord: Uint8Array }) {
    this.tag = obj.tag.toNumber() as Tag;
    this.registrar = new PublicKey(obj.registrar);
    this.mintRecord = new PublicKey(obj.mintRecord);
  }

  static deserialize(data: Buffer): SubRecord {
    return deserialize(this.schema, SubRecord, data);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("State account not found");
    }
    return this.deserialize(accountInfo.data);
  }
  static findKey(domain: PublicKey, programId: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(SubRecord.SEED), domain.toBuffer()],
      programId
    );
  }
}
