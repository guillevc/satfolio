import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";

export type UpdateState =
  | "idle"
  | "checking"
  | "prompt"
  | "downloading"
  | "success"
  | "error";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

class Updater {
  state: UpdateState = $state("idle");
  progress: number | null = $state(0);
  errorMessage: string = $state("");
  currentVersion: string = $state("");
  totalBytes: number = $state(0);

  update: Update | null = $state(null);

  constructor() {
    getVersion()
      .then((v) => (this.currentVersion = v))
      .catch(() => {});
  }

  get version() {
    return this.update?.version ?? "";
  }

  get body() {
    return this.update?.body ?? "";
  }

  get formattedTotalBytes() {
    return this.totalBytes > 0 ? formatBytes(this.totalBytes) : "";
  }

  async check() {
    if (import.meta.env.DEV) return;
    await this.doCheck();
  }

  /** Bypasses the DEV guard — for debug use only. */
  async forceCheck() {
    await this.doCheck();
  }

  private async doCheck() {
    this.state = "checking";
    try {
      const result = await check();
      if (result) {
        this.update = result;
        this.state = "prompt";
      } else {
        this.state = "idle";
      }
    } catch (e) {
      console.error("Update check failed:", e);
      this.state = "idle";
    }
  }

  async startInstall() {
    if (!this.update) return;

    this.state = "downloading";
    this.progress = 0;
    this.totalBytes = 0;

    let downloadedBytes = 0;

    try {
      await this.update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          this.totalBytes = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloadedBytes += event.data.chunkLength;
          if (this.totalBytes > 0) {
            this.progress = Math.min(
              100,
              Math.round((downloadedBytes / this.totalBytes) * 100),
            );
          } else {
            this.progress = null;
          }
        } else if (event.event === "Finished") {
          this.progress = 100;
        }
      });
      this.state = "success";
    } catch (e) {
      this.state = "error";
      this.errorMessage =
        e instanceof Error ? e.message : "Update installation failed";
      console.error("Update install failed:", e);
    }
  }

  async restart() {
    await relaunch();
  }

  dismiss() {
    this.state = "idle";
  }
}

export const updater = new Updater();
