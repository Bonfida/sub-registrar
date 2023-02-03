import { ReactNode } from "react";
import { Footer } from "./Footer";
import { Topbar } from "./Topbar";
import { useLocalStorageState } from "ahooks";

export const Layout = ({ children }: { children: ReactNode }) => {
  const [visible, setVisible] = useLocalStorageState<boolean>("warning", {
    defaultValue: true,
  });

  return (
    <div className="pb-10 overflow-x-hidden max-w-[1920px] mx-auto w-screen bg-bds-dark-blues-DB900">
      <Topbar />
      {children}
      <Footer />
    </div>
  );
};
