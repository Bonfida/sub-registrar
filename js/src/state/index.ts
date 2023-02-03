import { deserialize, Schema } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Tag } from "./tag";

export interface Price {
  length: number;
  price: number;
}

export type Schedule = Price[];

export const formatSchedule = (schedule: Schedule): BN[][] => {
  const result: BN[][] = [];
  schedule.forEach((s) => result.push([new BN(s.length), new BN(s.price)]));
  return result;
};

export * from "./mint-record";
export * from "./registrar";
export * from "./subrecord";
