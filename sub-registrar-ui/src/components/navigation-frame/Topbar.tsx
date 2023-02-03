import { Logo, Button } from "@bonfida/components";
import { Link } from "../Link";
import { useScroll } from "ahooks";
import { WalletConnect } from "../WalletConnect";
import { twMerge } from "tailwind-merge";
import { useSmallScreen } from "@bonfida/hooks";
import Image from "next/image";
import { customLoader } from "../../utils/custom-loader";
import { useState } from "react";

const sections = [
  { name: "Trade", href: "/trade" },
  { name: "Stake", href: "/stake" },
  { name: "Buy & Burn", href: "/buy-and-burn" },
  { name: "Name Service", href: "https://naming.bonfida.org" },
];

export const Topbar = () => {
  const [showMenu, setShowMenu] = useState(false);
  const smallScreen = useSmallScreen("lg");

  let doc = undefined;
  if (typeof window !== "undefined") {
    doc = document;
  }

  const scroll = useScroll(doc);
  const sticky = !!scroll && scroll?.top > 5;

  if (smallScreen) {
    return (
      <div
        className={twMerge(
          sticky &&
            "fixed w-full z-navbar bg-[#13122B] bg-opacity-80 backdrop-blur-[30px]"
        )}
      >
        <div className="relative z-50 mx-5 py-[14px] flex items-center justify-between">
          <Link href="/">
            <div className="flex items-center cursor-pointer">
              <Logo alt="" className="h-[40px] w-[40px] mr-2" variant="white" />
              <span className="font-azeret text-[20px] font-bold text-white hidden md:block">
                Bonfida
              </span>
            </div>
          </Link>

          <Button
            onClick={() => setShowMenu(true)}
            className="rounded-[16px] border-[2px]"
            buttonType="secondary"
          >
            <Image
              loader={customLoader}
              layout="fixed"
              alt=""
              width={16}
              height={16}
              src="/images/burger/burger.svg"
            />
          </Button>
          {showMenu && (
            <div className="fixed top-0 left-0 w-screen h-screen bg-bds-dark-blues-DB900 px-5 py-[50px]">
              <div className="flex flex-col items-start w-full text-white text-[16px] font-azeret mx-[16px] font-medium my-10 ml-5">
                <Button
                  onClick={() => setShowMenu(false)}
                  className="rounded-[16px] border-[2px] absolute top-[14px] right-5"
                  buttonType="secondary"
                >
                  <Image
                    loader={customLoader}
                    layout="fixed"
                    alt=""
                    width={16}
                    height={16}
                    src="/images/close/close.webp"
                  />
                </Button>

                {sections.map((e) => {
                  return (
                    <>
                      <Link key={e.name} href={e.href}>
                        <button
                          onClick={() => setShowMenu(false)}
                          type="button"
                        >
                          {e.name}
                        </button>
                      </Link>
                      <div className="w-[80%] divider" />
                    </>
                  );
                })}

                <WalletConnect />
              </div>
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div
      className={twMerge(
        sticky &&
          "fixed bg-[#13122B] bg-opacity-80 backdrop-blur-[30px] w-full max-w-[1920px] z-navbar",
        "py-[14px]"
      )}
    >
      <div className="flex items-center justify-between mx-5 md:mx-[68px] relative z-50">
        <div className="flex items-center">
          <Link href="/">
            <div className="flex items-center cursor-pointer">
              <Logo alt="" className="h-[40px] w-[40px] mr-2" variant="white" />
              <span className="font-azeret text-[20px] font-bold text-white hidden md:block">
                Bonfida
              </span>
            </div>
          </Link>
          <div className="hidden items-center ml-10 space-x-10 font-medium text-white lg:flex font-azeret">
            {sections.map((e) => {
              return (
                <Link key={e.name} href={e.href}>
                  {e.name}
                </Link>
              );
            })}
          </div>
        </div>

        <div className="hidden md:block">
          <WalletConnect />
        </div>
      </div>
    </div>
  );
};
