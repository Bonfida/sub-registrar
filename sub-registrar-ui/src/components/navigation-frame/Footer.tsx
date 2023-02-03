import { Logo } from "@bonfida/components";
import { Link } from "../Link";
import { Urls } from "../../utils/urls";
import { customLoader } from "../../utils/custom-loader";
import Image from "next/image";

const sections = [
  { name: "Trade", href: "/trade" },
  { name: "Stake", href: "/stake" },
  { name: "Buy & Burn", href: "/buy-and-burn" },
  { name: "Name Service", href: "https://naming.bonfida.org" },
  { name: "Github", href: "https://github.com/Bonfida" },
  { name: "Mastodon", href: Urls.mastodon },
];

export const Footer = () => {
  return (
    <div className="relative mt-10">
      {/* Desktop */}
      <div className="lg:flex items-center justify-around w-full h-[106px] hidden relative z-0">
        {/* Logo */}
        <div className="flex items-center space-x-4">
          <Logo variant="white" className="w-[30px] h-[35px]" alt="" />
          <span className="text-xl font-bold text-white font-azeret">
            Bonfida
          </span>
        </div>
        {/*  */}

        <div className="flex items-center space-x-10 text-sm font-medium text-white font-azeret">
          {sections.map((e) => {
            return (
              <Link key={e.name} href={e.href}>
                {e.name}
              </Link>
            );
          })}
        </div>

        {/*  */}

        <div className="flex items-center space-x-10 text-sm font-medium text-white font-azeret">
          <Link href={Urls.gitbook}>Help</Link>
          <div className="cursor-pointer">
            <Link href={Urls.telegram}>
              <Image
                alt="Telegram"
                loader={customLoader}
                width={20}
                height={20}
                src="/images/telegram/telegram.webp"
              />
            </Link>
          </div>
          <div className="cursor-pointer">
            <Link href={Urls.twitter}>
              <Image
                alt="Twitter"
                loader={customLoader}
                width={20}
                height={16.65}
                src="/images/twitter/twitter.webp"
              />
            </Link>
          </div>
          <div className="cursor-pointer">
            <Link href={Urls.mastodon}>
              <Image
                alt="Mastodon"
                loader={customLoader}
                width={25}
                height={20}
                src="/images/mastodon/mastodon.png"
              />
            </Link>
          </div>
        </div>
      </div>

      {/* Mobile */}
      <div className="block lg:hidden px-[64px] mb-20">
        {/* Logo */}
        <div className="flex items-center space-x-4 mb-[29px]">
          <Logo variant="white" className="w-[30px] h-[35px]" alt="" />
          <span className="text-xl font-bold text-white font-azeret">
            Bonfida
          </span>
        </div>

        <div className="grid grid-cols-2 gap-x-[72px] gap-y-[24px] text-sm font-medium text-white font-azeret">
          {sections.map((e) => {
            return (
              <Link key={e.name} href={e.href}>
                {e.name}
              </Link>
            );
          })}
          <div className="flex items-center space-x-2">
            <Link href={Urls.telegram}>
              <Image
                alt="Telegram"
                loader={customLoader}
                width={20}
                height={20}
                src="/images/telegram/telegram.webp"
              />
            </Link>
            <Link href={Urls.twitter}>
              <Image
                alt="Twitter"
                loader={customLoader}
                width={20}
                height={16.65}
                src="/images/twitter/twitter.webp"
              />
            </Link>
            <Link href={Urls.mastodon}>
              <Image
                alt="Mastodon"
                loader={customLoader}
                width={25}
                height={20}
                src="/images/mastodon/mastodon.png"
              />
            </Link>
          </div>
        </div>

        <div className="text-[12px] font-azeret text-white text-opacity-90 mt-[24px]">
          This web site is hosted on IPFS and is not available in the United
          States or other prohibited jurisdictions
        </div>
      </div>
    </div>
  );
};
