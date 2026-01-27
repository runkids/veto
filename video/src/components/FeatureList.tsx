import { interpolate, spring, useVideoConfig } from "remotion";

interface FeatureListProps {
  frame: number;
}

const features = [
  { icon: "🔍", title: "Risk Evaluation", desc: "Auto-classify command danger levels" },
  { icon: "🔐", title: "Multi-Factor Auth", desc: "Touch ID, PIN, TOTP, Telegram" },
  { icon: "📋", title: "Audit Trail", desc: "Every command logged with context" },
  { icon: "⚙️", title: "Custom Rules", desc: "Define your own security policies" },
];

export const FeatureList: React.FC<FeatureListProps> = ({ frame }) => {
  const { fps } = useVideoConfig();

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(2, 1fr)",
        gap: 30,
        maxWidth: 900,
      }}
    >
      {features.map((feature, index) => {
        const delay = index * 12;
        const opacity = interpolate(
          frame - delay,
          [0, 15],
          [0, 1],
          { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
        );

        const translateY = interpolate(
          frame - delay,
          [0, 15],
          [30, 0],
          { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
        );

        return (
          <div
            key={index}
            style={{
              opacity,
              transform: `translateY(${translateY}px)`,
              background: "rgba(255, 255, 255, 0.05)",
              backdropFilter: "blur(10px)",
              borderRadius: 16,
              padding: 24,
              border: "1px solid rgba(255, 255, 255, 0.1)",
              display: "flex",
              alignItems: "flex-start",
              gap: 16,
            }}
          >
            <div
              style={{
                fontSize: 40,
                background: "linear-gradient(135deg, #1e3a5f, #1e293b)",
                padding: 12,
                borderRadius: 12,
              }}
            >
              {feature.icon}
            </div>
            <div>
              <div
                style={{
                  color: "#fff",
                  fontSize: 22,
                  fontWeight: 600,
                  marginBottom: 6,
                }}
              >
                {feature.title}
              </div>
              <div style={{ color: "#94a3b8", fontSize: 16 }}>
                {feature.desc}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
};
