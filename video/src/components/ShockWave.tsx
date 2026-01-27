import { useCurrentFrame, interpolate, spring, useVideoConfig } from "remotion";

interface ShockWaveProps {
  startFrame: number;
  color?: string;
  maxRadius?: number;
}

export const ShockWave: React.FC<ShockWaveProps> = ({
  startFrame,
  color = "rgba(34, 197, 94, 0.6)",
  maxRadius = 800,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  const rings = [0, 5, 10]; // Staggered rings

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        left: "50%",
        transform: "translate(-50%, -50%)",
        pointerEvents: "none",
      }}
    >
      {rings.map((delay, index) => {
        const ringFrame = localFrame - delay;
        if (ringFrame < 0) return null;

        const scale = interpolate(
          ringFrame,
          [0, 30],
          [0, 1],
          { extrapolateRight: "clamp" }
        );

        const opacity = interpolate(
          ringFrame,
          [0, 10, 30],
          [0, 0.8, 0],
          { extrapolateRight: "clamp" }
        );

        const radius = maxRadius * scale;

        return (
          <div
            key={index}
            style={{
              position: "absolute",
              top: "50%",
              left: "50%",
              width: radius * 2,
              height: radius * 2,
              marginLeft: -radius,
              marginTop: -radius,
              borderRadius: "50%",
              border: `3px solid ${color}`,
              opacity,
              boxShadow: `0 0 30px ${color}, inset 0 0 30px ${color}`,
            }}
          />
        );
      })}
    </div>
  );
};
