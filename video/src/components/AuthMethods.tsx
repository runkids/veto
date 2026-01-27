import { useCurrentFrame, interpolate, spring, useVideoConfig } from "remotion";

interface AuthMethodsProps {
  startFrame: number;
  size?: "normal" | "large";
}

const methods = [
  { icon: "👆", label: "Touch ID", color: "#3b82f6" },
  { icon: "🔢", label: "PIN", color: "#8b5cf6" },
  { icon: "🔐", label: "TOTP", color: "#f59e0b" },
  { icon: "📱", label: "Telegram", color: "#0ea5e9" },
];

export const AuthMethods: React.FC<AuthMethodsProps> = ({ startFrame, size = "large" }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  const isLarge = size === "large";
  const iconSize = isLarge ? 72 : 48;
  const labelSize = isLarge ? 18 : 14;
  const padding = isLarge ? 24 : 16;
  const gap = isLarge ? 32 : 20;
  const borderRadius = isLarge ? 20 : 16;

  return (
    <div
      style={{
        display: "flex",
        gap,
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      {methods.map((method, index) => {
        const delay = index * 10;
        const itemFrame = localFrame - delay;

        const scale = spring({
          frame: Math.max(0, itemFrame),
          fps,
          config: { damping: 12, stiffness: 200 },
        });

        const opacity = interpolate(
          itemFrame,
          [-5, 8],
          [0, 1],
          { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
        );

        // Highlight animation - cycle through methods (slower)
        const cycleFrame = localFrame - 50; // Start cycling after all appear
        const cycleIndex = Math.floor(cycleFrame / 20) % methods.length;
        const isHighlighted = cycleFrame > 0 && cycleIndex === index;

        const highlightScale = isHighlighted ? 1.15 : 1;
        const glowIntensity = isHighlighted ? 30 : 0;

        return (
          <div
            key={index}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 12,
              opacity,
              transform: `scale(${scale * highlightScale})`,
              transition: "transform 0.2s ease-out",
            }}
          >
            <div
              style={{
                fontSize: iconSize,
                background: `linear-gradient(135deg, ${method.color}33, ${method.color}11)`,
                padding,
                borderRadius,
                border: `3px solid ${method.color}${isHighlighted ? "ff" : "44"}`,
                boxShadow: isHighlighted
                  ? `0 0 ${glowIntensity}px ${method.color}, 0 0 ${glowIntensity * 2}px ${method.color}44`
                  : "none",
              }}
            >
              {method.icon}
            </div>
            <div
              style={{
                fontSize: labelSize,
                color: isHighlighted ? method.color : "#94a3b8",
                fontWeight: isHighlighted ? 700 : 500,
                fontFamily: "system-ui",
                letterSpacing: 0.5,
              }}
            >
              {method.label}
            </div>
          </div>
        );
      })}
    </div>
  );
};
