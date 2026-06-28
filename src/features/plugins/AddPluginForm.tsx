import { useState } from "react";

import { Button } from "@/components/ui/Button";
import type { AddPluginInput, ValidateSourceInput } from "@/types/desktop";
import { pickPluginSourcePath } from "@/utils/nativeDesktop";

type AddPluginFormProps = {
  busy: boolean;
  onSubmit: (input: AddPluginInput) => Promise<void>;
  onValidate?: (input: ValidateSourceInput) => Promise<void>;
  validationMessage?: string | null;
};

export function AddPluginForm({ busy, onSubmit, onValidate, validationMessage }: AddPluginFormProps) {
  const [sourceKind, setSourceKind] = useState<"file" | "path" | "git">("file");
  const [location, setLocation] = useState("");
  const [version, setVersion] = useState("");
  const [gitRefKind, setGitRefKind] = useState<"branch" | "tag" | "commit">("branch");
  const [gitRef, setGitRef] = useState("");
  const [gitSubdir, setGitSubdir] = useState("");

  function buildSourceRef(kind: "file" | "path" | "git", nextLocation: string) {
    return `${kind}:${nextLocation.trim()}`;
  }

  function buildInput(kind: "file" | "path" | "git", nextLocation: string): AddPluginInput {
    return {
      pluginRef: buildSourceRef(kind, nextLocation),
      version: version.trim() || undefined,
      gitRefKind: kind === "git" && gitRef.trim() ? gitRefKind : undefined,
      gitRef: kind === "git" ? gitRef.trim() || undefined : undefined,
      gitSubdir: kind === "git" ? gitSubdir.trim() || undefined : undefined,
    };
  }

  function buildValidationInput(kind: "file" | "path" | "git", nextLocation: string): ValidateSourceInput {
    return {
      sourceRef: buildSourceRef(kind, nextLocation),
      gitRefKind: kind === "git" && gitRef.trim() ? gitRefKind : undefined,
      gitRef: kind === "git" ? gitRef.trim() || undefined : undefined,
      gitSubdir: kind === "git" ? gitSubdir.trim() || undefined : undefined,
    };
  }

  function sourceIsReady() {
    if (!location.trim()) {
      return false;
    }

    if (sourceKind !== "git") {
      return true;
    }

    return !gitRef.trim() || Boolean(gitRefKind);
  }

  async function handleSubmit() {
    if (!sourceIsReady()) {
      return;
    }

    await onSubmit(buildInput(sourceKind, location));
    setLocation("");
    setVersion("");
    setGitRef("");
    setGitSubdir("");
  }

  async function handlePick() {
    if (sourceKind === "git") {
      return;
    }
    const selectedPath = await pickPluginSourcePath(sourceKind);
    if (!selectedPath) {
      return;
    }

    setLocation(selectedPath);
    if (onValidate) {
      await onValidate(buildValidationInput(sourceKind, selectedPath));
    }
  }

  return (
    <div className="space-y-3">
      <div className="inline-flex rounded-md border border-zinc-800 bg-zinc-950">
        {(["file", "path", "git"] as const).map((kind) => (
          <button
            key={kind}
            className={`px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] ${
              sourceKind === kind
                ? "bg-teal-500/10 text-teal-100"
                : "text-zinc-500 hover:bg-zinc-900 hover:text-zinc-100"
            }`}
            onClick={() => {
              setSourceKind(kind);
              if (onValidate && location.trim()) {
                void onValidate(buildValidationInput(kind, location));
              }
            }}
            type="button"
          >
            {kind}
          </button>
        ))}
      </div>

      <label className="block">
        <span className="mb-2 block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
          {sourceKind === "git" ? "Repository URL" : "Source location"}
        </span>
        <div className="flex gap-2">
          <input
            className="h-9 min-w-0 flex-1 rounded-md border border-zinc-800 bg-zinc-950 px-3 font-mono text-sm text-zinc-100 outline-none transition focus:border-teal-400"
            onChange={(event) => {
              const nextLocation = event.target.value;
              setLocation(nextLocation);
              if (onValidate && nextLocation.trim()) {
                void onValidate(buildValidationInput(sourceKind, nextLocation));
              }
            }}
            placeholder={
              sourceKind === "file"
                ? "C:\\Downloads\\plugin.rbxm"
                : sourceKind === "path"
                  ? "..\\plugins\\my-tool"
                  : "https://github.com/org/plugin-repo"
            }
            value={location}
          />
          {sourceKind !== "git" ? (
            <Button onClick={() => void handlePick()} type="button" variant="secondary">
              Browse
            </Button>
          ) : null}
        </div>
      </label>

      {sourceKind === "git" ? (
        <div className="grid gap-3 md:grid-cols-[0.8fr_1fr]">
          <label className="block">
            <span className="mb-2 block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Ref type
            </span>
            <select
              className="h-9 w-full rounded-md border border-zinc-800 bg-zinc-950 px-3 text-sm text-zinc-100 outline-none transition focus:border-teal-400"
              onChange={(event) => {
                const nextRefKind = event.target.value as "branch" | "tag" | "commit";
                setGitRefKind(nextRefKind);
                if (onValidate && location.trim()) {
                  void onValidate({
                    sourceRef: buildSourceRef(sourceKind, location),
                    gitRefKind: gitRef.trim() ? nextRefKind : undefined,
                    gitRef: gitRef.trim() || undefined,
                    gitSubdir: gitSubdir.trim() || undefined,
                  });
                }
              }}
              value={gitRefKind}
            >
              <option value="branch">Branch</option>
              <option value="tag">Tag</option>
              <option value="commit">Commit</option>
            </select>
          </label>
          <label className="block">
            <span className="mb-2 block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Ref
            </span>
            <input
              className="h-9 w-full rounded-md border border-zinc-800 bg-zinc-950 px-3 font-mono text-sm text-zinc-100 outline-none transition focus:border-teal-400"
              onChange={(event) => {
                const nextRef = event.target.value;
                setGitRef(nextRef);
                if (onValidate && location.trim()) {
                  void onValidate({
                    sourceRef: buildSourceRef(sourceKind, location),
                    gitRefKind: nextRef.trim() ? gitRefKind : undefined,
                    gitRef: nextRef.trim() || undefined,
                    gitSubdir: gitSubdir.trim() || undefined,
                  });
                }
              }}
              placeholder="Optional"
              value={gitRef}
            />
          </label>
          <label className="block md:col-span-2">
            <span className="mb-2 block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Subdirectory
            </span>
            <input
              className="h-9 w-full rounded-md border border-zinc-800 bg-zinc-950 px-3 font-mono text-sm text-zinc-100 outline-none transition focus:border-teal-400"
              onChange={(event) => {
                const nextSubdir = event.target.value;
                setGitSubdir(nextSubdir);
                if (onValidate && location.trim()) {
                  void onValidate({
                    sourceRef: buildSourceRef(sourceKind, location),
                    gitRefKind: gitRef.trim() ? gitRefKind : undefined,
                    gitRef: gitRef.trim() || undefined,
                    gitSubdir: nextSubdir.trim() || undefined,
                  });
                }
              }}
              placeholder="Optional"
              value={gitSubdir}
            />
          </label>
        </div>
      ) : null}
      {validationMessage ? <p className="text-xs text-amber-300">{validationMessage}</p> : null}

      <label className="block">
        <span className="mb-2 block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
          Version override
        </span>
        <input
          className="h-9 w-full rounded-md border border-zinc-800 bg-zinc-950 px-3 font-mono text-sm text-zinc-100 outline-none transition focus:border-teal-400"
          onChange={(event) => setVersion(event.target.value)}
          placeholder="Optional, e.g. ^1.2.0"
          value={version}
        />
      </label>

      <div className="flex justify-end">
        <Button
          onClick={() => void handleSubmit()}
          variant="primary"
          disabled={busy || !sourceIsReady() || Boolean(validationMessage)}
        >
          Add source
        </Button>
      </div>
    </div>
  );
}
