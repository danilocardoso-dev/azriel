import { FilesetResolver, HandLandmarker } from "@mediapipe/tasks-vision";
import { ENGINEERING_CONFIG } from "./config";
import type { TrackedHand, TrackingFrame } from "./types";

export type TrackingListener = (frame: TrackingFrame) => void;
export type TrackingErrorListener = (error: Error) => void;

function localAsset(path: string): string {
  return new URL(path.replace(/^\//, ""), document.baseURI).href;
}

export class HandTrackingService {
  private landmarker: HandLandmarker | null = null;
  private animationFrame: number | null = null;
  private lastVideoTime = -1;
  private lastInferenceAt = 0;
  private fpsStartedAt = 0;
  private fpsFrames = 0;
  private fps = 0;
  private running = false;

  async initialize(): Promise<void> {
    if (this.landmarker) return;
    const vision = await FilesetResolver.forVisionTasks(localAsset("mediapipe/wasm"));
    this.landmarker = await HandLandmarker.createFromOptions(vision, {
      baseOptions: {
        modelAssetPath: localAsset("mediapipe/models/hand_landmarker.task"),
        delegate: "CPU",
      },
      runningMode: "VIDEO",
      numHands: 1,
      minHandDetectionConfidence: ENGINEERING_CONFIG.minimumConfidence,
      minHandPresenceConfidence: ENGINEERING_CONFIG.minimumConfidence,
      minTrackingConfidence: ENGINEERING_CONFIG.minimumConfidence,
    });
  }

  start(video: HTMLVideoElement, listener: TrackingListener, onError?: TrackingErrorListener): void {
    if (!this.landmarker) throw new Error("Hand Landmarker ainda não foi inicializado.");
    this.stopLoop();
    this.running = true;
    this.lastVideoTime = -1;
    this.lastInferenceAt = 0;
    this.fpsStartedAt = performance.now();
    this.fpsFrames = 0;
    this.fps = 0;

    const interval = 1000 / ENGINEERING_CONFIG.maximumTrackingFps;
    const loop = (timestamp: number) => {
      if (!this.running || !this.landmarker) return;
      if (video.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA && video.currentTime !== this.lastVideoTime && timestamp - this.lastInferenceAt >= interval) {
        try {
          const result = this.landmarker.detectForVideo(video, timestamp);
          this.lastVideoTime = video.currentTime;
          this.lastInferenceAt = timestamp;
          this.updateFps(timestamp);
          listener({ hand: this.toTrackedHand(result.landmarks[0], result.handedness[0]?.[0]), fps: this.fps });
        } catch (reason) {
          this.stopLoop();
          onError?.(reason instanceof Error ? reason : new Error(String(reason)));
          return;
        }
      }
      this.animationFrame = requestAnimationFrame(loop);
    };
    this.animationFrame = requestAnimationFrame(loop);
  }

  stop(): void {
    this.stopLoop();
    this.landmarker?.close();
    this.landmarker = null;
  }

  private stopLoop(): void {
    this.running = false;
    if (this.animationFrame !== null) cancelAnimationFrame(this.animationFrame);
    this.animationFrame = null;
  }

  private updateFps(timestamp: number): void {
    this.fpsFrames += 1;
    const elapsed = timestamp - this.fpsStartedAt;
    if (elapsed >= 500) {
      this.fps = Math.round((this.fpsFrames * 1000) / elapsed);
      this.fpsFrames = 0;
      this.fpsStartedAt = timestamp;
    }
  }

  private toTrackedHand(
    landmarks: ReadonlyArray<{ x: number; y: number; z: number }> | undefined,
    handedness: { categoryName: string; score: number } | undefined,
  ): TrackedHand | null {
    if (!landmarks?.length) return null;
    const side = handedness?.categoryName.toLowerCase();
    return {
      landmarks: landmarks.map(({ x, y, z }) => ({ x, y, z })),
      handedness: side === "left" || side === "right" ? side : undefined,
      confidence: handedness?.score,
    };
  }
}
