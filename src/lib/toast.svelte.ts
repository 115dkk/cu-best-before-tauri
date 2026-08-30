export type ToastKind = "info" | "error";

export const toast = $state<{ message: string; kind: ToastKind; visible: boolean }>({
  message: "",
  kind: "info",
  visible: false,
});

let timer: ReturnType<typeof setTimeout> | undefined;

export function showToast(message: string, kind: ToastKind = "info", ms = 2800): void {
  toast.message = message;
  toast.kind = kind;
  toast.visible = true;
  clearTimeout(timer);
  timer = setTimeout(() => {
    toast.visible = false;
  }, ms);
}
