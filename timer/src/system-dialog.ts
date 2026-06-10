export type SystemDialogKind = "info" | "confirm" | "warning";

const TITLE_MAP: Record<SystemDialogKind, string> = {
  info: "提示",
  confirm: "确认",
  warning: "警告",
};

const dialogRoot = document.getElementById("system-dialog") as HTMLDivElement;
const dialogTitle = document.getElementById("system-dialog-title") as HTMLHeadingElement;
const dialogMessage = document.getElementById("system-dialog-message") as HTMLParagraphElement;
const dialogIcon = document.getElementById("system-dialog-icon") as HTMLSpanElement;
const btnOk = document.getElementById("btn-system-dialog-ok") as HTMLButtonElement;
const btnCancel = document.getElementById("btn-system-dialog-cancel") as HTMLButtonElement;

let pendingResolve: ((value: boolean) => void) | null = null;
let dialogMode: "alert" | "confirm" = "alert";

function closeDialog(result: boolean) {
  dialogRoot.classList.add("hidden");
  dialogRoot.classList.remove("is-warning", "is-confirm", "is-info");
  const resolve = pendingResolve;
  pendingResolve = null;
  resolve?.(result);
}

function bindDialogControls() {
  btnOk.addEventListener("click", () => closeDialog(true));
  btnCancel.addEventListener("click", () => closeDialog(false));
  dialogRoot.addEventListener("click", (event) => {
    if (event.target === dialogRoot && dialogMode === "confirm") {
      closeDialog(false);
    }
  });
  window.addEventListener("keydown", (event) => {
    if (dialogRoot.classList.contains("hidden") || !pendingResolve) {
      return;
    }
    if (event.key === "Escape" && dialogMode === "confirm") {
      event.preventDefault();
      closeDialog(false);
    }
    if (event.key === "Enter") {
      event.preventDefault();
      closeDialog(true);
    }
  });
}

function showSystemDialog(kind: SystemDialogKind, message: string, mode: "alert" | "confirm"): Promise<boolean> {
  if (pendingResolve) {
    return Promise.resolve(false);
  }

  dialogTitle.textContent = TITLE_MAP[kind];
  dialogMessage.textContent = message;
  dialogIcon.textContent = kind === "warning" ? "!" : kind === "confirm" ? "?" : "i";
  dialogRoot.classList.remove("hidden", "is-warning", "is-confirm", "is-info");
  dialogRoot.classList.add(
    kind === "warning" ? "is-warning" : kind === "confirm" ? "is-confirm" : "is-info"
  );

  dialogMode = mode;
  const isConfirm = mode === "confirm";
  btnCancel.classList.toggle("hidden", !isConfirm);
  btnOk.textContent = "确定";
  btnCancel.textContent = "取消";

  return new Promise((resolve) => {
    pendingResolve = resolve;
    btnOk.focus();
  });
}

export function showSystemAlert(message: string, kind: SystemDialogKind = "info"): Promise<void> {
  const resolvedKind = kind === "confirm" ? "info" : kind;
  return showSystemDialog(resolvedKind, message, "alert").then(() => undefined);
}

export function showSystemWarning(message: string): Promise<void> {
  return showSystemAlert(message, "warning");
}

export function showSystemConfirm(message: string): Promise<boolean> {
  return showSystemDialog("confirm", message, "confirm");
}

bindDialogControls();
