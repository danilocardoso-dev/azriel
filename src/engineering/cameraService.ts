import { ENGINEERING_CONFIG } from "./config";

export class CameraAccessError extends Error {
  constructor(public readonly code: "unsupported" | "denied" | "busy" | "unavailable" | "unknown", message: string) {
    super(message);
    this.name = "CameraAccessError";
  }
}

function cameraError(reason: unknown): CameraAccessError {
  if (!(reason instanceof DOMException)) return new CameraAccessError("unknown", reason instanceof Error ? reason.message : String(reason));
  if (reason.name === "NotAllowedError" || reason.name === "SecurityError") return new CameraAccessError("denied", "Permissão de câmera negada. Autorize o Azriel nas configurações de privacidade do Windows.");
  if (reason.name === "NotReadableError" || reason.name === "AbortError") return new CameraAccessError("busy", "A câmera está ocupada ou não pôde ser iniciada.");
  if (reason.name === "NotFoundError" || reason.name === "OverconstrainedError") return new CameraAccessError("unavailable", "Nenhuma câmera compatível foi encontrada.");
  return new CameraAccessError("unknown", reason.message || "Não foi possível iniciar a câmera.");
}

export class CameraService {
  private stream: MediaStream | null = null;

  async start(video: HTMLVideoElement): Promise<MediaStream> {
    this.stop(video);
    if (!navigator.mediaDevices?.getUserMedia) throw new CameraAccessError("unsupported", "A captura de câmera não está disponível neste ambiente.");
    try {
      this.stream = await navigator.mediaDevices.getUserMedia({
        audio: false,
        video: {
          width: { ideal: ENGINEERING_CONFIG.cameraWidth },
          height: { ideal: ENGINEERING_CONFIG.cameraHeight },
          facingMode: "user",
        },
      });
      video.srcObject = this.stream;
      video.muted = true;
      video.playsInline = true;
      await video.play();
      return this.stream;
    } catch (reason) {
      this.stop(video);
      throw cameraError(reason);
    }
  }

  stop(video?: HTMLVideoElement | null): void {
    this.stream?.getTracks().forEach((track) => track.stop());
    this.stream = null;
    if (video) {
      video.pause();
      video.srcObject = null;
    }
  }

  isActive(): boolean {
    return Boolean(this.stream?.getVideoTracks().some((track) => track.readyState === "live"));
  }
}
