import { useCurrentFrame, interpolate } from "remotion";

interface ScanLineProps {
  startFrame: number;
  duration?: number;
  width?: number;
  color?: string;
}

export const ScanLine: React.FC<ScanLineProps> = ({
  startFrame,
  duration = 20,
  width = 400,
  color = "#22c55e",
}) => {
  const frame = useCurrentFrame();
  const localFrame = frame - startFrame;

  if (localFrame < 0 || localFrame > duration) return null;

  const progress = interpolate(
    localFrame,
    [0, duration],
    [0, 1],
    { extrapolateRight: "clamp" }
  );

  const opacity = interpolate(
    localFrame,
    [0, 5, duration - 5, duration],
    [0, 1, 1, 0],
    { extrapolateRight: "clamp" }
  );

  return (
    <div
      style={{
        position: "absolute",
        left: `${progress * 100}%`,
        top: 0,
        bottom: 0,
        width: 3,
        background: `linear-gradient(180deg, transparent, ${color}, transparent)`,
        boxShadow: `0 0 20px ${color}, 0 0 40px ${color}`,
        opacity,
      }}
    />
  );
};
