import {
  AbsoluteFill,
  Sequence,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
} from "remotion";
import { GlitchText } from "../components/GlitchText";
import { TypewriterText } from "../components/TypewriterText";
import { GeometricBackground } from "../components/GeometricBackground";
import { ShockWave } from "../components/ShockWave";
import { Particles } from "../components/Particles";
import { ScanLine } from "../components/ScanLine";
import { Logo } from "../components/Logo";
import { FingerprintScan } from "../components/FingerprintScan";

// Total: 540 frames = 18 seconds @ 30fps
export const VetoIntro: React.FC = () => {
  return (
    <AbsoluteFill
      style={{
        background: "#0a0a0f",
        fontFamily: "system-ui, -apple-system, sans-serif",
      }}
    >
      {/* Scene 1: AI Request (0-90 frames = 0-3s) */}
      <Sequence from={0} durationInFrames={90}>
        <RequestScene />
      </Sequence>

      {/* Scene 2: Auth Gate (85-170 frames = 2.8-5.7s) */}
      <Sequence from={85} durationInFrames={85}>
        <AuthGateScene />
      </Sequence>

      {/* Scene 3: Authentication (165-340 frames = 5.5-11.3s) */}
      <Sequence from={165} durationInFrames={175}>
        <AuthScene />
      </Sequence>

      {/* Scene 4: Success Execution (335-465 frames = 11.2-15.5s) */}
      <Sequence from={335} durationInFrames={130}>
        <ExecutionScene />
      </Sequence>

      {/* Scene 5: Logo + CTA (460-540 frames = 15.3-18s) */}
      <Sequence from={460} durationInFrames={80}>
        <CTAScene />
      </Sequence>
    </AbsoluteFill>
  );
};

