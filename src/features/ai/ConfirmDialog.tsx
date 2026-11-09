import { useEffect, useState } from "react";
import { Button, Field, Input, Modal } from "@/components/ui";
import { api, type AiConfirmRequest } from "@/lib/ipc";
import { aiErrorText } from "./contract";

export default function ConfirmDialog({ request, onAnswered }: {
  request: AiConfirmRequest;
  onAnswered: () => void;
}) {
  const [word, setWord] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [now, setNow] = useState(Date.now);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  const expired = now >= request.expiresAt;
  const strong = request.kind === "strong";
  const canAllow = !expired && (!strong || (!!request.word && word === request.word));
  const answer = async (allow: boolean) => {
    if (busy || (allow && (!canAllow || Date.now() >= request.expiresAt))) return;
    setBusy(true);
    setError(null);
    try {
      await api.aiConfirmAnswer(request.id, allow, allow && strong ? word : null);
      onAnswered();
    } catch (error) {
      // An expired/cancelled broker request cannot be answered again.
      if (error === "not_confirmed" || error === "The vault is locked.") onAnswered();
      else setError(aiErrorText(error, "The answer could not be saved. Try again."));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal open onOpenChange={(open) => { if (!open) void answer(false); }}
      title="Allow this AI change?" description="Check the action and the items before allowing it." width={520}>
      <p className="break-words text-sm"><bdi>{request.pairingName}</bdi> wants to {request.summary}</p>
      <ul aria-label="Items affected" className="mt-3 max-h-48 list-disc space-y-1 overflow-auto rounded-lg border border-line py-3 pl-8 pr-4 text-sm">
        {request.items.map((item, i) => <li key={i} className="break-words"><bdi>{item}</bdi></li>)}
      </ul>
      {strong && <div className="mt-4"><Field label={`Type ${request.word ?? "the confirmation word"} to confirm`}>
        <Input aria-label="Confirmation word" value={word} onChange={(e) => setWord(e.target.value)} disabled={busy || expired} />
      </Field></div>}
      <p role="status" className="mt-3 text-xs text-muted">{expired ? "This request has expired. Nothing was allowed." : `Expires in ${Math.ceil((request.expiresAt - now) / 1000)} seconds. Closing this dialog denies the change.`}</p>
      {error && <p role="alert" className="mt-3 text-sm text-danger">{error}</p>}
      <div className="mt-6 flex justify-end gap-2">
        <Button disabled={busy} onClick={() => void answer(false)}>Deny</Button>
        <Button variant="primary" disabled={busy || !canAllow} onClick={() => void answer(true)}>Allow</Button>
      </div>
    </Modal>
  );
}
