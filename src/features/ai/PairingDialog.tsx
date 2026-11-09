import { useEffect, useState } from "react";
import { Button, Modal } from "@/components/ui";
import { api, type AiPairingRequest } from "@/lib/ipc";
import { aiErrorText } from "./contract";

export default function PairingDialog({ request, alreadyPaired, onAnswered }: {
  request: AiPairingRequest;
  alreadyPaired: boolean;
  onAnswered: () => void;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [now, setNow] = useState(Date.now);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  const expired = now >= request.expiresAt;
  const answer = async (allow: boolean) => {
    if (busy || (allow && (alreadyPaired || Date.now() >= request.expiresAt))) return;
    setBusy(true);
    setError(null);
    try {
      await api.aiPairingAnswer(request.id, allow);
      onAnswered();
    } catch (error) {
      if (error === "cancelled" || error === "pairing_refused" || error === "The vault is locked.") onAnswered();
      else setError(aiErrorText(error, "The answer could not be saved. Try again."));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal open onOpenChange={(open) => { if (!open) void answer(false); }}
      title="Connect an AI?" description="Allow only a connection you started yourself.">
      {alreadyPaired ? <p role="alert" className="text-sm text-danger">This connection is already paired. Use its existing connection, or remove its pairing in Settings before connecting again.</p>
        : <><p className="break-words text-sm"><bdi>{request.name}</bdi> wants to connect to Notaya. Allow?</p>
          <p className="mt-3 text-sm text-muted">It starts at Look: it can read the notebooks you have not hidden. Change its level or remove it in Settings &gt; AI access.</p></>}
      {expired && <p role="status" className="mt-3 text-sm text-muted">This request has expired. Deny it and connect again.</p>}
      {error && <p role="alert" className="mt-3 text-sm text-danger">{error}</p>}
      <div className="mt-6 flex justify-end gap-2">
        <Button disabled={busy} onClick={() => void answer(false)}>Deny</Button>
        <Button variant="primary" disabled={busy || expired || alreadyPaired} onClick={() => void answer(true)}>Allow</Button>
      </div>
    </Modal>
  );
}
