import { Button as BaseButton, type ButtonProps } from "@mui/base/Button";
import { css, cx } from "../../../../styled-system/css";

export const Button = (props: ButtonProps) => {
  return (
    <BaseButton
      {...props}
      className={cx(
        css({
          padding: "8px 16px",
          fontSize: "0.9rem",
          fontWeight: "bold",
          cursor: "pointer",
          backgroundColor: "#444",
          color: "white",
          border: "1px solid #666",
          borderRadius: "4px",
          transition: "all 0.2s",
          fontFamily: "inherit",
          _hover: { backgroundColor: "#666" },
          _active: { transform: "translateY(1px)" },
          _disabled: {
            backgroundColor: "#ccc",
            color: "#888",
            cursor: "not-allowed",
            border: "1px solid #ccc",
          },
          // Add variants logic here if needed
        }),
        props.className,
      )}
    />
  );
};
