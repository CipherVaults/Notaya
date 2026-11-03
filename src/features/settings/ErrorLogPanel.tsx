import { useEffect, useState } from "react";
import { Copy, FolderOpen } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { LogInfo } from "@/lib/types";
import { Button } from "@/components/ui";

/** "12 KB" for the size of the log files. */
function sizeText(bytes: number): string {
  if (bytes <= 0) return "empty";
  if (bytes < 1024) return `${bytes} bytes`;
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** "2026-10-29 07:45 UTC" for an ISO time stamp from the log. */
function lastText(iso: string | null): string {
  if (!iso) return "no entries yet";
  const m = /^(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2})/.exec(iso);
  return m ? `${m[1]} ${m[2]} UTC` : iso;
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    // The older way, if the page is not allowed to use the clipboard API.
    const ta = document.createElement("textarea");
    ta.value = text;
    document.body.append(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }
}

/** The body of the "Error log" section of Settings. */
export default function ErrorLogPanel() {
  const [info, setInfo] = useState<LogInfo | null>(null);

  useEffect(() => {
    let alive = true;
    api.logInfo().then((i) => alive && setInfo(i), () => {});
    return () => {
      alive = false;
    };
  }, []);

  return (
    <div className="pb-2">
      <p className="pb-3 text-sm leading-relaxed text-muted">
        Notaya writes problems and crashes to a small text file on this computer. It never contains your notes or passwords and is never
        sent anywhere. If something goes wrong, you can send this file to whoever helps you.
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={async () => {
            try {
              await api.openLogFolder();
            } catch (e) {
              toast.error(errorText(e));
            }
          }}
        >
          <FolderOpen size={14} /> Open log folder
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={!info}
          onClick={async () => {
            if (!info) return;
            try {
              await copyText(info.folder);
              toast.success("Log location copied.");
            } catch {
              toast.error("Could not copy the location.");
            }
          }}
        >
          <Copy size={14} /> Copy log location
        </Button>
      </div>
      {info && (
        <dl className="mt-3 grid grid-cols-[160px_1fr] gap-y-1.5 text-sm">
          <dt className="text-faint">Location</dt>
          <dd className="selectable break-all font-mono text-xs">{info.folder}</dd>
          <dt className="text-faint">Size</dt>
          <dd>{sizeText(info.bytes)}</dd>
          <dt className="text-faint">Last entry</dt>
          <dd>{lastText(info.lastEntry)}</dd>
        </dl>
      )}
    </div>
  );
}
