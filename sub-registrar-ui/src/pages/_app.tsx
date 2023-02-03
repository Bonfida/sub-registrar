import "@/styles/globals.css";
import type { AppProps } from "next/app";
import { Layout } from "../components/navigation-frame/Layout";
import { useMemo } from "react";
import { RPC_URL, WSS_URL } from "../settings/rpc";
import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
  Coin98WalletAdapter,
  CloverWalletAdapter,
  TorusWalletAdapter,
  MathWalletAdapter,
  GlowWalletAdapter,
  BraveWalletAdapter,
  CoinbaseWalletAdapter,
  HuobiWalletAdapter,
  BackpackWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { tokenAuthFetchMiddleware } from "@strata-foundation/web3-token-auth";
import { getToken } from "@bonfida/hooks";
import { ToastContextProvider } from "@bonfida/components";
import { ModalContextProvider } from "../context/modal";

function MyApp({ Component, pageProps }: AppProps) {
  const network = WalletAdapterNetwork.Mainnet;

  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new BackpackWalletAdapter(),
      new SolflareWalletAdapter(),
      new TorusWalletAdapter(),
      new MathWalletAdapter(),
      new Coin98WalletAdapter(),
      new CloverWalletAdapter(),
      new GlowWalletAdapter(),
      new BraveWalletAdapter(),
      new Coin98WalletAdapter(),
      new HuobiWalletAdapter(),
      new CoinbaseWalletAdapter(),
    ],
    [network]
  );

  const endpoint = useMemo(() => RPC_URL, []);

  return (
    <ToastContextProvider>
      <ModalContextProvider>
        <ConnectionProvider
          config={{
            commitment: "processed",
            confirmTransactionInitialTimeout: 15 * 1_000,
            wsEndpoint: WSS_URL,
            fetchMiddleware: tokenAuthFetchMiddleware({
              getToken,
              tokenExpiry: 2.5 * 60 * 1_000,
            }),
          }}
          endpoint={endpoint as string}
        >
          <WalletProvider wallets={wallets} autoConnect>
            <WalletModalProvider>
              <Layout>
                <Component {...pageProps} />
              </Layout>
            </WalletModalProvider>
          </WalletProvider>
        </ConnectionProvider>
      </ModalContextProvider>
    </ToastContextProvider>
  );
}

export default MyApp;
