// Context to know whether a modal is currently displayed

import { Context, createContext, ReactNode, useState, Dispatch } from "react";

interface ModalValue {
  visible: boolean;
  setVisible: Dispatch<React.SetStateAction<boolean>>;
}

export const ModalContext: Context<null | ModalValue> =
  createContext<null | ModalValue>(null);

export const ModalContextProvider = ({ children }: { children: ReactNode }) => {
  const [visible, setVisible] = useState(false);
  return (
    <ModalContext.Provider value={{ visible, setVisible }}>
      {children}
    </ModalContext.Provider>
  );
};
