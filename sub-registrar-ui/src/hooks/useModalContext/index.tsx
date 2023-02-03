import { ModalContext } from "../../context/modal";
import { useContext } from "react";

export const useModalContext = () => {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error("Modal context missing");
  }
  return context;
};
