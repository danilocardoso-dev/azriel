import { spotifyRepository, type SpotifyAction, type SpotifyPlayer } from "../repositories/spotifyRepository";

export const AZRIEL_DEVICE = "Azriel";

/**
 * O player embutido é a saída padrão: o Azriel a assume sozinho quando o player está pronto,
 * ainda não é o dispositivo ativo e nenhum outro aparelho está tocando (não rouba música alheia).
 */
export function shouldAdoptAzriel(player: SpotifyPlayer | null, loaded: boolean, deviceId: string | null): boolean {
  return loaded && Boolean(deviceId) && player?.device?.name !== AZRIEL_DEVICE && !player?.isPlaying;
}

/**
 * Controla o Spotify. "Tocar" com o player do Azriel pronto vai sempre para ele: se não há dispositivo
 * ativo, ou o ativo é outro (ex.: o app do Spotify pausado), a reprodução é movida para o Azriel e retomada.
 */
export async function spotifyControl(action: SpotifyAction, value: number | undefined, context: { active: boolean; activeDevice?: string | null; deviceId: string | null }) {
  if (action === "play" && context.deviceId && (!context.active || context.activeDevice !== AZRIEL_DEVICE)) return spotifyRepository.transfer(context.deviceId, true);
  return spotifyRepository.control(action, value);
}
