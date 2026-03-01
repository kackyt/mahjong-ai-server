import { css, cx } from "../../../../styled-system/css";
import type { Pai } from "../../../features/mahjong/types";

interface TileProps {
  pai: Pai;
  onClick?: () => void;
  className?: string;
}

// Convert PaiNum to logical name for image file
export const getTileImageName = (paiNum: number): string => {
  if (paiNum < 9) return `man${paiNum + 1}.gif`;
  if (paiNum < 18) return `pin${paiNum - 8}.gif`;
  if (paiNum < 27) return `sou${paiNum - 17}.gif`;

  const zihai = ["ton", "nan", "sha", "pei", "haku", "hatu", "tyun"];
  if (paiNum < 34) {
    return `${zihai[paiNum - 27]}.gif`;
  }

  return "ura.gif";
};

export const Tile = ({ pai, onClick, className }: TileProps) => {
  const imgName = getTileImageName(pai.paiNum);
  const src = `/images/haiga/${imgName}`;

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: Tile needs to be clickable div for game UI
    <div
      className={cx(
        css({
          cursor: onClick ? "pointer" : "default",
          display: "inline-block",
          width: "clamp(30px, 4vw, 44px)",
          lineHeight: 0,
          userSelect: "none",
          transition: "transform 0.1s",
          _active: onClick ? { transform: "translateY(1px)" } : {},
        }),
        className,
      )}
      onClick={onClick}
      onKeyDown={
        onClick
          ? (e) => {
              if (e.key === "Enter" || e.key === " ") onClick();
            }
          : undefined
      }
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
    >
      <img
        src={src}
        alt={`${pai.paiNum}`}
        className={css({
          width: "100%",
          height: "auto",
          display: "block",
          boxShadow: "1px 1px 3px rgba(0,0,0,0.3)",
          borderRadius: "2px",
        })}
      />
    </div>
  );
};
