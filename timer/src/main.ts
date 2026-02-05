import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// DOM 元素
const timerDisplay = document.getElementById("timer-display") as HTMLDivElement;
const timerStatus = document.getElementById("timer-status") as HTMLDivElement;
const progressFill = document.getElementById("progress-fill") as HTMLDivElement;
const statusText = document.getElementById("status-text") as HTMLSpanElement;

const btnStart = document.getElementById("btn-start") as HTMLButtonElement;
const btnPause = document.getElementById("btn-pause") as HTMLButtonElement;
const btnStop = document.getElementById("btn-stop") as HTMLButtonElement;
const btnSetCustom = document.getElementById("btn-set-custom") as HTMLButtonElement;
const customMinutes = document.getElementById("custom-minutes") as HTMLInputElement;

// 状态
let currentState = "Idle";
let totalSeconds = 1800;

// 格式化时间显示
function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

// 更新UI
function updateUI(runtime: any) {
  currentState = runtime.state;
  totalSeconds = runtime.total_seconds;

  // 更新时间显示
  timerDisplay.textContent = formatTime(runtime.remaining_seconds);

  // 更新进度条
  const progress = totalSeconds > 0 ? (runtime.remaining_seconds / totalSeconds) * 100 : 100;
  progressFill.style.width = `${progress}%`;

  // 更新状态文本
  const stateMap: Record<string, string> = {
    Idle: "准备就绪",
    Running: "计时中",
    Paused: "已暂停",
  };
  timerStatus.textContent = stateMap[runtime.state] || runtime.state;
  statusText.textContent = `状态: ${stateMap[runtime.state] || runtime.state}`;

  // 更新按钮状态
  updateButtonStates(runtime.state);
}

// 更新按钮状态
function updateButtonStates(state: string) {
  switch (state) {
    case "Idle":
      btnStart.disabled = false;
      btnStart.textContent = "开始";
      btnPause.disabled = true;
      btnPause.textContent = "暂停";
      break;
    case "Running":
      btnStart.disabled = true;
      btnPause.disabled = false;
      btnPause.textContent = "暂停";
      break;
    case "Paused":
      btnStart.disabled = false;
      btnStart.textContent = "继续";
      btnPause.disabled = true;
      btnPause.textContent = "暂停";
      break;
  }
}

// 获取初始状态
async function init() {
  try {
    const runtime = await invoke("get_timer_runtime") as any;
    updateUI(runtime);
  } catch (e) {
    console.error("获取计时器状态失败:", e);
  }
}

// 监听计时器更新事件
listen("timer-update", (event: any) => {
  updateUI(event.payload);
});

// 开始/继续
btnStart.addEventListener("click", async () => {
  try {
    if (currentState === "Paused") {
      await invoke("resume_timer");
    } else {
      await invoke("start_timer");
    }
  } catch (e) {
    console.error("启动失败:", e);
    alert("启动失败: " + e);
  }
});

// 暂停
btnPause.addEventListener("click", async () => {
  try {
    await invoke("pause_timer");
  } catch (e) {
    console.error("暂停失败:", e);
  }
});

// 停止/重置
btnStop.addEventListener("click", async () => {
  try {
    await invoke("stop_timer");
  } catch (e) {
    console.error("停止失败:", e);
  }
});

// 快速设置
document.querySelectorAll(".btn-quick").forEach((btn) => {
  btn.addEventListener("click", async (e) => {
    const minutes = parseInt((e.target as HTMLElement).dataset.minutes || "30");
    try {
      await invoke("set_timer_interval", { minutes });
      await invoke("stop_timer");
      // 更新显示
      const runtime = await invoke("get_timer_runtime") as any;
      updateUI(runtime);
    } catch (err) {
      alert("设置失败: " + err);
    }
  });
});

// 自定义设置
btnSetCustom.addEventListener("click", async () => {
  const minutes = parseInt(customMinutes.value);
  if (minutes < 1 || minutes > 1440) {
    alert("请输入 1-1440 之间的数字");
    return;
  }
  try {
    await invoke("set_timer_interval", { minutes });
    await invoke("stop_timer");
    const runtime = await invoke("get_timer_runtime") as any;
    updateUI(runtime);
  } catch (err) {
    alert("设置失败: " + err);
  }
});

// 监听托盘事件
listen("tray-pause", () => btnPause.click());
listen("tray-resume", () => btnStart.click());
listen("tray-stop", () => btnStop.click());
listen("tray-quick-set", (event: any) => {
  const minutes = event.payload as number;
  customMinutes.value = minutes.toString();
  btnSetCustom.click();
});

// 初始化
document.addEventListener("DOMContentLoaded", init);
