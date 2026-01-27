import { useCurrentFrame, interpolate, spring, useVideoConfig } from "remotion";

interface RiskTagsProps {
  startFrame: number;
  tags?: string[];
}

export const RiskTags: React.FC<RiskTagsProps> = ({
  startFrame,
  tags = ["filesystem", "destructive", "root-level"],
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const localFrame = frame - startFrame;

  if (localFrame < 0) return null;

  return (
    <div
      style={{
        display: "flex",
        gap: 12,
        flexWrap: "wrap",
        justifyContent: "center",
      }}
    >
      {tags.map((tag, index) => {
        const delay = index * 6;
        const tagFrame = localFrame - delay;

        const scale = spring({
          frame: Math.max(0, tagFrame),
          fps,
          config: { damping: 10, stiffness: 300 },
        });

        const opacity = interpolate(
          tagFrame,
          [-3, 3],
          [0, 1],
          { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
        );

        return (
          <div
            key={tag}
            style={{
              opacity,
              transform: `scale(${scale})`,
              background: "rgba(239, 68, 68, 0.15)",
              border: "1px solid rgba(239, 68, 68, 0.4)",
              color: "#fca5a5",
              padding: "6px 14px",
              borderRadius: 20,
              fontSize: 14,
              fontFamily: "'SF Mono', monospace",
              fontWeight: 500,
            }}
          >
            {tag}
          </div>
        );
      })}
    </div>
  );
};
