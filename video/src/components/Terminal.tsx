import { interpolate } from "remotion";

interface TerminalLine {
  type: "prompt" | "command" | "danger" | "success" | "output";
  text: string;
}

interface TerminalProps {
  lines: TerminalLine[];
  frame: number;
}

export const Terminal: React.FC<TerminalProps> = ({ lines, frame }) => {
  const getLineColor = (type: TerminalLine["type"]) => {
    switch (type) {
      case "prompt":
        return "#94a3b8";
      case "command":
        return "#fff";
      case "danger":
        return "#ef4444";
      case "success":
        return "#22c55e";
      case "output":
        return "#64748b";
      default:
        return "#fff";
    }
  };

  return (
    <div
      style={{
        background: "#1e1e2e",
        borderRadius: 12,
        padding: 24,
        fontFamily: "'SF Mono', 'Fira Code', monospace",
        fontSize: 18,
        boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.5)",
        border: "1px solid #2d2d3d",
      }}
    >
      {/* Terminal header */}
      <div
        style={{
          display: "flex",
          gap: 8,
          marginBottom: 16,
          paddingBottom: 12,
          borderBottom: "1px solid #2d2d3d",
        }}
      >
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            background: "#ff5f56",
          }}
        />
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            background: "#ffbd2e",
          }}
        />
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            background: "#27ca40",
          }}
        />
      </div>

      {/* Terminal content */}
      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        {lines.map((line, index) => {
          const lineDelay = index * 15;
          const opacity = interpolate(
            frame - lineDelay,
            [0, 10],
            [0, 1],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
          );

          return (
            <div
              key={index}
              style={{
                opacity,
                color: getLineColor(line.type),
                display: "flex",
                alignItems: "center",
                gap: 8,
              }}
            >
              {line.type === "danger" && (
                <span style={{ color: "#ef4444" }}>⚠️</span>
              )}
              {line.text}
            </div>
          );
        })}
      </div>
    </div>
  );
};
