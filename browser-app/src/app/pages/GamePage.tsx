import { useEffect, useState } from "react";
import { css } from "../../../styled-system/css";
import { GameLayout } from "../../components/layouts/GameLayout";
import { Button } from "../../components/ui/Button";
import { Hand } from "../../components/ui/Hand";
import { River } from "../../components/ui/River";
import { Tile } from "../../components/ui/Tile";
import { useGame } from "../../features/game/hooks/useGame";
import { useAi } from "../../features/mahjong/hooks/useAi";

export const GamePage = () => {
  const {
    gameState,
    initGame,
    dahai,
    tsumo,
    riichi,
    agari,
    riichiMode,
    tryRiichiDiscard,
    nextHand,
  } = useGame();
  const player = gameState.players[0];

  const { isReady, getDiscard } = useAi();
  const [isAutoPlay, setIsAutoPlay] = useState(false);

  useEffect(() => {
    initGame();
  }, [initGame]);

  // Auto Play Effect
  useEffect(() => {
    if (!isAutoPlay || !isReady || gameState.isGameOver || !player) return;

    if (player.isTsumo && !player.isRiichi) {
      // Add a small delay for visual effect
      const timer = setTimeout(async () => {
        try {
          // Convert regular array to Uint8Array expected by Wasm
          const discardIndex = await getDiscard(player.tehai.map(p => p.paiNum));
          if (discardIndex !== null && discardIndex !== undefined) {
            dahai(discardIndex);
          }
        } catch (e) {
          console.error("AI Error:", e);
        }
      }, 500);
      return () => clearTimeout(timer);
    }

    if (player.isTsumo && player.isRiichi) {
      // Auto discard when riichi (tsumogiri)
      const timer = setTimeout(() => {
        dahai(player.tehai.length); // Tsumogiri
      }, 500);
      return () => clearTimeout(timer);
    }
  }, [isAutoPlay, isReady, gameState.isGameOver, player, getDiscard, dahai]);

  if (!player) return <div>Loading...</div>;

  // Handlers
  const handleTehaiClick = (index: number) => {
    if (gameState.isGameOver || isAutoPlay) return;

    if (riichiMode) {
      tryRiichiDiscard(index);
      return;
    }

    if (player.isRiichi) return;

    if (player.isTsumo) {
      dahai(index);
    }
  };

  const handleTsumoClick = () => {
    if (gameState.isGameOver || isAutoPlay) return;

    if (riichiMode) {
      tryRiichiDiscard(player.tehai.length);
      return;
    }

    if (player.isTsumo) {
      dahai(player.tehai.length);
    }
  };

  // --- Sections ---

  // Header
  const Header = (
    <div
      className={css({
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        width: "100%",
      })}
    >
      <div>
        <div className={css({ display: "flex", alignItems: "center", gap: "10px" })}>
          <h2 className={css({ margin: 0, fontSize: "1.2rem", fontWeight: "bold" })}>
            OpenMahjong TS
          </h2>
          <Button
            onClick={() => {
              if (confirm("Restart Game? Score will be reset.")) initGame();
            }}
            className={css({ fontSize: "0.7rem", padding: "2px 5px" })}
          >
            Reset
          </Button>
          <Button
            onClick={() => setIsAutoPlay(!isAutoPlay)}
            className={css({
              fontSize: "0.7rem",
              padding: "2px 5px",
              backgroundColor: isAutoPlay ? "green.600" : "gray.500"
            })}
            disabled={!isReady}
          >
            {isAutoPlay ? "Auto Check: ON" : "Auto Check: OFF"} {isReady ? "" : "(Loading AI...)"}
          </Button>
        </div>
        <div className={css({ fontSize: "0.9rem", marginTop: "5px" })}>
          <span>
            {["東", "南", "西", "北"][gameState.bakaze] ?? "?"} {gameState.kyoku}局
          </span>
          <span className={css({ marginLeft: "10px" })}>{gameState.honba}本場</span>
          <span className={css({ marginLeft: "10px" })}>
            供託: {gameState.kyoutaku}
          </span>
          <span className={css({ marginLeft: "10px" })}>
            Score: {player.score} {gameState.oya === 0 ? "(親)" : ""}
          </span>
          {player.isRiichi && (
            <span className={css({ color: "red.300", fontWeight: "bold", marginLeft: "10px" })}>
              RIICHI
            </span>
          )}
        </div>
      </div>
      <div>
        <div
          className={css({
            display: "flex",
            flexDirection: "column",
            alignItems: "flex-end",
            gap: "5px",
          })}
        >
          <div className={css({ display: "flex", alignItems: "center", gap: "5px" })}>
            <span className={css({ fontSize: "0.8rem", marginRight: "5px" })}>DORA</span>
            {gameState.dora.map((p, i) => (
              <Tile key={`dora-${p.id ?? i}`} pai={p} />
            ))}
          </div>
          {/* Ura Dora */}
          {gameState.isGameOver && player.isRiichi && player.shanten === -1 && (
            <div className={css({ display: "flex", alignItems: "center", gap: "5px" })}>
              <span className={css({ fontSize: "0.8rem", marginRight: "5px", color: "#aaf" })}>
                URA
              </span>
              {gameState.uraDora.slice(0, gameState.dora.length).map((p, i) => (
                <Tile key={`ura-${p.id ?? i}`} pai={p} />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );

  // Main (River)
  const Main = <River tiles={player.kawahai} />;

  // Footer (Controls + Hand)
  const Footer = (
    <div
      className={css({
        display: "flex",
        flexDirection: "column",
        gap: "10px",
        alignItems: "center",
      })}
    >
      {/* Controls */}
      <div className={css({ display: "flex", gap: "10px", justifyContent: "center" })}>
        <Button onClick={tsumo} disabled={player.isTsumo || gameState.isGameOver}>
          Force Tsumo
        </Button>
        <Button
          onClick={riichi}
          disabled={
            !riichiMode &&
            (player.isRiichi ||
              !player.isTsumo ||
              gameState.isGameOver ||
              player.shanten > 0 ||
              player.score < 1000)
          }
          className={css({
            backgroundColor: riichiMode
              ? "red.600"
              : player.shanten <= 0 && !player.isRiichi
                ? "orange.500"
                : undefined,
          })}
        >
          {riichiMode ? "Cancel Riichi" : "Riichi"}
        </Button>
        <Button
          onClick={agari}
          disabled={!player.isTsumo || gameState.isGameOver || player.shanten !== -1}
          className={css({ backgroundColor: player.shanten === -1 ? "red.600" : undefined })}
        >
          Tsumo
        </Button>
      </div>

      {/* Hand */}
      <Hand
        tehai={player.tehai}
        tsumo={player.tsumohai}
        onClickTile={handleTehaiClick}
        onClickTsumo={handleTsumoClick}
        disableInteraction={(!player.isTsumo && !riichiMode) || gameState.isGameOver}
      />
    </div>
  );

  // Overlay
  const Overlay = gameState.isGameOver ? (
    <div
      className={css({
        backgroundColor: "rgba(0,0,0,0.8)",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
        width: "100%",
        height: "100%",
        color: "white",
      })}
    >
      <h2 className={css({ fontSize: "2rem", marginBottom: "20px" })}>Game Over</h2>
      <pre
        className={css({
          textAlign: "left",
          whiteSpace: "pre-wrap",
          marginBottom: "20px",
          backgroundColor: "rgba(255,255,255,0.1)",
          padding: "20px",
          borderRadius: "8px",
        })}
      >
        {gameState.resultMessage}
      </pre>
      <div className={css({ display: "flex", gap: "20px" })}>
        <Button
          onClick={nextHand}
          className={css({
            fontSize: "1.2rem",
            padding: "10px 20px",
            backgroundColor: "green.600",
            _hover: { backgroundColor: "green.700" },
          })}
        >
          Next Hand
        </Button>
        <Button
          onClick={initGame}
          className={css({ fontSize: "1.0rem", padding: "10px 20px", backgroundColor: "gray.600" })}
        >
          New Game (Reset)
        </Button>
      </div>
    </div>
  ) : null;

  return <GameLayout header={Header} main={Main} footer={Footer} overlay={Overlay} />;
};
