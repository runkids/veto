import { useCurrentFrame, interpolate } from "remotion";

interface ProgressBarProps {
  startFrame: number;
  duration: number;
  maxProgress?: number;
  width?: number;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  startFrame,
  duration,
  maxProgress = 0.85,
  width = 400,
}) => {
  const frame = useCurrentFrame();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  const progress = interpolate(
    localFrame,
    [0, duration],
    [0, maxProgress],
    { extrapolateRight: "clamp" }
  );

  // Color transition from yellow to red as progress increases
  const r = Math.round(interpolate(progress, [0, 0.5, 1], [234, 239, 239]));
  const g = Math.round(interpolate(progress, [0, 0.5, 1], [179, 68, 68]));
  const b = Math.round(interpolate(progress, [0, 0.5, 1], [8, 68, 68]));
  const barColor = `rgb(${r}, ${g}, ${b})`;

  const pulseIntensity = interpolate(
    localFrame % 15,
    [0, 7, 15],
    [0, 10, 0]
  );

  return (
    <div style={{ width }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          marginBottom: 8,
          fontFamily: "'SF Mono', monospace",
          fontSize: 14,
        }}
      >
        <span style={{ color: "#94a3b8" }}>Executing...</span>
        <span style={{ color: barColor }}>{Math.round(progress * 100)}%</span>
      </div>
      <div
        style={{
          height: 8,
          background: "rgba(255, 255, 255, 0.1)",
          borderRadius: 4,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            height: "100%",
            width: `${progress * 100}%`,
            background: `linear-gradient(90deg, ${barColor}, ${barColor}dd)`,
            borderRadius: 4,
            boxShadow: `0 0 ${pulseIntensity}px ${barColor}`,
            transition: "width 0.1s linear",
          }}
        />
      </div>
    </div>
  );
};
