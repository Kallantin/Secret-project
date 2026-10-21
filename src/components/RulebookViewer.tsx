"use client";

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@radix-ui/react-dialog";
import { cn } from "../lib/cn";

type RulebookEntry = {
  filename: string;
  title: string;
  description: string;
};

const RULEBOOKS: RulebookEntry[] = [
  {
    filename: "httpshallofheroeslcgcomwpcontentuploads202007arkhamhorrorlcgrrgpdf.pdf",
    title: "Arkham Horror LCG — Rules Reference Guide",
    description: "Comprehensive rules reference for the Arkham Horror LCG.",
  },
  {
    filename: "httpsimagescdnfantasyflightgamescomfilerpublicca77ca776ab581694e4e9036dd61843b08e1ahb01learntoplaycompressedpdf.pdf",
    title: "Core Set — Learn to Play",
    description: "Introductory rules booklet for the Core Set.",
  },
  {
    filename: "httpsimagescdnfantasyflightgamescomfilerpublic701570156879aefe441e94a01a816872058dahb01rulesreferencecompressedpdf.pdf",
    title: "Core Set — Rules Reference",
    description: "Rules reference for the Core Set campaign.",
  },
  {
    filename: "httpsimagescdnfantasyflightgamescomfilerpublic235e235e1477d981409c89f12034d0f3a8a0ahb04rulespagepdf.pdf",
    title: "The Boundary Beyond — Rules Page",
    description: "Rules for the The Boundary Beyond scenario pack.",
  },
  {
    filename: "httpsimagescdnfantasyflightgamescomfilerpublic9f4a9f4a410bec0145c79b29d33eb24b5d24ahb05rulesbookletv22goodpdf.pdf",
    title: "The Dunwich Legacy — Rules Booklet",
    description: "Rules booklet for The Dunwich Legacy campaign.",
  },
  {
    filename: "httpsimagescdnfantasyflightgamescomfilerpublic885388538d1152744b4aac8ce8d758f71132ahc01learntoplaywebpdf.pdf",
    title: "The Circle Undone — Learn to Play",
    description: "Learn to play booklet for The Circle Undone cycle.",
  },
  {
    filename: "httpsimagescdnfantasyflightgamescomfilerpublic39903990f6aeb14343a2a1ea8aff693addafahbfaq91820goodpdf.pdf",
    title: "Arkham Horror LCG — FAQ",
    description: "Frequently asked questions and official rulings.",
  },
  {
    filename: "httpsimagescdnfantasyflightgamescomfilerpublicec9aec9a0359f2e0419baa0d1506e0b3b1e6recursiveechoesinvestigatorrulespdf.pdf",
    title: "Recursive Echoes — Investigator Rules",
    description: "Investigator rules for the Recursive Echoes scenario.",
  },
  {
    filename: "httpscdn1j1jucommedias8e5695arkhamhorrorthirdeditionsecretsoftheorderrulebookpdf.pdf",
    title: "Arkham Horror Third Edition — Secrets of the Order Rulebook",
    description: "Rulebook for Arkham Horror Third Edition (separate game line).",
  },
];

function RulebookOption({
  entry,
  onOpen,
}: {
  entry: RulebookEntry;
  onOpen: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onOpen}
      className={cn(
        "flex items-center justify-between w-full rounded border p-3 transition-all text-left hover:bg-black/20",
        "border-arkham-border hover:border-gray-600 bg-black/20",
      )}
    >
      <div className="flex-1">
        <h4 className="font-semibold text-sm">{entry.title}</h4>
        <p className="text-[10px] text-gray-400 uppercase tracking-wide mt-1">
          {entry.description}
        </p>
      </div>
      <span className="text-[10px] text-gray-500">Open PDF</span>
    </button>
  );
}

export default function RulebookViewer({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [error, setError] = useState<string | null>(null);

  const handleOpenPdf = async (filename: string) => {
    setError(null);
    try {
      await invoke("open_pdf", { filename });
      onOpenChange(false);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to open PDF. Make sure the game is running via Tauri.",
      );
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <button type="button" className="hidden" />
      </DialogTrigger>
      <DialogContent className="sm:max-w-[560px] max-h-[70vh] overflow-y-auto">
        <div className="px-4 pb-4">
          <DialogTitle>Rulebook &amp; Reference</DialogTitle>
          <DialogDescription>
            Open any rulebook or reference document in your system's PDF viewer.
          </DialogDescription>
        </div>

        {error && (
          <div className="mx-4 mb-3 p-2 rounded bg-red-900/20 border border-red-900/50 text-red-200 text-xs">
            {error}
          </div>
        )}

        <div className="mx-4 flex flex-col gap-2">
          {RULEBOOKS.map((entry) => (
            <RulebookOption
              key={entry.filename}
              entry={entry}
              onOpen={() => handleOpenPdf(entry.filename)}
            />
          ))}
        </div>

        <div className="mx-4 mt-4 pb-4 text-[10px] text-gray-500">
          PDFs open in your system's default PDF viewer. Rulebooks are bundled
          with the game and located in src-tauri/src/rules/.
        </div>
      </DialogContent>
    </Dialog>
  );
}
