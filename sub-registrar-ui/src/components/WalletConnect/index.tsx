import { useWalletModal } from "@solana/wallet-adapter-react-ui";
import { useWallet } from "@solana/wallet-adapter-react";
import { abbreviate } from "../../utils/pubkey";
import { customLoader } from "../../utils/custom-loader";
import { useProfilePic } from "@/hooks/useProfilePic";
import { twMerge } from "tailwind-merge";
import { useAsyncEffect, useSafeState, usePrevious } from "ahooks";
import axios from "axios";
import { Popover } from "@headlessui/react";
import { usePopper } from "react-popper";
import { useState, useEffect } from "react";
import { useFavouriteDomain } from "@/hooks/useFavoriteDomain";
import { useModalContext } from "@/hooks/useModalContext";
import Image from "next/image";

const robotPic = "/images/robot/robot.svg";

export const WalletConnect = ({ width }: { width?: string }) => {
  const { setVisible, visible } = useWalletModal();
  const { setVisible: setVisibleContext, visible: visibleContext } =
    useModalContext();
  const { connected, publicKey, connecting, disconnect } = useWallet();
  const { data: pic } = useProfilePic(publicKey?.toBase58());
  const { data: fav } = useFavouriteDomain(publicKey?.toBase58());
  const [validCustomPic, setValicCustomPic] = useSafeState(false);

  let [referenceElement, setReferenceElement] = useState<HTMLButtonElement>();
  let [popperElement, setPopperElement] = useState<HTMLDivElement>();
  let { styles, attributes } = usePopper(referenceElement, popperElement, {
    placement: "top-start",
  });

  const previous = usePrevious(visible);

  useEffect(() => {
    if (previous && visibleContext) {
      setVisibleContext(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [visible]);

  useAsyncEffect(async () => {
    // Check custom pic validity
    try {
      if (!pic) {
        setValicCustomPic(false);
        return;
      }
      await axios.get(pic);
      setValicCustomPic(true);
    } catch {}
  }, [connected, pic]);

  if (connected && publicKey) {
    return (
      <Popover>
        {/* Button */}
        <Popover.Button
          // @ts-ignore
          ref={setReferenceElement}
        >
          <div
            className={twMerge(
              "bg-gradient-to-r from-[#00F0FF] to-[#CBFF5E] p-[1.5px] rounded-[16px] h-[52px]",
              width ? width : "w-[182px]"
            )}
          >
            <div className="bg-[#13122b] h-full w-full rounded-[14px] px-4 flex items-center justify-center lg:justify-start space-x-2">
              <Image
                className="rounded-full"
                loader={customLoader}
                src={validCustomPic ? (pic as string) : robotPic}
                alt=""
                width={24}
                height={24}
                layout="fixed"
              />

              <p className="font-bold font-azeret text-[16px] w-fit text-white">
                {fav ? fav + ".sol" : abbreviate(publicKey)}
              </p>
            </div>
          </div>
        </Popover.Button>

        {/* Dropdown */}
        <Popover.Panel
          // @ts-ignore
          ref={setPopperElement}
          style={styles.popper}
          {...attributes.popper}
          className="absolute bg-[#13122B] rounded-[24px] border-[1px] border-[#2A2A51] w-full md:w-[200px] p-[8px] md:mt-2 mb-2 md:mb-0"
        >
          <div className="flex flex-col pl-[20px] space-y-2 text-white font-azeret font-medium text-[16px] my-2">
            <button
              onClick={() => setVisible(true)}
              type="button"
              className="w-fit"
            >
              Change wallet
            </button>
            <div className="w-[95%] my-0 divider" />
            <button onClick={disconnect} type="button" className="w-fit">
              Disconnect
            </button>
          </div>
        </Popover.Panel>
      </Popover>
    );
  }

  return (
    <div
      className={twMerge(
        "bg-gradient-to-r from-[#00F0FF] to-[#CBFF5E] p-[2px] rounded-[16px] h-[52px]",
        width ? width : "w-[182px]",
        "relative z-30"
      )}
    >
      <button
        className="bg-[#13122b] h-full w-full rounded-[14px] px-4 flex items-center justify-center space-x-2"
        onClick={() => {
          setVisibleContext(true);
          setVisible(true);
        }}
        type="button"
      >
        <span className="font-bold font-azeret text-[16px] w-fit text-white normal-case">
          {connecting ? "Connecting..." : "Connect wallet"}
        </span>
      </button>
    </div>
  );
};
