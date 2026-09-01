import type { MouseEvent } from "react";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";

export function FloatingOrb() {
  const restoreAzriel = async () => {
    const mainWindow = await Window.getByLabel("main");
    if (!mainWindow) return;
    await mainWindow.unminimize();
    await mainWindow.show();
    await mainWindow.setFocus();
    await getCurrentWindow().hide();
  };

  const startDragging = (event: MouseEvent<HTMLDivElement>) => {
    if (event.button === 0) void getCurrentWindow().startDragging();
  };

  return (
    <div className="floating-orb" aria-label="Azriel minimizado">
      <div className="floating-orb__drag" onMouseDown={startDragging} title="Arraste para mover" />
      <button onClick={() => void restoreAzriel()} aria-label="Abrir Azriel" title="Abrir Azriel">
        <span className="floating-orb__outer"><span className="floating-orb__inner"><i /></span></span>
      </button>
    </div>
  );
}
