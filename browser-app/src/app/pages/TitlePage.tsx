import { useNavigate } from "react-router-dom";
import { css } from "../../../styled-system/css";
import { Button } from "../../components/ui/Button";

export const TitlePage = () => {
  const navigate = useNavigate();
  return (
    <div
      className={css({
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        height: "100vh",
        backgroundColor: "#004400",
        color: "white",
        fontFamily: "sans-serif",
      })}
    >
      <h1 className={css({ fontSize: "3rem", marginBottom: "2rem", fontWeight: "bold" })}>
        TS Mahjong
      </h1>
      <Button
        onClick={() => navigate("/game")}
        className={css({
          fontSize: "1.5rem",
          padding: "1rem 2rem",
          backgroundColor: "green.600",
          _hover: { backgroundColor: "green.700" },
        })}
      >
        Start Game
      </Button>
    </div>
  );
};
