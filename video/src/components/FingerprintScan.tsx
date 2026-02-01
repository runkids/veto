import { useCurrentFrame, interpolate, spring, useVideoConfig } from "remotion";

interface FingerprintScanProps {
  startFrame: number;
  result?: "authorized" | "denied";
}

export const FingerprintScan: React.FC<FingerprintScanProps> = ({ startFrame, result = "denied" }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  // Fingerprint icon fade in
  const iconOpacity = interpolate(localFrame, [0, 10], [0, 1], {
    extrapolateRight: "clamp",
  });

  // Scan line animation
  const scanY = interpolate(localFrame % 40, [0, 40], [-60, 60]);
  const scanOpacity = interpolate(localFrame, [5, 15], [0, 0.8], {
    extrapolateRight: "clamp",
  });

  // Progress ring
  const progressAngle = interpolate(localFrame, [10, 50], [0, 360], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Result state
  const showResult = localFrame > 55;
  const resultScale = spring({
    frame: localFrame - 55,
    fps,
    config: { damping: 10, stiffness: 200 },
  });

  const resultOpacity = interpolate(localFrame, [55, 65], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Glow pulse
  const resultColor = result === "authorized" ? "#22c55e" : "#ef4444";
  const glowIntensity = showResult
    ? interpolate(localFrame % 30, [0, 15, 30], [30, 50, 30])
    : interpolate(localFrame % 20, [0, 10, 20], [10, 20, 10]);

  const ringColor = showResult ? resultColor : "#3b82f6";

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 20,
      }}
    >
      {/* Fingerprint container */}
      <div
        style={{
          position: "relative",
          width: 140,
          height: 140,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
        }}
      >
        {/* Progress ring */}
        <svg
          width="140"
          height="140"
          style={{
            position: "absolute",
            transform: "rotate(-90deg)",
          }}
        >
          <circle
            cx="70"
            cy="70"
            r="65"
            fill="none"
            stroke="rgba(255, 255, 255, 0.1)"
            strokeWidth="4"
          />
          <circle
            cx="70"
            cy="70"
            r="65"
            fill="none"
            stroke={ringColor}
            strokeWidth="4"
            strokeDasharray={`${(progressAngle / 360) * 408} 408`}
            style={{
              filter: `drop-shadow(0 0 ${glowIntensity}px ${ringColor})`,
            }}
          />
        </svg>

        {/* Fingerprint icon */}
        <div
          style={{
            opacity: iconOpacity,
            fontSize: 72,
            filter: `drop-shadow(0 0 ${glowIntensity}px ${ringColor})`,
            transform: showResult ? `scale(${resultScale})` : "scale(1)",
          }}
        >
          {showResult ? (result === "authorized" ? "✓" : "✗") : "👆"}
        </div>

        {/* Scan line */}
        {!showResult && (
          <div
            style={{
              position: "absolute",
              width: 100,
              height: 3,
              background: `linear-gradient(90deg, transparent, ${ringColor}, transparent)`,
              transform: `translateY(${scanY}px)`,
              opacity: scanOpacity,
              borderRadius: 2,
            }}
          />
        )}
      </div>

      {/* Status text */}
      <div
        style={{
          fontSize: 18,
          fontWeight: 600,
          color: showResult ? resultColor : "#94a3b8",
          letterSpacing: 2,
          opacity: showResult ? resultOpacity : iconOpacity,
        }}
      >
        {showResult ? (result === "authorized" ? "AUTHORIZED" : "DENIED") : "Touch ID"}
      </div>
    </div>
  );
};
