"use client";

import { useState } from "react";
import type { InvestigatorTemplate } from "../data/investigators";
import { INVESTIGATORS } from "../data/investigators";
import { cn } from "../lib/cn";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@radix-ui/react-dialog";

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex flex-col items-center rounded bg-black/20 px-2 py-1">
      <span className="text-[10px] uppercase tracking-wide text-gray-400">
        {label}
      </span>
      <span className="text-xs font-semibold">{value}</span>
    </div>
  );
}

function InvestigatorOption({
  investigator,
  selected,
  onSelect,
}: {
  investigator: InvestigatorTemplate;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "rounded border p-3 transition-all",
        selected
          ? "border-arkham-accent bg-arkham-accent/10"
          : "border-arkham-border hover:border-gray-600 bg-black/20",
      )}
    >
      <h3 className="font-semibold text-sm">{investigator.name}</h3>
      <p className="text-[10px] text-gray-400 uppercase tracking-wide">
        {investigator.class}
      </p>
      <div className="mt-2 grid grid-cols-4 gap-1">
        <Stat label="Will" value={investigator.willpower} />
        <Stat label="Int" value={investigator.intellect} />
        <Stat label="Com" value={investigator.combat} />
        <Stat label="Agi" value={investigator.agility} />
      </div>
      <div className="mt-2 grid grid-cols-2 gap-1 text-[10px] text-gray-400">
        <span>HP {investigator.max_health}</span>
        <span>SAN {investigator.max_sanity}</span>
      </div>
      {selected ? (
        <span className="mt-2 block rounded bg-arkham-accent px-2 py-0.5 text-[10px] font-medium text-black">
          Selected
        </span>
      ) : null}
    </button>
  );
}

export default function InvestigatorPicker({
  open,
  onOpenChange,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: (investigators: InvestigatorTemplate[]) => void;
}) {
  const [selectedCodes, setSelectedCodes] = useState<Set<string>>(new Set());

  const isSelected = (code: string) => selectedCodes.has(code);
  const toggle = (code: string) => {
    setSelectedCodes((prev) => {
      const next = new Set(prev);
      if (next.has(code)) {
        next.delete(code);
      } else {
        next.add(code);
      }
      return next;
    });
  };

  const selected = INVESTIGATORS.filter((i) => isSelected(i.code));

  const handleConfirm = () => {
    if (selected.length === 0) return;
    onConfirm(selected);
    onOpenChange(false);
    setSelectedCodes(new Set());
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <button type="button" className="hidden" />
      </DialogTrigger>
      <DialogContent className="sm:max-w-[480px]">
        <div className="px-4 pb-4">
          <DialogTitle>Select Investigators</DialogTitle>
          <DialogDescription>
            Choose one or more investigators for the new game.
          </DialogDescription>
        </div>
        <div className="flex flex-col gap-3 p-4 max-h-[60vh] overflow-y-auto">
          {INVESTIGATORS.map((investigator) => (
            <InvestigatorOption
              key={investigator.code}
              investigator={investigator}
              selected={isSelected(investigator.code)}
              onSelect={() => toggle(investigator.code)}
            />
          ))}
        </div>
        <div className="flex justify-end gap-2 px-4 py-3 bg-black/20">
          <button
            type="button"
            onClick={() => onOpenChange(false)}
            className="rounded border border-arkham-border px-4 py-2 text-sm text-gray-300 hover:bg-black/30"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={selected.length === 0}
            onClick={handleConfirm}
            className={cn(
              "rounded px-4 py-2 text-sm font-medium transition-all disabled:opacity-50 disabled:pointer-events-none",
              selected.length > 0
                ? "bg-arkham-accent text-black hover:bg-arkham-accent/90"
                : "border border-arkham-border text-gray-400",
            )}
          >
            Start Game ({selected.length})
          </button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
