import { useCurrentFrame, interpolate } from "remotion";

interface TypewriterTextProps {
  text: string;
  startFrame?: number;
  charsPerFrame?: number;
  fontSize?: number;
  color?: string;
  showCursor?: boolean;
}

export const TypewriterText: React.FC<TypewriterTextProps> = ({
  text,
  startFrame = 0,
  charsPerFrame = 0.5,
  fontSize = 24,
  color = "#22c55e",
  showCursor = true,
}) => {
  const frame = useCurrentFrame();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  const charsToShow = Math.floor(localFrame * charsPerFrame);
  const visibleText = text.slice(0, charsToShow);
  const isComplete = charsToShow >= text.length;

  const cursorOpacity = isComplete
    ? interpolate(frame % 30, [0, 15, 30], [1, 0, 1])
    : 1;

  return (
    <div
      style={{
        fontFamily: "'SF Mono', 'Fira Code', monospace",
        fontSize,
        color,
        display: "flex",
        alignItems: "center",
      }}
    >
      <span>{visibleText}</span>
      {showCursor && (
        <span
          style={{
            opacity: cursorOpacity,
            marginLeft: 2,
            color,
          }}
        >
          █
        </span>
      )}
    </div>
  );
};
