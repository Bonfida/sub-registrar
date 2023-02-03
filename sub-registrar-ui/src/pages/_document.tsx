import { Html, Head, Main, NextScript } from "next/document";

export default function Document() {
  return (
    <Html lang="en">
      <Head>
        <meta name="description" content="Bonfida subdomain registrar" />
        <link rel="icon" type="image/svg+xml" href="favicon.svg" />
        <meta name="theme-color" content="#03001A" />
        <meta name="robots" content="index,follow" />
      </Head>
      <body className="overflow-x-hidden bg-bds-dark-blues-DB900">
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
