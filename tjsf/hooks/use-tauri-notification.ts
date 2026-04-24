'use client';

import { useRef, useCallback } from 'react';

const isTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Cache permission across hook instances
let cachedPermission: boolean | null = null;

async function getPermission(): Promise<boolean> {
  if (cachedPermission !== null) return cachedPermission;
  try {
    const { isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification');
    let granted = await isPermissionGranted();
    if (!granted) {
      const result = await requestPermission();
      granted = result === 'granted';
    }
    cachedPermission = granted;
    return granted;
  } catch {
    cachedPermission = false;
    return false;
  }
}

export function useTauriNotification() {
  const permissionRef = useRef<boolean | null>(cachedPermission);

  const notify = useCallback(async (title: string, body: string) => {
    if (!isTauri()) return;
    try {
      if (permissionRef.current === null) {
        permissionRef.current = await getPermission();
      }
      if (!permissionRef.current) return;
      const { sendNotification } = await import('@tauri-apps/plugin-notification');
      sendNotification({ title, body });
    } catch {

    }
  }, []);

  return { notify };
}
