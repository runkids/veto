import { useCurrentFrame, interpolate } from "remotion";

interface GeometricBackgroundProps {
  variant?: "grid" | "dots" | "lines";
  color?: string;
  speed?: number;
}

export const GeometricBackground: React.FC<GeometricBackgroundProps> = ({
  variant = "grid",
  color = "rgba(59, 130, 246, 0.1)",
  speed = 0.5,
}) => {
  const frame = useCurrentFrame();
  const offset = frame * speed;

  if (variant === "grid") {
    return (
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage: `
            linear-gradient(${color} 1px, transparent 1px),
            linear-gradient(90deg, ${color} 1px, transparent 1px)
          `,
          backgroundSize: "60px 60px",
          backgroundPosition: `${offset}px ${offset}px`,
          perspective: "1000px",
          transform: "rotateX(60deg) scale(2.5)",
          transformOrigin: "center 120%",
          opacity: 0.6,
        }}
      />
    );
  }

  if (variant === "dots") {
    return (
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage: `radial-gradient(circle, ${color} 2px, transparent 2px)`,
          backgroundSize: "40px 40px",
          backgroundPosition: `${offset}px ${offset}px`,
        }}
      />
    );
  }

  // Lines variant
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
      }}
    >
      {Array.from({ length: 10 }, (_, i) => {
        const y = (i * 120 + offset) % 1200 - 100;
        const opacity = interpolate(y, [0, 600, 1100], [0.3, 0.1, 0.3]);

        return (
          <div
            key={i}
            style={{
              position: "absolute",
              left: 0,
              right: 0,
              top: y,
              height: 1,
              background: `linear-gradient(90deg, transparent, ${color}, transparent)`,
              opacity,
            }}
          />
        );
      })}
    </div>
  );
};
