import { Connection } from "@solana/web3.js";
import { RPC_URL } from "../../settings/rpc";

export const getConnection = () => {
  const token = localStorage.getItem("auth-token");

  const connection = new Connection(RPC_URL, {
    httpHeaders: { Authorization: `Bearer ${token}` },
  });

  return connection;
};
