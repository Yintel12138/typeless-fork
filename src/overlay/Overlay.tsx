import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ─── Types ─────────────────────────────────────────────────────────────────────

type Phase = "idle" | "recording" | "processing";

// ─── Constants ────────────────────────────────────────────────────────────────

const BAR_COUNT = 5;
const RMS_POLL_MS = 1000 / 60; // 60fps
const BAR_MIN_H = 4;
const BAR_MAX_H = 36;

// ─── Component ────────────────────────────────────────────────────────────────

export default function Overlay() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [transcript, setTranscript] = useState<string>("");
  const [bars, setBars] = useState<number[]>(Array(BAR_COUNT).fill(BAR_MIN_H));
  const rafRef = useRef<number | null>(null);
  const phaseRef = useRef<Phase>("idle");

  useEffect(() => {
    phaseRef.current = phase;
  }, [phase]);

  // ── RMS polling loop ────────────────────────────────────────────────────────

  const startRmsLoop = useCallback(() => {
    let last = 0;
    const loop = (ts: number) => {
      if (phaseRef.current !== "recording") return;
      if (ts - last >= RMS_POLL_MS) {
        last = ts;
        invoke<number>("get_rms")
          .then((rms) => {
            setBars(
              Array.from({ length: BAR_COUNT }, (_, i) => {
                const offset = ((i - Math.floor(BAR_COUNT / 2)) * 0.15);
                const v = Math.max(0, Math.min(1, rms + offset));
                return BAR_MIN_H + v * (BAR_MAX_H - BAR_MIN_H);
              })
            );
          })
          .catch(() => {});
      }
      rafRef.current = requestAnimationFrame(loop);
    };
    rafRef.current = requestAnimationFrame(loop);
  }, []);

  const stopRmsLoop = useCallback(() => {
    if (rafRef.current !== null) {
      cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    }
    setBars(Array(BAR_COUNT).fill(BAR_MIN_H));
  }, []);

  // ── Tauri event listeners ────────────────────────────────────────────────────

  useEffect(() => {
    const unlisten: Array<() => void> = [];

    (async () => {
      unlisten.push(
        await listen("recording_start", () => {
          setPhase("recording");
          setTranscript("");
          startRmsLoop();
        })
      );

      unlisten.push(
        await listen("recording_stop", () => {
          stopRmsLoop();
          setPhase("processing");
        })
      );

      unlisten.push(
        await listen("processing_start", () => {
          setPhase("processing");
        })
      );

      unlisten.push(
        await listen<string>("transcript_partial", (e) => {
          setTranscript(e.payload);
        })
      );

      unlisten.push(
        await listen<string>("transcript_final", (e) => {
          setTranscript(e.payload);
          setPhase("idle");
          // Auto-hide after 3 s
          setTimeout(() => {
            setTranscript("");
          }, 3000);
        })
      );
    })();

    return () => {
      unlisten.forEach((fn) => fn());
      stopRmsLoop();
    };
  }, [startRmsLoop, stopRmsLoop]);

  // ── Visibility: show when not idle, hide when idle & no text ────────────────

  const visible = phase !== "idle" || transcript.length > 0;

  if (!visible) return null;

  return (
    <div style={styles.capsule}>
      {/* Waveform bars */}
      <div style={styles.barsContainer}>
        {bars.map((h, i) => (
          <div
            key={i}
            style={{
              ...styles.bar,
              height: h,
              background:
                phase === "recording" ? "#5865f2" : "#888",
              transition: phase === "recording"
                ? "height 0.08s ease"
                : "height 0.3s ease",
            }}
          />
        ))}
      </div>

      {/* Status text */}
      <div style={styles.textArea}>
        {phase === "recording" && !transcript && (
          <span style={styles.statusText}>Recording…</span>
        )}
        {phase === "processing" && (
          <span style={{ ...styles.statusText, color: "#f5a623" }}>
            Processing…
          </span>
        )}
        {transcript && (
          <span style={styles.transcriptText}>{transcript}</span>
        )}
      </div>
    </div>
  );
}

// ─── Styles ────────────────────────────────────────────────────────────────────

const styles: Record<string, React.CSSProperties> = {
  capsule: {
    position: "fixed",
    bottom: 0,
    left: 0,
    right: 0,
    display: "flex",
    alignItems: "center",
    gap: 12,
    padding: "0 20px",
    height: 60,
    background: "rgba(24, 25, 28, 0.88)",
    backdropFilter: "blur(20px)",
    WebkitBackdropFilter: "blur(20px)",
    borderRadius: 30,
    boxShadow: "0 8px 32px rgba(0,0,0,0.48)",
    border: "1px solid rgba(255,255,255,0.08)",
    margin: "10px 10px",
    overflow: "hidden",
  },
  barsContainer: {
    display: "flex",
    alignItems: "center",
    gap: 3,
    flexShrink: 0,
  },
  bar: {
    width: 4,
    borderRadius: 2,
    background: "#5865f2",
  },
  textArea: {
    flex: 1,
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
  statusText: {
    fontFamily: "'Inter', '-apple-system', 'Segoe UI', sans-serif",
    fontSize: 14,
    color: "#aaa",
    fontStyle: "italic",
  },
  transcriptText: {
    fontFamily: "'Inter', '-apple-system', 'Segoe UI', sans-serif",
    fontSize: 14,
    color: "#e0e0e0",
    letterSpacing: 0.2,
  },
};
