import { interpolate, spring, useVideoConfig } from "remotion";

interface FlowDiagramProps {
  frame: number;
}

export const FlowDiagram: React.FC<FlowDiagramProps> = ({ frame }) => {
  const { fps } = useVideoConfig();

  const box1Opacity = interpolate(frame, [0, 15], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const arrow1Opacity = interpolate(frame, [15, 30], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const box2Opacity = interpolate(frame, [30, 45], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const arrow2Opacity = interpolate(frame, [45, 60], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const box3Opacity = interpolate(frame, [60, 75], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const vetoGlow = interpolate(frame, [30, 45, 75], [0, 30, 15], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const BoxStyle: React.CSSProperties = {
    padding: "20px 30px",
    borderRadius: 12,
    fontWeight: 600,
    fontSize: 18,
    textAlign: "center",
    minWidth: 160,
  };

  const ArrowStyle: React.CSSProperties = {
    fontSize: 32,
    color: "#64748b",
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 20,
      }}
    >
      {/* AI Assistant */}
      <div
        style={{
          ...BoxStyle,
          opacity: box1Opacity,
          background: "linear-gradient(135deg, #3b82f6, #2563eb)",
          color: "#fff",
          boxShadow: "0 10px 25px rgba(59, 130, 246, 0.3)",
        }}
      >
        <div style={{ fontSize: 32, marginBottom: 8 }}>🤖</div>
        AI Assistant
      </div>

      {/* Arrow 1 */}
      <div style={{ ...ArrowStyle, opacity: arrow1Opacity }}>→</div>

      {/* Veto */}
      <div
        style={{
          ...BoxStyle,
          opacity: box2Opacity,
          background: "linear-gradient(135deg, #22c55e, #16a34a)",
          color: "#fff",
          boxShadow: `0 0 ${vetoGlow}px rgba(34, 197, 94, 0.5), 0 10px 25px rgba(34, 197, 94, 0.3)`,
          border: "2px solid rgba(255, 255, 255, 0.2)",
        }}
      >
        <div style={{ fontSize: 32, marginBottom: 8 }}>✋</div>
        veto
        <div style={{ fontSize: 12, marginTop: 4, opacity: 0.8 }}>
          Parse • Evaluate • Auth
        </div>
      </div>

      {/* Arrow 2 */}
      <div style={{ ...ArrowStyle, opacity: arrow2Opacity }}>→</div>

      {/* Shell */}
      <div
        style={{
          ...BoxStyle,
          opacity: box3Opacity,
          background: "linear-gradient(135deg, #1e1e2e, #2d2d3d)",
          color: "#fff",
          boxShadow: "0 10px 25px rgba(0, 0, 0, 0.3)",
          border: "1px solid #3d3d4d",
        }}
      >
        <div style={{ fontSize: 32, marginBottom: 8 }}>💻</div>
        Shell
      </div>
    </div>
  );
};
