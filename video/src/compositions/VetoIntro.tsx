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
import { ProgressBar } from "../components/ProgressBar";
import { GeometricBackground } from "../components/GeometricBackground";
import { ShockWave } from "../components/ShockWave";
import { Particles } from "../components/Particles";
import { ScanLine } from "../components/ScanLine";
import { RiskTags } from "../components/RiskTags";
import { AuthMethods } from "../components/AuthMethods";
import { DisasterScene } from "../components/DisasterScene";
import { Logo } from "../components/Logo";

// Total: 540 frames = 18 seconds @ 30fps
export const VetoIntro: React.FC = () => {
  return (
    <AbsoluteFill
      style={{
        background: "#0a0a0f",
        fontFamily: "system-ui, -apple-system, sans-serif",
      }}
    >
      {/* Scene 1: Crisis Hook (0-80 frames = 0-2.7s) */}
      <Sequence from={0} durationInFrames={80}>
        <CrisisScene />
      </Sequence>

      {/* Scene 2: Interception (75-160 frames = 2.5-5.3s) */}
      <Sequence from={75} durationInFrames={85}>
        <InterceptionScene />
      </Sequence>

      {/* Scene 3: Risk Analysis + Auth (155-380 frames = 5.2-12.7s) - MORE TIME for auth cycle */}
      <Sequence from={155} durationInFrames={225}>
        <AnalysisScene />
      </Sequence>

      {/* Scene 4: Contrast Flash (375-475 frames = 12.5-15.8s) */}
      <Sequence from={375} durationInFrames={100}>
        <ContrastScene />
      </Sequence>

      {/* Scene 5: Logo + CTA (470-540 frames = 15.7-18s) */}
      <Sequence from={465} durationInFrames={75}>
        <CTAScene />
      </Sequence>
    </AbsoluteFill>
  );
};

