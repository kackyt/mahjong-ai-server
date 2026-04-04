import { css, cx } from "../../../../styled-system/css";
import type { Pai } from "../../../features/mahjong/types";
import { Tile } from "../Tile";

interface RiverProps {
  tiles: Pai[];
  className?: string; // allow override
}

export const River = ({ tiles, className }: RiverProps) => {
  return (
    <div
      className={cx(
        css({
          display: "grid",
          gridTemplateColumns: "repeat(6, auto)",
          gap: "4px",
          alignItems: "center",
          width: "fit-content",
        }),
        className,
      )}
    >
      {tiles.map((p, i) => (
        <div key={`${p.paiNum}-${i}`} className={css({ opacity: p.isTsumogiri ? 0.6 : 1.0 })}>
          <Tile
            pai={p}
            className={p.isRiichi ? css({ transform: "rotate(90deg)", mx: "5px" }) : ""}
          />
        </div>
      ))}
    </div>
  );
};
