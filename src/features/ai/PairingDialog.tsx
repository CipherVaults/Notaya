import { useEffect, useState } from "react";
import { Button, Modal } from "@/components/ui";
import { api, type AiPairingRequest } from "@/lib/ipc";

export default function PairingDialog({ request, onAnswered }: {
  request: AiPairingRequest;
  onAnswered: () => void;
}) {
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  const [now, setNow] = useState(Date.now);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  const expired = now >= request.expiresAt;
  const answer = async (allow: boolean) => {
    if (busy || (allow && Date.now() >= request.expiresAt)) return;
    setBusy(true);
    setFailed(false);
    try {
      await api.aiPairingAnswer(request.id, allow);
      onAnswered();
    } catch {
      setFailed(true);
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal open onOpenChange={(open) => { if (!open) void answer(false); }}
      title="Connect an AI?" description="Allow only a connection you started yourself.">
      <p className="break-words text-sm"><bdi>{request.name}</bdi> wants to connect to Notaya. Allow?</p>
      <p className="mt-3 text-sm text-muted">It starts at Look: it can read the notebooks you have not hidden. Change its level or remove it in Settings &gt; AI access.</p>
      {expired && <p role="status" className="mt-3 text-sm text-muted">This request has expired. Deny it and connect again.</p>}
      {failed && <p role="alert" className="mt-3 text-sm text-danger">The answer could not be saved. Try again.</p>}
      <div className="mt-6 flex justify-end gap-2">
        <Button disabled={busy} onClick={() => void answer(false)}>Deny</Button>
        <Button variant="primary" disabled={busy || expired} onClick={() => void answer(true)}>Allow</Button>
      </div>
    </Modal>
  );
}
