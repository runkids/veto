import { useCurrentFrame, random, interpolate } from "remotion";

interface VHSEffectProps {
  intensity?: number;
  children: React.ReactNode;
}

export const VHSEffect: React.FC<VHSEffectProps> = ({
  intensity = 1,
  children,
}) => {
  const frame = useCurrentFrame();

  const scanlineOpacity = 0.1 * intensity;
  const noiseOpacity = interpolate(
    random(`noise-${frame}`),
    [0, 1],
    [0.02, 0.08]
  ) * intensity;

  const horizontalShift = (random(`h-${frame}`) - 0.5) * 4 * intensity;
  const verticalShift = (random(`v-${frame}`) - 0.5) * 2 * intensity;

  // Occasional tracking glitch
  const hasTrackingGlitch = random(`track-${frame}`) > 0.92;
  const trackingOffset = hasTrackingGlitch ? (random(`trackOff-${frame}`) - 0.5) * 30 : 0;

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        height: "100%",
        overflow: "hidden",
      }}
    >
      {/* Main content with color separation */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          transform: `translate(${horizontalShift}px, ${verticalShift + trackingOffset}px)`,
        }}
      >
        {children}
      </div>

      {/* Scanlines */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `repeating-linear-gradient(
            0deg,
            transparent,
            transparent 2px,
            rgba(0, 0, 0, ${scanlineOpacity}) 2px,
            rgba(0, 0, 0, ${scanlineOpacity}) 4px
          )`,
          pointerEvents: "none",
        }}
      />

      {/* Noise overlay */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E")`,
          opacity: noiseOpacity,
          pointerEvents: "none",
          mixBlendMode: "overlay",
        }}
      />

      {/* Vignette */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: "radial-gradient(ellipse at center, transparent 40%, rgba(0,0,0,0.6) 100%)",
          pointerEvents: "none",
        }}
      />

      {/* Color aberration line */}
      {hasTrackingGlitch && (
        <div
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            top: `${random(`linePos-${frame}`) * 100}%`,
            height: 4,
            background: "linear-gradient(90deg, #ff0000, #00ff00, #0000ff)",
            opacity: 0.5,
            mixBlendMode: "screen",
          }}
        />
      )}
    </div>
  );
};
