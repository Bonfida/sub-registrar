export interface Price {
  length: number;
  price: number;
}

export type Schedule = Price[];

export const formatSchedule = (schedule: Schedule): bigint[][] => {
  const result: bigint[][] = [];
  schedule.forEach((s) => result.push([BigInt(s.length), BigInt(s.price)]));
  return result;
};

export * from "./mint-record";
export * from "./registrar";
export * from "./subrecord";
