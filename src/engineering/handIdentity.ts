import { HAND_LANDMARK_INDEX } from "./config";
import { landmarkDistance } from "./trackingMath";
import type { HandLandmark, HandSide, TrackedHand } from "./types";

export interface RawTrackedHand {
  landmarks: HandLandmark[];
  handedness?: HandSide;
  confidence?: number;
}

export type PreviousHandPositions = Partial<Record<HandSide, HandLandmark>>;

function preferredFallbackSide(hand: RawTrackedHand): HandSide {
  return hand.landmarks[HAND_LANDMARK_INDEX.wrist]?.x <= 0.5 ? "right" : "left";
}

export function stabilizeHandIdentity(hands: RawTrackedHand[], previous: PreviousHandPositions = {}): TrackedHand[] {
  const selected = new Map<HandSide, RawTrackedHand>();
  const remaining: RawTrackedHand[] = [];

  for (const hand of hands) {
    if (!hand.handedness) {
      remaining.push(hand);
      continue;
    }
    const current = selected.get(hand.handedness);
    if (!current || (hand.confidence ?? 0) > (current.confidence ?? 0)) {
      if (current) remaining.push(current);
      selected.set(hand.handedness, hand);
    } else {
      remaining.push(hand);
    }
  }

  for (const hand of remaining) {
    const available = (["left", "right"] as const).filter((side) => !selected.has(side));
    if (!available.length) break;
    const wrist = hand.landmarks[HAND_LANDMARK_INDEX.wrist];
    const side = available.reduce<HandSide>((best, candidate) => {
      const candidatePrevious = previous[candidate];
      const bestPrevious = previous[best];
      if (!candidatePrevious) return bestPrevious ? best : preferredFallbackSide(hand);
      if (!bestPrevious) return candidate;
      return landmarkDistance(wrist, candidatePrevious) < landmarkDistance(wrist, bestPrevious) ? candidate : best;
    }, available.includes(preferredFallbackSide(hand)) ? preferredFallbackSide(hand) : available[0]);
    selected.set(side, hand);
  }

  return (["left", "right"] as const).flatMap((side) => {
    const hand = selected.get(side);
    return hand ? [{ ...hand, id: side, handedness: side }] : [];
  });
}

export function handPositions(hands: TrackedHand[]): PreviousHandPositions {
  return Object.fromEntries(hands.map((hand) => [hand.id, hand.landmarks[HAND_LANDMARK_INDEX.wrist]]));
}
