import { interpolate, useCurrentFrame, spring, useVideoConfig } from "remotion";

interface AuthPromptProps {
  type: "touchid" | "pin" | "totp" | "telegram";
}

export const AuthPrompt: React.FC<AuthPromptProps> = ({ type }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const icons = {
    touchid: "👆",
    pin: "🔢",
    totp: "🔐",
    telegram: "📱",
  };

  const labels = {
    touchid: "Touch ID",
    pin: "PIN Code",
    totp: "TOTP",
    telegram: "Telegram",
  };

  const scale = spring({
    frame,
    fps,
    config: { damping: 12, stiffness: 150 },
  });

  const pulse = interpolate(
    frame % 40,
    [0, 20, 40],
    [1, 1.08, 1]
  );

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        background: "linear-gradient(135deg, #1e3a5f, #1e293b)",
        padding: "12px 20px",
        borderRadius: 10,
        transform: `scale(${scale * pulse})`,
        boxShadow: "0 10px 25px rgba(0, 0, 0, 0.3)",
        border: "1px solid rgba(59, 130, 246, 0.3)",
      }}
    >
      <span style={{ fontSize: 28 }}>{icons[type]}</span>
      <span
        style={{
          color: "#fff",
          fontSize: 18,
          fontWeight: 600,
        }}
      >
        {labels[type]}
      </span>
    </div>
  );
};