// Scene 1: AI Agent requesting to execute a command
const RequestScene: React.FC = () => {
  const frame = useCurrentFrame();

  const bgOpacity = interpolate(frame, [0, 20], [0, 1]);

  // Subtle pulse effect for "waiting" state
  const pulseOpacity = interpolate(frame % 40, [0, 20, 40], [0.5, 1, 0.5]);

  return (
    <AbsoluteFill>
      {/* Geometric grid background with amber tint */}
      <div style={{ opacity: bgOpacity }}>
        <GeometricBackground variant="grid" color="rgba(245, 158, 11, 0.08)" speed={0.3} />
      </div>

      {/* Main content */}
      <AbsoluteFill
        style={{
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
          gap: 24,
        }}
      >
        {/* AI Agent label */}
        <div
          style={{
            opacity: interpolate(frame, [5, 15], [0, 1]),
            display: "flex",
            alignItems: "center",
            gap: 12,
            background: "rgba(245, 158, 11, 0.15)",
            padding: "12px 24px",
            borderRadius: 12,
            border: "1px solid rgba(245, 158, 11, 0.3)",
          }}
        >
          <span style={{ fontSize: 32 }}>🤖</span>
          <span
            style={{
              color: "#f59e0b",
              fontSize: 20,
              fontWeight: 700,
              letterSpacing: 2,
              textTransform: "uppercase",
            }}
          >
            AI Agent
          </span>
        </div>

        {/* Terminal window */}
        <div
          style={{
            opacity: interpolate(frame, [10, 25], [0, 1]),
            transform: `translateY(${interpolate(frame, [10, 25], [20, 0])}px)`,
            background: "rgba(15, 15, 25, 0.95)",
            borderRadius: 16,
            padding: 32,
            border: "1px solid rgba(245, 158, 11, 0.2)",
            boxShadow: "0 25px 80px rgba(0, 0, 0, 0.5)",
            minWidth: 550,
          }}
        >
          {/* Terminal header */}
          <div
            style={{
              display: "flex",
              gap: 8,
              marginBottom: 20,
              paddingBottom: 16,
              borderBottom: "1px solid rgba(255, 255, 255, 0.1)",
            }}
          >
            <div style={{ width: 12, height: 12, borderRadius: "50%", background: "#ff5f56" }} />
            <div style={{ width: 12, height: 12, borderRadius: "50%", background: "#ffbd2e" }} />
            <div style={{ width: 12, height: 12, borderRadius: "50%", background: "#27ca40" }} />
            <span style={{ marginLeft: 12, color: "#64748b", fontSize: 13, fontFamily: "monospace" }}>
              veto
            </span>
          </div>

          {/* Request message */}
          <div style={{ marginBottom: 16 }}>
            <span style={{ color: "#94a3b8", fontFamily: "monospace", fontSize: 14 }}>
              AI wants to execute:
            </span>
          </div>

          {/* Command */}
          <div
            style={{
              fontFamily: "'SF Mono', monospace",
              fontSize: 28,
              color: "#f59e0b",
              marginBottom: 20,
            }}
          >
            terraform apply -auto-approve
          </div>

          {/* Waiting indicator */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              opacity: interpolate(frame, [40, 55], [0, pulseOpacity]),
            }}
          >
            <div
              style={{
                width: 10,
                height: 10,
                borderRadius: "50%",
                background: "#f59e0b",
                boxShadow: "0 0 10px #f59e0b",
              }}
            />
            <span style={{ color: "#94a3b8", fontSize: 14, fontFamily: "monospace" }}>
              Awaiting your authorization...
            </span>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// Scene 2: Auth Gate - You hold the key
const AuthGateScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Key animation - dramatic entrance
  const keyScale = spring({
    frame: frame - 10,
    fps,
    config: { damping: 8, stiffness: 120 },
  });

  const keyRotation = interpolate(frame, [10, 30], [-30, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Glow effect
  const glowIntensity = interpolate(frame % 30, [0, 15, 30], [20, 40, 20]);

  // Text fade in
  const textOpacity = interpolate(frame, [35, 50], [0, 1], {
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill>
      <GeometricBackground variant="grid" color="rgba(59, 130, 246, 0.1)" speed={0} />

      {/* Shockwave effect */}
      <ShockWave startFrame={15} color="rgba(59, 130, 246, 0.4)" maxRadius={800} />

      {/* Particles */}
      <Particles startFrame={20} count={30} color="#3b82f6" />

      {/* Central content */}
      <AbsoluteFill
        style={{
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
          gap: 30,
        }}
      >
        {/* Key emoji with glow */}
        <div
          style={{
            fontSize: 150,
            transform: `scale(${keyScale}) rotate(${keyRotation}deg)`,
            filter: `drop-shadow(0 0 ${glowIntensity}px rgba(59, 130, 246, 0.8))`,
          }}
        >
          🔑
        </div>

        {/* Message */}
        <div
          style={{
            opacity: textOpacity,
            textAlign: "center",
          }}
        >
          <div
            style={{
              fontSize: 42,
              fontWeight: 700,
              color: "#f8fafc",
              letterSpacing: 2,
              marginBottom: 12,
            }}
          >
            You hold the key
          </div>
          <div
            style={{
              fontSize: 20,
              color: "#94a3b8",
              fontWeight: 400,
            }}
          >
            Authenticate to continue
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// Scene 3: Authentication - Fingerprint scan & success
const AuthScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const panelOpacity = interpolate(frame, [0, 20], [0, 1]);

  // Badge for risk level
  const badgeScale = spring({
    frame: frame - 25,
    fps,
    config: { damping: 10, stiffness: 200 },
  });

  return (
    <AbsoluteFill>
      <GeometricBackground variant="dots" color="rgba(59, 130, 246, 0.12)" speed={0.2} />

      <AbsoluteFill
        style={{
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
          gap: 30,
          opacity: panelOpacity,
        }}
      >
        {/* Analysis panel */}
        <div
          style={{
            background: "rgba(15, 23, 42, 0.9)",
            borderRadius: 20,
            padding: "30px 50px",
            border: "1px solid rgba(59, 130, 246, 0.2)",
            boxShadow: "0 20px 60px rgba(0, 0, 0, 0.4)",
            position: "relative",
            overflow: "hidden",
          }}
        >
          {/* Scan line effect */}
          <ScanLine startFrame={10} duration={30} width={400} />

          {/* Command */}
          <div
            style={{
              fontFamily: "'SF Mono', monospace",
              fontSize: 20,
              color: "#f8fafc",
              marginBottom: 16,
              position: "relative",
              zIndex: 1,
            }}
          >
            <span style={{ color: "#64748b" }}>$ </span>
            terraform apply -auto-approve
          </div>

          {/* Risk tags */}
          <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
            {["DESTRUCTIVE", "INFRA", "AUTO-APPROVE"].map((tag, i) => (
              <span
                key={tag}
                style={{
                  opacity: interpolate(frame, [35 + i * 8, 45 + i * 8], [0, 1]),
                  background: "rgba(239, 68, 68, 0.2)",
                  color: "#fca5a5",
                  padding: "6px 14px",
                  borderRadius: 6,
                  fontSize: 13,
                  fontWeight: 600,
                  fontFamily: "monospace",
                  border: "1px solid rgba(239, 68, 68, 0.3)",
                }}
              >
                {tag}
              </span>
            ))}
          </div>
        </div>

        {/* Risk badge */}
        {frame > 55 && (
          <div
            style={{
              transform: `scale(${badgeScale})`,
              background: "linear-gradient(135deg, #7f1d1d, #991b1b)",
              color: "#fecaca",
              padding: "10px 28px",
              borderRadius: 8,
              fontSize: 20,
              fontWeight: 700,
              fontFamily: "monospace",
              letterSpacing: 3,
              border: "2px solid #ef4444",
              boxShadow: "0 0 20px rgba(239, 68, 68, 0.4)",
            }}
          >
            CRITICAL
          </div>
        )}

        {/* Fingerprint authentication */}
        {frame > 70 && (
          <div
            style={{
              marginTop: 10,
              opacity: interpolate(frame, [70, 85], [0, 1]),
            }}
          >
            <FingerprintScan startFrame={80} result="authorized" />
          </div>
        )}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// Scene 4: Success - Command executed with your authorization
const ExecutionScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Flash transition (green for success)
  const flashOpacity = interpolate(frame, [0, 5, 10], [0, 0.6, 0]);

  // Checkmark animation
  const checkScale = spring({
    frame: frame - 10,
    fps,
    config: { damping: 8, stiffness: 150 },
  });

  // Terminal output lines
  const outputLines = [
    { text: "$ terraform apply -auto-approve", color: "#94a3b8", delay: 15 },
    { text: "aws_instance.web: Creating...", color: "#22c55e", delay: 25 },
    { text: "aws_instance.web: Creation complete", color: "#22c55e", delay: 35 },
    { text: "aws_db_instance.main: Creating...", color: "#22c55e", delay: 45 },
    { text: "aws_db_instance.main: Creation complete", color: "#22c55e", delay: 55 },
    { text: "", color: "#22c55e", delay: 65 },
    { text: "Apply complete! Resources: 2 added.", color: "#3b82f6", delay: 70 },
  ];

  // Final message
  const showFinal = frame > 85;
  const finalY = interpolate(
    frame,
    [85, 95, 105, 115],
    [-15, 5, -2, 0],
    { extrapolateRight: "clamp" }
  );
  const finalScale = interpolate(
    frame,
    [85, 95, 105, 115],
    [0.95, 1.03, 0.99, 1],
    { extrapolateRight: "clamp" }
  );

  return (
    <AbsoluteFill
      style={{
        background: "linear-gradient(135deg, #0a0a0f 0%, #0a0f0a 50%, #0a0a0f 100%)",
      }}
    >
      <GeometricBackground variant="dots" color="rgba(34, 197, 94, 0.08)" speed={0.1} />

      {/* Flash transition */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: "#22c55e",
          opacity: flashOpacity,
          zIndex: 100,
        }}
      />

      {/* Main content */}
      <AbsoluteFill
        style={{
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
          gap: 30,
        }}
      >
        {/* Success checkmark */}
        <div
          style={{
            transform: `scale(${checkScale})`,
            background: "linear-gradient(135deg, #166534, #15803d)",
            borderRadius: "50%",
            width: 80,
            height: 80,
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            boxShadow: "0 0 40px rgba(34, 197, 94, 0.5)",
            border: "3px solid #22c55e",
          }}
        >
          <span style={{ fontSize: 40, color: "#fff" }}>✓</span>
        </div>

        {/* Terminal output */}
        <div
          style={{
            background: "rgba(15, 23, 42, 0.9)",
            borderRadius: 12,
            padding: "20px 28px",
            border: "1px solid rgba(34, 197, 94, 0.3)",
            minWidth: 480,
            fontFamily: "'SF Mono', monospace",
            fontSize: 14,
          }}
        >
          {outputLines.map((line, index) => {
            const lineOpacity = interpolate(
              frame,
              [line.delay, line.delay + 5],
              [0, 1],
              { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
            );

            return (
              <div
                key={index}
                style={{
                  color: line.color,
                  opacity: lineOpacity,
                  marginBottom: 6,
                  minHeight: 18,
                }}
              >
                {line.text}
              </div>
            );
          })}
        </div>

        {/* Final message */}
        {showFinal && (
          <div
            style={{
              transform: `translateY(${finalY}px) scale(${finalScale})`,
              textAlign: "center",
              opacity: interpolate(frame, [85, 95, 115, 130], [0, 1, 1, 0], {
                extrapolateRight: "clamp",
              }),
            }}
          >
            <div
              style={{
                color: "#22c55e",
                fontSize: 36,
                fontWeight: 300,
                letterSpacing: 2,
                textShadow: "0 0 30px rgba(34, 197, 94, 0.5)",
              }}
            >
              You approved. It's deployed.
            </div>
          </div>
        )}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// Scene 5: Logo and CTA
const CTAScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Fade in at start
  const sceneOpacity = interpolate(frame, [0, 15], [0, 1], {
    extrapolateRight: "clamp",
  });

  const logoScale = spring({
    frame: frame - 5,
    fps,
    config: { damping: 10, stiffness: 150 },
  });

  const taglineOpacity = interpolate(frame, [12, 20], [0, 1], {
    extrapolateRight: "clamp",
  });
  const taglineY = interpolate(frame, [12, 20], [15, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const cmdOpacity = interpolate(frame, [22, 28], [0, 1]);

  const finalGlow = interpolate(frame, [35, 55], [0, 20], { extrapolateRight: "clamp" });

  return (
    <AbsoluteFill
      style={{
        background: "linear-gradient(135deg, #0a0a0f 0%, #0f172a 50%, #0a0a0f 100%)",
        opacity: sceneOpacity,
      }}
    >
      <GeometricBackground variant="dots" color="rgba(34, 197, 94, 0.08)" speed={0.1} />

      <AbsoluteFill
        style={{
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
          gap: 24,
        }}
      >
        {/* Logo */}
        <div style={{ transform: `scale(${logoScale})` }}>
          <Logo size={140} />
        </div>

        {/* Tagline */}
        <div
          style={{
            opacity: taglineOpacity,
            transform: `translateY(${taglineY}px)`,
            color: "#94a3b8",
            fontSize: 24,
            fontWeight: 300,
            letterSpacing: 2,
          }}
        >
The sudo for AI Agents
        </div>

        {/* Install command */}
        <div
          style={{
            opacity: cmdOpacity,
            marginTop: 20,
            padding: "14px 28px",
            background: "rgba(34, 197, 94, 0.1)",
            borderRadius: 10,
            border: "1px solid rgba(34, 197, 94, 0.3)",
            boxShadow: `0 0 ${finalGlow}px rgba(34, 197, 94, 0.4)`,
          }}
        >
          <TypewriterText
            text="curl -sSL https://github.com/runkids/veto | bash"
            startFrame={25}
            charsPerFrame={2.5}
            fontSize={18}
            color="#22c55e"
          />
        </div>

        {/* GitHub link */}
        <div
          style={{
            opacity: interpolate(frame, [48, 55], [0, 1]),
            color: "#64748b",
            fontSize: 16,
            marginTop: 16,
          }}
        >
          github.com/runkids/veto
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
