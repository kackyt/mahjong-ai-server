import type { ReactNode } from "react";
import { css } from "../../../../styled-system/css";

interface GameLayoutProps {
  header?: ReactNode;
  main: ReactNode;
  footer?: ReactNode;
  overlay?: ReactNode; // For modal results
}

export const GameLayout = ({ header, main, footer, overlay }: GameLayoutProps) => {
  return (
    <div
      className={css({
        backgroundColor: "#006400",
        width: "100vw",
        height: "100vh",
        color: "white",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
        fontFamily: "sans-serif",
        position: "relative",
      })}
    >
      {header && (
        <div className={css({ flex: "0 0 auto", padding: "10px", background: "rgba(0,0,0,0.2)" })}>
          {header}
        </div>
      )}

      <div
        className={css({
          flex: "1 1 auto",
          position: "relative",
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
        })}
      >
        {main}
      </div>

      {footer && (
        <div className={css({ flex: "0 0 auto", padding: "10px", background: "rgba(0,0,0,0.2)" })}>
          {footer}
        </div>
      )}

      {overlay && (
        <div
          className={css({
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            zIndex: 100,
            pointerEvents: "auto", // ensure overlay captures clicks
          })}
        >
          {overlay}
        </div>
      )}
    </div>
  );
};
