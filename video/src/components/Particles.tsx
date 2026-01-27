import { useCurrentFrame, interpolate, random } from "remotion";

interface ParticlesProps {
  startFrame: number;
  count?: number;
  color?: string;
}

interface Particle {
  x: number;
  y: number;
  angle: number;
  speed: number;
  size: number;
  rotationSpeed: number;
}

export const Particles: React.FC<ParticlesProps> = ({
  startFrame,
  count = 30,
  color = "#22c55e",
}) => {
  const frame = useCurrentFrame();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  // Generate consistent particles based on seed
  const particles: Particle[] = Array.from({ length: count }, (_, i) => ({
    x: 0,
    y: 0,
    angle: random(`angle-${i}`) * Math.PI * 2,
    speed: 5 + random(`speed-${i}`) * 15,
    size: 4 + random(`size-${i}`) * 8,
    rotationSpeed: (random(`rot-${i}`) - 0.5) * 20,
  }));

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        left: "50%",
        pointerEvents: "none",
      }}
    >
      {particles.map((particle, index) => {
        const distance = particle.speed * localFrame;
        const x = Math.cos(particle.angle) * distance;
        const y = Math.sin(particle.angle) * distance;
        const rotation = particle.rotationSpeed * localFrame;

        const opacity = interpolate(
          localFrame,
          [0, 5, 40],
          [0, 1, 0],
          { extrapolateRight: "clamp" }
        );

        const scale = interpolate(
          localFrame,
          [0, 10, 40],
          [0.5, 1, 0.3],
          { extrapolateRight: "clamp" }
        );

        return (
          <div
            key={index}
            style={{
              position: "absolute",
              width: particle.size,
              height: particle.size,
              background: color,
              transform: `translate(${x}px, ${y}px) rotate(${rotation}deg) scale(${scale})`,
              opacity,
              boxShadow: `0 0 10px ${color}`,
            }}
          />
        );
      })}
    </div>
  );
};
