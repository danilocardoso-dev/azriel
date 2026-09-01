import { useEffect, useRef, type RefObject } from "react";
import { HAND_CONNECTIONS } from "../../engineering/config";
import type { TrackedHand } from "../../engineering/types";

interface CameraPreviewProps {
  videoRef: RefObject<HTMLVideoElement | null>;
  hand: TrackedHand | null;
  visible: boolean;
  debug: boolean;
  fps: number;
}

export function CameraPreview({ videoRef, hand, visible, debug, fps }: CameraPreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const context = canvas.getContext("2d");
    if (!context) return;
    context.clearRect(0, 0, canvas.width, canvas.height);
    if (!debug || !hand) return;

    context.strokeStyle = "rgba(70, 233, 255, .68)";
    context.fillStyle = "#46e9ff";
    context.lineWidth = 1.5;
    for (const [start, end] of HAND_CONNECTIONS) {
      const first = hand.landmarks[start];
      const second = hand.landmarks[end];
      if (!first || !second) continue;
      context.beginPath();
      context.moveTo(first.x * canvas.width, first.y * canvas.height);
      context.lineTo(second.x * canvas.width, second.y * canvas.height);
      context.stroke();
    }
    for (const landmark of hand.landmarks) {
      context.beginPath();
      context.arc(landmark.x * canvas.width, landmark.y * canvas.height, 2.4, 0, Math.PI * 2);
      context.fill();
    }
  }, [debug, hand]);

  return (
    <section className="camera-preview">
      <header><strong>CAMERA PREVIEW</strong><span>{visible ? "ON" : "OFF"}</span></header>
      <div className={`camera-preview__feed ${visible ? "" : "camera-preview__feed--hidden"}`}>
        <video ref={videoRef} autoPlay muted playsInline aria-label="Preview local e espelhado da webcam" />
        <canvas ref={canvasRef} width={640} height={480} aria-hidden={!debug} />
        {!visible && <p>PREVIEW OCULTO<br /><small>TRACKING PODE CONTINUAR ATIVO</small></p>}
        {debug && visible && <div className="camera-preview__debug">
          <span>{hand ? hand.handedness?.toUpperCase() ?? "HAND" : "NO HAND"}</span>
          <span>{hand?.confidence !== undefined ? `${Math.round(hand.confidence * 100)}%` : "—"}</span>
          <span>{fps} FPS</span>
        </div>}
      </div>
    </section>
  );
}
