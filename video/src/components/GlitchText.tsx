import { useCurrentFrame, interpolate, random } from "remotion";

interface GlitchTextProps {
  text: string;
  fontSize?: number;
  color?: string;
  glitchIntensity?: number;
}

export const GlitchText: React.FC<GlitchTextProps> = ({
  text,
  fontSize = 32,
  color = "#fff",
  glitchIntensity = 1,
}) => {
  const frame = useCurrentFrame();

  const glitchOffset = () => {
    const seed = frame * 0.1;
    return {
      x: (random(`x-${frame}`) - 0.5) * 4 * glitchIntensity,
      y: (random(`y-${frame}`) - 0.5) * 2 * glitchIntensity,
    };
  };

  const offset = glitchOffset();
  const showGlitch = random(`show-${frame}`) > 0.7;

  return (
    <div style={{ position: "relative", fontFamily: "'SF Mono', monospace" }}>
      {/* Main text */}
      <div
        style={{
          fontSize,
          color,
          fontWeight: 600,
          position: "relative",
          zIndex: 2,
        }}
      >
        {text}
      </div>

      {/* Glitch layers */}
      {showGlitch && (
        <>
          <div
            style={{
              position: "absolute",
              top: offset.y,
              left: offset.x + 2,
              fontSize,
              color: "#ff0040",
              fontWeight: 600,
              opacity: 0.8,
              zIndex: 1,
              clipPath: `inset(${random(`clip-${frame}`) * 50}% 0 ${random(`clip2-${frame}`) * 50}% 0)`,
            }}
          >
            {text}
          </div>
          <div
            style={{
              position: "absolute",
              top: -offset.y,
              left: -offset.x - 2,
              fontSize,
              color: "#00ffff",
              fontWeight: 600,
              opacity: 0.8,
              zIndex: 1,
              clipPath: `inset(${random(`clip3-${frame}`) * 50}% 0 ${random(`clip4-${frame}`) * 50}% 0)`,
            }}
          >
            {text}
          </div>
        </>
      )}
    </div>
  );
};
