import { CSSProperties } from "react";

export const ExternalLink = ({
  color = "#FFFFFF",
  className,
}: {
  color?: CSSProperties["color"];
  className?: string;
}) => {
  return (
    <svg
      width="16"
      height="17"
      viewBox="0 0 16 17"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <path
        d="M6.66675 4.49999H4.00008C3.64646 4.49999 3.30732 4.64047 3.05727 4.89051C2.80722 5.14056 2.66675 5.4797 2.66675 5.83332V12.5C2.66675 12.8536 2.80722 13.1927 3.05727 13.4428C3.30732 13.6928 3.64646 13.8333 4.00008 13.8333H10.6667C11.0204 13.8333 11.3595 13.6928 11.6096 13.4428C11.8596 13.1927 12.0001 12.8536 12.0001 12.5V9.83332M9.33341 3.16666H13.3334M13.3334 3.16666V7.16666M13.3334 3.16666L6.66675 9.83332"
        stroke={color}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
};
