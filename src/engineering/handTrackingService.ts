import { FilesetResolver, HandLandmarker } from "@mediapipe/tasks-vision";
import { ENGINEERING_CONFIG } from "./config";
import { handPositions, stabilizeHandIdentity, type PreviousHandPositions, type RawTrackedHand } from "./handIdentity";
import type { HandSide, TrackingFrame } from "./types";

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
  private previousHands: PreviousHandPositions = {};

  async initialize(): Promise<void> {
    if (this.landmarker) return;
    const vision = await FilesetResolver.forVisionTasks(localAsset("mediapipe/wasm"));
    this.landmarker = await HandLandmarker.createFromOptions(vision, {
      baseOptions: {
        modelAssetPath: localAsset("mediapipe/models/hand_landmarker.task"),
        delegate: "CPU",
      },
      runningMode: "VIDEO",
      numHands: 2,
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
    this.previousHands = {};

    const interval = 1000 / ENGINEERING_CONFIG.maximumTrackingFps;
    const loop = (timestamp: number) => {
      if (!this.running || !this.landmarker) return;
      if (video.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA && video.currentTime !== this.lastVideoTime && timestamp - this.lastInferenceAt >= interval) {
        try {
          const result = this.landmarker.detectForVideo(video, timestamp);
          this.lastVideoTime = video.currentTime;
          this.lastInferenceAt = timestamp;
          this.updateFps(timestamp);
          const rawHands = result.landmarks.map((landmarks, index) => this.toRawTrackedHand(landmarks, result.handedness[index]?.[0]));
          const hands = stabilizeHandIdentity(rawHands, this.previousHands);
          this.previousHands = handPositions(hands);
          listener({ hands, fps: this.fps });
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
    this.previousHands = {};
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

  private toRawTrackedHand(
    landmarks: ReadonlyArray<{ x: number; y: number; z: number }>,
    handedness: { categoryName: string; score: number } | undefined,
  ): RawTrackedHand {
    const side = handedness?.categoryName.toLowerCase();
    return {
      landmarks: landmarks.map(({ x, y, z }) => ({ x, y, z })),
      handedness: side === "left" || side === "right" ? side as HandSide : undefined,
      confidence: handedness?.score,
    };
  }
}
