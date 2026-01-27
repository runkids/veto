import { useCurrentFrame, interpolate, random } from "remotion";
import { VHSEffect } from "./VHSEffect";

interface DisasterSceneProps {
  startFrame: number;
}

const errorMessages = [
  "FATAL ERROR",
  "Data Corrupted",
  "System Failure",
  "Files Deleted",
  "Unrecoverable",
];

export const DisasterScene: React.FC<DisasterSceneProps> = ({ startFrame }) => {
  const frame = useCurrentFrame();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  return (
    <VHSEffect intensity={1.5}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: "linear-gradient(135deg, #1a0000 0%, #330000 50%, #1a0000 100%)",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
          gap: 20,
        }}
      >
        {/* Flashing error messages */}
        {errorMessages.map((msg, index) => {
          const delay = index * 5;
          const msgFrame = localFrame - delay;

          const opacity = interpolate(
            msgFrame,
            [0, 3, 6],
            [0, 1, 0.7],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
          );

          const x = (random(`x-${index}`) - 0.5) * 600;
          const y = (random(`y-${index}`) - 0.5) * 400;
          const rotation = (random(`r-${index}`) - 0.5) * 20;
          const scale = 0.8 + random(`s-${index}`) * 0.8;

          const flickerOpacity = random(`flicker-${frame}-${index}`) > 0.3 ? opacity : opacity * 0.3;

          return (
            <div
              key={index}
              style={{
                position: "absolute",
                left: `calc(50% + ${x}px)`,
                top: `calc(50% + ${y}px)`,
                transform: `translate(-50%, -50%) rotate(${rotation}deg) scale(${scale})`,
                opacity: flickerOpacity,
                color: "#ff0000",
                fontSize: 36,
                fontWeight: 700,
                fontFamily: "system-ui",
                textShadow: "0 0 20px #ff0000, 0 0 40px #ff0000",
                whiteSpace: "nowrap",
              }}
            >
              {msg}
            </div>
          );
        })}

        {/* Falling file icons */}
        {Array.from({ length: 15 }, (_, i) => {
          const startX = random(`fileX-${i}`) * 100;
          const fallSpeed = 5 + random(`fileSpeed-${i}`) * 10;
          const y = localFrame * fallSpeed - 200;
          const rotation = localFrame * (random(`fileRot-${i}`) - 0.5) * 10;

          const opacity = interpolate(
            y,
            [-200, 0, 800],
            [0, 1, 0],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
          );

          return (
            <div
              key={`file-${i}`}
              style={{
                position: "absolute",
                left: `${startX}%`,
                top: y,
                transform: `rotate(${rotation}deg)`,
                opacity,
                fontSize: 32,
              }}
            >
              📄
            </div>
          );
        })}

        {/* Central skull warning */}
        <div
          style={{
            fontSize: 120,
            opacity: interpolate(localFrame % 20, [0, 10, 20], [0.5, 1, 0.5]),
            filter: "drop-shadow(0 0 30px #ff0000)",
          }}
        >
          💀
        </div>
      </div>
    </VHSEffect>
  );
};
