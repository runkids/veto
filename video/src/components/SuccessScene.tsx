import { useCurrentFrame, interpolate, spring, useVideoConfig } from "remotion";

interface SuccessSceneProps {
  startFrame: number;
}

export const SuccessScene: React.FC<SuccessSceneProps> = ({ startFrame }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  // Checkmark animation
  const checkScale = spring({
    frame: localFrame - 5,
    fps,
    config: { damping: 8, stiffness: 150 },
  });

  const checkOpacity = interpolate(localFrame, [5, 15], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Terminal output animation
  const outputLines = [
    { text: "$ git push origin main --force", color: "#94a3b8", delay: 15 },
    { text: "Enumerating objects: 42, done.", color: "#22c55e", delay: 25 },
    { text: "Counting objects: 100% (42/42)", color: "#22c55e", delay: 30 },
    { text: "Writing objects: 100% (24/24)", color: "#22c55e", delay: 35 },
    { text: "remote: Resolving deltas: 100%", color: "#22c55e", delay: 40 },
    { text: "To github.com:user/project.git", color: "#64748b", delay: 45 },
    { text: "   abc1234..def5678  main -> main", color: "#3b82f6", delay: 50 },
  ];

  // Ripple effect
  const rippleOpacity = interpolate(localFrame, [0, 10, 30], [0.5, 0.3, 0], {
    extrapolateRight: "clamp",
  });
  const rippleScale = interpolate(localFrame, [0, 30], [1, 3], {
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        background: "linear-gradient(135deg, #0a0f0a 0%, #0f1a0f 50%, #0a0f0a 100%)",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
        gap: 30,
      }}
    >
      {/* Success ripple */}
      <div
        style={{
          position: "absolute",
          width: 200,
          height: 200,
          borderRadius: "50%",
          border: "3px solid #22c55e",
          opacity: rippleOpacity,
          transform: `scale(${rippleScale})`,
        }}
      />

      {/* Checkmark badge */}
      <div
        style={{
          opacity: checkOpacity,
          transform: `scale(${checkScale})`,
          background: "linear-gradient(135deg, #166534, #15803d)",
          borderRadius: "50%",
          width: 100,
          height: 100,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          boxShadow: "0 0 40px rgba(34, 197, 94, 0.5), 0 0 80px rgba(34, 197, 94, 0.2)",
          border: "3px solid #22c55e",
        }}
      >
        <span style={{ fontSize: 50, color: "#fff" }}>✓</span>
      </div>

      {/* Terminal output */}
      <div
        style={{
          background: "rgba(15, 23, 42, 0.9)",
          borderRadius: 12,
          padding: "20px 28px",
          border: "1px solid rgba(34, 197, 94, 0.3)",
          minWidth: 500,
          fontFamily: "'SF Mono', monospace",
          fontSize: 14,
        }}
      >
        {outputLines.map((line, index) => {
          const lineOpacity = interpolate(
            localFrame,
            [line.delay, line.delay + 5],
            [0, 1],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
          );

          return (
            <div
              key={index}
              style={{
                color: line.color,
                opacity: lineOpacity,
                marginBottom: 6,
              }}
            >
              {line.text}
            </div>
          );
        })}
      </div>
    </div>
  );
};