// Scene 1: Crisis - Command being executed
const CrisisScene: React.FC = () => {
  const frame = useCurrentFrame();

  const bgOpacity = interpolate(frame, [0, 20], [0, 1]);
  const vignetteIntensity = interpolate(frame, [30, 60], [0, 0.8], {
    extrapolateRight: "clamp",
  });

  const shakeX = frame > 40 ? (Math.sin(frame * 0.8) * interpolate(frame, [40, 60], [0, 3])) : 0;
  const shakeY = frame > 40 ? (Math.cos(frame * 1.1) * interpolate(frame, [40, 60], [0, 2])) : 0;

  return (
    <AbsoluteFill
      style={{
        transform: `translate(${shakeX}px, ${shakeY}px)`,
      }}
    >
      {/* Geometric grid background with 3D perspective */}
      <div style={{ opacity: bgOpacity }}>
        <GeometricBackground variant="grid" color="rgba(239, 68, 68, 0.08)" speed={0.3} />
      </div>

      {/* Main content */}
      <AbsoluteFill
        style={{
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
          gap: 30,
        }}
      >
        {/* Terminal window */}
        <div
          style={{
            background: "rgba(15, 15, 25, 0.95)",
            borderRadius: 16,
            padding: 32,
            border: "1px solid rgba(255, 255, 255, 0.1)",
            boxShadow: "0 25px 80px rgba(0, 0, 0, 0.5)",
            minWidth: 500,
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
              AI Agent Terminal
            </span>
          </div>

          {/* Command with glitch */}
          <div style={{ marginBottom: 24 }}>
            <span style={{ color: "#64748b", fontFamily: "monospace", fontSize: 14 }}>
              $ executing command...
            </span>
          </div>

          <GlitchText
            text="rm -rf /"
            fontSize={42}
            color="#ef4444"
            glitchIntensity={frame > 30 ? interpolate(frame, [30, 60], [0.5, 2]) : 0}
          />

          {/* Progress bar */}
          <div style={{ marginTop: 30 }}>
            <ProgressBar startFrame={15} duration={50} width={436} />
          </div>
        </div>
      </AbsoluteFill>

      {/* Red vignette danger effect */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(ellipse at center, transparent 30%, rgba(239, 68, 68, ${vignetteIntensity * 0.3}) 100%)`,
          pointerEvents: "none",
        }}
      />
    </AbsoluteFill>
  );
};

// Scene 2: Interception - veto blocks the command
const InterceptionScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Time freeze effect - desaturate and cool color shift
  const saturation = interpolate(frame, [0, 10], [100, 30], { extrapolateRight: "clamp" });
  const hueShift = interpolate(frame, [0, 10], [0, -30], { extrapolateRight: "clamp" });

  // Hand entry animation
  const handScale = spring({
    frame: frame - 5,
    fps,
    config: { damping: 8, stiffness: 150 },
  });

  const handX = interpolate(frame, [5, 20], [400, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // BLOCKED text
  const blockedScale = spring({
    frame: frame - 25,
    fps,
    config: { damping: 10, stiffness: 200 },
  });

  const blockedOpacity = interpolate(frame, [25, 35], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        filter: `saturate(${saturation}%) hue-rotate(${hueShift}deg)`,
      }}
    >
      <GeometricBackground variant="grid" color="rgba(34, 197, 94, 0.1)" speed={0} />

      {/* Shockwave effect */}
      <ShockWave startFrame={15} color="rgba(34, 197, 94, 0.5)" maxRadius={900} />

      {/* Particles */}
      <Particles startFrame={15} count={40} color="#22c55e" />

      {/* Central content */}
      <AbsoluteFill
        style={{
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
        }}
      >
        {/* Frozen command (shattered effect) */}
        <div
          style={{
            opacity: interpolate(frame, [20, 40], [1, 0.3]),
            transform: `scale(${interpolate(frame, [15, 30], [1, 0.8])})`,
            filter: `blur(${interpolate(frame, [20, 40], [0, 2])}px)`,
          }}
        >
          <GlitchText text="rm -rf /" fontSize={36} color="#ef4444" glitchIntensity={0.3} />
        </div>

        {/* Hand emoji entering */}
        <div
          style={{
            position: "absolute",
            transform: `translateX(${handX}px) scale(${handScale})`,
            fontSize: 150,
            filter: "drop-shadow(0 0 40px rgba(34, 197, 94, 0.6))",
          }}
        >
          ✋
        </div>

        {/* BLOCKED text */}
        <div
          style={{
            position: "absolute",
            top: "65%",
            opacity: blockedOpacity,
            transform: `scale(${blockedScale})`,
          }}
        >
          <div
            style={{
              fontSize: 72,
              fontWeight: 800,
              color: "#22c55e",
              letterSpacing: 8,
              textShadow: "0 0 40px rgba(34, 197, 94, 0.8), 0 0 80px rgba(34, 197, 94, 0.4)",
            }}
          >
            BLOCKED
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// Scene 3: Analysis - Risk evaluation and authentication (160 frames)
const AnalysisScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const panelOpacity = interpolate(frame, [0, 20], [0, 1]);

  // Critical badge animation - delayed
  const badgeScale = spring({
    frame: frame - 70,
    fps,
    config: { damping: 10, stiffness: 200 },
  });

  const badgeGlow = interpolate(frame % 30, [0, 15, 30], [15, 25, 15]);

  return (
    <AbsoluteFill>
      <GeometricBackground variant="dots" color="rgba(59, 130, 246, 0.15)" speed={0.2} />

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
          {/* Scan line effect - slower */}
          <ScanLine startFrame={15} duration={35} width={400} />

          {/* Command being analyzed */}
          <div
            style={{
              fontFamily: "'SF Mono', monospace",
              fontSize: 28,
              color: "#f8fafc",
              marginBottom: 20,
              position: "relative",
              zIndex: 1,
            }}
          >
            <span style={{ color: "#64748b" }}>$ </span>
            rm -rf /
          </div>

          {/* Risk tags - delayed */}
          <RiskTags startFrame={45} />
        </div>

        {/* CRITICAL badge - more delay */}
        {frame > 65 && (
          <div
            style={{
              transform: `scale(${badgeScale})`,
              background: "linear-gradient(135deg, #7f1d1d, #991b1b)",
              color: "#fecaca",
              padding: "12px 32px",
              borderRadius: 8,
              fontSize: 24,
              fontWeight: 700,
              fontFamily: "monospace",
              letterSpacing: 4,
              boxShadow: `0 0 ${badgeGlow}px #ef4444, 0 0 ${badgeGlow * 2}px rgba(239, 68, 68, 0.3)`,
              border: "2px solid #ef4444",
            }}
          >
            CRITICAL
          </div>
        )}

        {/* Auth methods - more time to cycle through */}
        {frame > 85 && (
          <div style={{ marginTop: 10 }}>
            <div
              style={{
                color: "#94a3b8",
                fontSize: 16,
                marginBottom: 20,
                textAlign: "center",
                opacity: interpolate(frame, [85, 100], [0, 1]),
              }}
            >
              Authentication Required
            </div>
            <AuthMethods startFrame={95} />
          </div>
        )}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

// Scene 4: Contrast - What would have happened without veto (100 frames)
const ContrastScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Glitch transition at start
  const glitchIntensity = interpolate(frame, [0, 5, 10], [0, 1, 0]);

  // Show disaster scene - shorter to give more time to "Crisis averted"
  const showDisaster = frame >= 5 && frame < 45;

  // "Crisis averted" text with relief animation
  const avertedOpacity = interpolate(frame, [45, 52], [0, 1]);

  // Relief animation: tense up → exhale down → gentle bounce back
  const reliefY = interpolate(
    frame,
    [45, 50, 60, 70, 80],
    [-20, -25, 8, -2, 0],
    { extrapolateRight: "clamp" }
  );

  // Scale: hold breath → release → settle
  const reliefScale = interpolate(
    frame,
    [45, 50, 60, 70, 80],
    [0.9, 0.95, 1.05, 0.98, 1],
    { extrapolateRight: "clamp" }
  );

  // Letter spacing relaxes
  const reliefSpacing = interpolate(
    frame,
    [45, 60, 75],
    [8, 3, 4],
    { extrapolateRight: "clamp" }
  );

  return (
    <AbsoluteFill>
      {/* Glitch flash */}
      {glitchIntensity > 0 && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: "#fff",
            opacity: glitchIntensity * 0.8,
            zIndex: 100,
          }}
        />
      )}

      {/* Disaster scene */}
      {showDisaster && <DisasterScene startFrame={5} />}

      {/* Crisis averted - with relief animation and fade out */}
      {frame >= 45 && (
        <AbsoluteFill
          style={{
            background: "linear-gradient(135deg, #0a0f0a, #0f1a0f)",
            justifyContent: "center",
            alignItems: "center",
            opacity: interpolate(frame, [45, 52, 85, 100], [0, 1, 1, 0], {
              extrapolateRight: "clamp",
            }),
          }}
        >
          <div
            style={{
              opacity: avertedOpacity,
              transform: `translateY(${reliefY}px) scale(${reliefScale})`,
              color: "#22c55e",
              fontSize: 48,
              fontWeight: 300,
              letterSpacing: reliefSpacing,
              textShadow: "0 0 30px rgba(34, 197, 94, 0.5)",
            }}
          >
            Crisis averted. 😮‍💨
          </div>
        </AbsoluteFill>
      )}
    </AbsoluteFill>
  );
};

// Scene 5: Logo and CTA (75 frames) - faster animations with fade in
const CTAScene: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Fade in at start for smooth transition
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
            letterSpacing: 3,
          }}
        >
          AI Operation Guardian
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
