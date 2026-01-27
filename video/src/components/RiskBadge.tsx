import { interpolate, useCurrentFrame } from "remotion";

interface RiskBadgeProps {
  level: "CRITICAL" | "HIGH" | "MEDIUM" | "LOW";
}

export const RiskBadge: React.FC<RiskBadgeProps> = ({ level }) => {
  const frame = useCurrentFrame();

  const colors = {
    CRITICAL: { bg: "#7f1d1d", text: "#fecaca", glow: "#ef4444" },
    HIGH: { bg: "#7c2d12", text: "#fed7aa", glow: "#f97316" },
    MEDIUM: { bg: "#713f12", text: "#fef08a", glow: "#eab308" },
    LOW: { bg: "#14532d", text: "#bbf7d0", glow: "#22c55e" },
  };

  const { bg, text, glow } = colors[level];

  const pulse = interpolate(
    frame % 30,
    [0, 15, 30],
    [1, 1.05, 1]
  );

  const glowIntensity = interpolate(
    frame % 30,
    [0, 15, 30],
    [10, 20, 10]
  );

  return (
    <div
      style={{
        background: bg,
        color: text,
        padding: "8px 16px",
        borderRadius: 6,
        fontWeight: 700,
        fontSize: 18,
        fontFamily: "monospace",
        transform: `scale(${pulse})`,
        boxShadow: `0 0 ${glowIntensity}px ${glow}`,
        border: `2px solid ${glow}`,
      }}
    >
      {level}
    </div>
  );
};
