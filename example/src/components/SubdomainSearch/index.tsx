import { ChangeEventHandler, useState } from "react";
import Link from "next/link";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { getDomainKeySync, NAME_PROGRAM_ID } from "@bonfida/spl-name-service";
import { ExternalLink } from "@/components/icons/ExternalLink";
import { isValidSubdomain } from "@/utils/string";

enum Step {
  Searching,
  Processing,
  Success,
  Error,
}

export const SubdomainSearch = () => {
  const { connection } = useConnection();
  const { publicKey } = useWallet();

  const [step, setStep] = useState(Step.Searching);
  const [subdomain, setSubdomain] = useState("");
  const [errorText, setErrorText] = useState("");

  const onSearchChange: ChangeEventHandler<HTMLInputElement> = (e) => {
    const value = e.target.value.trim();
    setSubdomain(value);
    if (isValidSubdomain(value)) {
      setErrorText("");
    } else {
      setErrorText("Invalid Subdomain");
    }
  };

  const onSearch = async () => {
    const { pubkey } = getDomainKeySync(
      `${subdomain}.${process.env.NEXT_PUBLIC_DOMAIN_NAME}`
    );
    const info = await connection.getAccountInfo(pubkey);
    if (info?.owner?.equals(NAME_PROGRAM_ID)) {
      setErrorText("Subdomain Unavailable");
    } else {
      setStep(Step.Processing);
      await fetch(`/api/register`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          publicKey,
          subdomain,
        }),
      })
        .then((response) => response.json())
        .then((data) => {
          if (data.success) {
            setStep(Step.Success);
          } else {
            setErrorText(`Error: ${data.error}`);
            setStep(Step.Error);
          }
        })
        .catch(() => {
          setErrorText("Error: Request failure");
          setStep(Step.Error);
        });
    }
  };

  const reset = () => {
    setStep(Step.Searching);
    setSubdomain("");
    setErrorText("");
  };

  if (step === Step.Searching) {
    return (
      <>
        <div className="flex w-full">
          <input
            className="w-full text-ellipsis border-b-2 bg-transparent p-1 text-lg focus-within:outline-none"
            value={subdomain}
            placeholder={`Search for your .${process.env.NEXT_PUBLIC_DOMAIN_NAME} subdomain...`}
            onChange={onSearchChange}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                onSearch();
              }
            }}
          />
          <button
            className="px-3 text-2xl disabled:text-white/40"
            disabled={!!errorText}
            onClick={onSearch}
          >
            🡒
          </button>
        </div>
        {errorText && (
          <span className="w-full p-1 text-sm text-red-400">{errorText}</span>
        )}
      </>
    );
  }

  if (step === Step.Processing) {
    return <span className="animate-pulse text-xl">Processing...</span>;
  }

  if (step === Step.Success) {
    return (
      <>
        <span className="mb-4 text-xl text-green-400">
          Congratulations! You are now the proud owner of
        </span>
        <Link
          href={`https://sns.id/domain?domain=${subdomain}.${process.env.NEXT_PUBLIC_DOMAIN_NAME}`}
          target="_blank"
          className="mb-8 text-3xl font-semibold text-white"
        >
          {`${subdomain}.${process.env.NEXT_PUBLIC_DOMAIN_NAME}.sol`}
          <ExternalLink className="ml-1 inline-block size-6" />
        </Link>
        <button className="animate-pulse text-xl text-white/75" onClick={reset}>
          Try again 🡒
        </button>
      </>
    );
  }

  if (step === Step.Error) {
    return (
      <>
        <span className="mb-8 text-xl text-red-400">{errorText}</span>
        <button className="animate-pulse text-xl text-white/75" onClick={reset}>
          Try again 🡒
        </button>
      </>
    );
  }
};
