import { interpolate, useCurrentFrame } from "remotion";

interface LogoProps {
  size?: number;
}

export const Logo: React.FC<LogoProps> = ({ size = 100 }) => {
  const frame = useCurrentFrame();

  const handRotation = interpolate(
    frame % 60,
    [0, 15, 30, 45, 60],
    [0, -5, 0, 5, 0]
  );

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: size * 0.2,
      }}
    >
      <div
        style={{
          fontSize: size * 0.8,
          transform: `rotate(${handRotation}deg)`,
          filter: "drop-shadow(0 0 20px rgba(239, 68, 68, 0.5))",
        }}
      >
        ✋
      </div>
      <div
        style={{
          fontSize: size * 0.6,
          fontWeight: 800,
          color: "#fff",
          letterSpacing: size * 0.05,
          textShadow: "0 0 30px rgba(255, 255, 255, 0.3)",
        }}
      >
        veto
      </div>
    </div>
  );
};
