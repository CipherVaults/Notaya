import { useEffect } from "react";
import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { api, type AiActivityEvent, type AiConfirmEvent, type AiPairingEvent, type AiState, type AiStatus } from "@/lib/ipc";
import { activityUpdate, confirmRequest, pairingRequest, withStatus } from "@/features/ai/contract";

type Display = Pick<AiState, "pairingRequests" | "confirmRequests" | "activity">;
const empty = (): Display => ({ pairingRequests: [], confirmRequests: [], activity: [] });
const useAi = create<Display & { status: AiStatus | null; failed: boolean }>(() => ({ status: null, failed: false, ...empty() }));
let users = 0;
let dispose: (() => void) | undefined;
let generation = 0;
let sequence = 0;

export async function refreshAi() {
  const request = ++sequence;
  const session = generation;
  try {
    const status = await api.aiGetStatus();
    if (users && session === generation && request === sequence) {
      useAi.setState((current) => {
        // Events received before the first snapshot retain their original local
        // deadlines. Unknown status never supplies permission to show a dialog.
        const next = withStatus({ ...current, ...(current.status ?? status) }, status);
        return { status, failed: false, pairingRequests: next.pairingRequests, confirmRequests: next.confirmRequests, activity: next.activity };
      });
    }
  } catch {
    if (users && session === generation && request === sequence) useAi.setState({ status: null, failed: true, ...empty() });
  }
}

export function dismissAiRequest(id: string) {
  useAi.setState((state) => ({
    pairingRequests: state.pairingRequests.filter((r) => r.id !== id),
    confirmRequests: state.confirmRequests.filter((r) => r.id !== id),
  }));
  void refreshAi();
}

function subscribe() {
  if (users++ === 0) {
    const session = ++generation;
    let alive = true;
    const off: (() => void)[] = [];
    let timer: ReturnType<typeof setInterval> | undefined;
    // Events carry requests; status does not. Keep listeners shared across the
    // title bar and Settings so navigating cannot lose a pending dialog.
    const pair = (event: AiPairingEvent) => {
      const request = pairingRequest(event);
      useAi.setState((state) => ({
        pairingRequests: [...state.pairingRequests.filter((r) => r.id !== request.id), request],
      }));
    };
    const confirm = (event: AiConfirmEvent) => {
      const request = confirmRequest(event);
      useAi.setState((state) => ({
        confirmRequests: [...state.confirmRequests.filter((r) => r.id !== request.id), request],
      }));
    };
    const add = async <T,>(event: string, callback: (payload: T) => void) => {
      const unlisten = await listen<T>(event, ({ payload }) => { if (alive) callback(payload); });
      if (alive) off.push(unlisten); else unlisten();
    };
    void (async () => {
      try {
        await add<AiPairingEvent>("ai-pairing", (event) => {
          pair(event);
          void refreshAi();
        });
        await add<AiConfirmEvent>("ai-confirm", (event) => {
          confirm(event);
          void refreshAi();
        });
        await add<AiActivityEvent>("ai-activity", (event) => {
          useAi.setState((state) => ({ activity: activityUpdate(state, event).activity }));
        });
        await add("items-changed", () => void refreshAi());
        await add("ai-connection", () => void refreshAi());
        await add("vault-locked", () => {
          ++sequence;
          useAi.setState({ status: null, failed: false, ...empty() });
        });
        if (!alive) return;
        await refreshAi();
        if (!alive) return;
        // C5 has no status-invalidation or pending-request snapshot command.
        // Poll only display status; never manufacture a permission or answer.
        timer = setInterval(() => void refreshAi(), 1000);
      } catch {
        if (alive && session === generation) useAi.setState({ status: null, failed: true, ...empty() });
      }
    })();
    dispose = () => { alive = false; if (timer) clearInterval(timer); off.forEach((f) => f()); };
  }
  return () => {
    if (--users === 0) {
      ++generation; ++sequence;
      dispose?.(); dispose = undefined;
      useAi.setState({ status: null, failed: false, ...empty() });
    }
  };
}

export function useAiState() {
  const value = useAi();
  useEffect(subscribe, []);
  return { state: value.status ? { ...value.status, pairingRequests: value.pairingRequests, confirmRequests: value.confirmRequests, activity: value.activity } : null,
    failed: value.failed, refresh: refreshAi };
}
