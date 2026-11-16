/** Id do player embutido do Spotify quando está pronto. O App atualiza; as ferramentas do Azriel leem. */
let current: string | null = null;

export const azrielDevice = {
  get: () => current,
  set: (deviceId: string | null) => { current = deviceId; },
};
