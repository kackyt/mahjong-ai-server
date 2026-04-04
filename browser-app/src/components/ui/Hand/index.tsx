import { css, cx } from "../../../../styled-system/css";
import type { Pai } from "../../../features/mahjong/types";
import { Tile } from "../Tile";

interface HandProps {
  tehai: Pai[];
  tsumo?: Pai | null;
  onClickTile?: (index: number) => void;
  onClickTsumo?: () => void;
  className?: string;
  disableInteraction?: boolean;
}

export const Hand = ({
  tehai,
  tsumo,
  onClickTile,
  onClickTsumo,
  className,
  disableInteraction = false,
}: HandProps) => {
  return (
    <div
      className={cx(
        css({
          display: "flex",
          alignItems: "flex-end",
          justifyContent: "center",
          gap: "2px",
          paddingBottom: "10px",
        }),
        className,
      )}
    >
      <div className={css({ display: "flex", gap: "2px" })}>
        {tehai.map((p, i) => (
          <Tile
            key={`${p.paiNum}-${i}`}
            pai={p}
            onClick={!disableInteraction && onClickTile ? () => onClickTile(i) : undefined}
          />
        ))}
      </div>

      <div className={css({ marginLeft: "15px", minWidth: "30px" })}>
        {tsumo && (
          <Tile
            pai={tsumo}
            onClick={!disableInteraction && onClickTsumo ? onClickTsumo : undefined}
          />
        )}
      </div>
    </div>
  );
};
