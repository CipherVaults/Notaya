import { TrashView } from "@/components/Sidebar";
import { AnimatePresence, motion } from "motion/react";
import { useApp } from "@/store/app";
import Home from "./HomeView";
import PageEditor from "./PageEditor";
import DatabasePage from "@/features/database/DatabasePage";

/** The main area of the Notes view. The notebook tree lives in the sidebar. */
export default function NotesView() {
  const activePageId = useApp((s) => s.activePageId);
  const showTrash = useApp((s) => s.showTrash);
  const activeKind = useApp((s) => s.activeKind);

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={showTrash ? "trash" : (activePageId ?? "home")}
        className="h-full"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.12 }}
      >
        {showTrash ? (
          <TrashView />
        ) : activePageId ? (
          activeKind === "database" ? <DatabasePage databaseId={activePageId} /> : <PageEditor pageId={activePageId} />
        ) : (
          <Home />
        )}
      </motion.div>
    </AnimatePresence>
  );
}
