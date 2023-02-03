import { useContext } from "react";
import { ToastContext } from "@bonfida/components";

export const useToastContext = () => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("Missing toast context");
  }
  return context;
};
