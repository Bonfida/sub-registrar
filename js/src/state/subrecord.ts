import { deserialize } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import { Tag } from "./tag";

export class SubRecord {
  static SEED = "subrecord";
  tag: Tag;
  registrar: PublicKey;
  subKey: PublicKey;
  mintRecord: PublicKey | undefined;

  static schema = {
    struct: {
      tag: "u8",
      registrar: { array: { type: "u8", len: 32 } },
      subKey: { array: { type: "u8", len: 32 } },
      mintRecord: { option: { array: { type: "u8", len: 32 } } },
    },
  };

  constructor(obj: {
    tag: number;
    registrar: Uint8Array;
    subKey: Uint8Array;
    mintRecord: Uint8Array | null;
  }) {
    this.tag = obj.tag as Tag;
    this.registrar = new PublicKey(obj.registrar);
    this.subKey = new PublicKey(obj.subKey);
    this.mintRecord = obj.mintRecord
      ? new PublicKey(obj.mintRecord)
      : undefined;
  }

  static deserialize(data: Buffer): SubRecord {
    return new SubRecord(deserialize(this.schema, data) as any);
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
