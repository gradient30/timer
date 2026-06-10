import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

// DOM 元素
const timerDisplay = document.getElementById("timer-display") as HTMLDivElement;
const timerStatus = document.getElementById("timer-status") as HTMLDivElement;
const progressFill = document.getElementById("progress-fill") as HTMLDivElement;
const statusText = document.getElementById("status-text") as HTMLSpanElement;
const timerDisplayContainer = document.querySelector(".timer-display") as HTMLElement;

const btnStart = document.getElementById("btn-start") as HTMLButtonElement;
const btnPause = document.getElementById("btn-pause") as HTMLButtonElement;
const btnStop = document.getElementById("btn-stop") as HTMLButtonElement;
const btnSetCustom = document.getElementById("btn-set-custom") as HTMLButtonElement;
const customMinutes = document.getElementById("custom-minutes") as HTMLInputElement;

const noticeModal = document.getElementById("notice-modal") as HTMLDivElement;
const noticeTitle = document.getElementById("notice-title") as HTMLHeadingElement;
const noticeMessage = document.getElementById("notice-message") as HTMLParagraphElement;
const noticeCountdown = document.getElementById("notice-countdown") as HTMLSpanElement;
const noticeDelayOptions = document.getElementById("notice-delay-options") as HTMLDivElement;
const btnNoticeExecute = document.getElementById("btn-notice-execute") as HTMLButtonElement;
const btnNoticeCancel = document.getElementById("btn-notice-cancel") as HTMLButtonElement;
const trayHint = document.getElementById("tray-hint") as HTMLSpanElement;

const activationModal = document.getElementById("activation-modal") as HTMLDivElement;
const activationCodeInput = document.getElementById("activation-code") as HTMLInputElement;
const activationError = document.getElementById("activation-error") as HTMLParagraphElement;
const btnActivate = document.getElementById("btn-activate") as HTMLButtonElement;

const activationGenModal = document.getElementById("activation-gen-modal") as HTMLDivElement;
const activationGenPassword = document.getElementById("activation-gen-password") as HTMLInputElement;
const activationGenError = document.getElementById("activation-gen-error") as HTMLParagraphElement;
const activationGenResult = document.getElementById("activation-gen-result") as HTMLTextAreaElement;
const btnGenerateActivation = document.getElementById("btn-generate-activation") as HTMLButtonElement;
const btnCloseActivationGen = document.getElementById("btn-close-activation-gen") as HTMLButtonElement;

const passwordSetupModal = document.getElementById("password-setup-modal") as HTMLDivElement;
const exitPasswordModal = document.getElementById("exit-password-modal") as HTMLDivElement;
const resetPasswordModal = document.getElementById("reset-password-modal") as HTMLDivElement;

const setupPasswordInput = document.getElementById("setup-password") as HTMLInputElement;
const setupPasswordConfirm = document.getElementById("setup-password-confirm") as HTMLInputElement;
const setupQuestionSelect = document.getElementById("setup-question") as HTMLSelectElement;
const setupAnswerInput = document.getElementById("setup-answer") as HTMLInputElement;
const setupError = document.getElementById("setup-error") as HTMLParagraphElement;
const btnSavePassword = document.getElementById("btn-save-password") as HTMLButtonElement;
const btnCloseSetup = document.getElementById("btn-close-setup") as HTMLButtonElement;

const exitPasswordInput = document.getElementById("exit-password") as HTMLInputElement;
const exitLockInfo = document.getElementById("exit-lock-info") as HTMLParagraphElement;
const exitError = document.getElementById("exit-error") as HTMLParagraphElement;
const btnExitConfirm = document.getElementById("btn-exit-confirm") as HTMLButtonElement;
const btnExitCancel = document.getElementById("btn-exit-cancel") as HTMLButtonElement;
const btnCloseExit = document.getElementById("btn-close-exit") as HTMLButtonElement;
const btnForgotPassword = document.getElementById("btn-forgot-password") as HTMLButtonElement;

const resetQuestion = document.getElementById("reset-question") as HTMLDivElement;
const resetAnswerInput = document.getElementById("reset-answer") as HTMLInputElement;
const resetPasswordInput = document.getElementById("reset-password") as HTMLInputElement;
const resetPasswordConfirm = document.getElementById("reset-password-confirm") as HTMLInputElement;
const resetError = document.getElementById("reset-error") as HTMLParagraphElement;
const btnResetPassword = document.getElementById("btn-reset-password") as HTMLButtonElement;
const btnResetCancel = document.getElementById("btn-reset-cancel") as HTMLButtonElement;
const btnCloseReset = document.getElementById("btn-close-reset") as HTMLButtonElement;
const appContainer = document.querySelector(".app") as HTMLDivElement;

const appWindow = getCurrentWindow();

// 状态
let currentState = "Idle";
let totalSeconds = 1800;
let statusOverride: { message: string; expiresAt: number } | null = null;
let noticeCountdownTimer: number | null = null;
let noticePending = false;
let setupMandatory = false;
let appInitialized = false;
let isScheduleEffective = true;
let currentActionType = "lock";
let loopEnabled = true;
let loopIntervalMinutes = 5;
let enforceRelockDuringRest = true;
let loopPausedBySchedule = false;
let latestRuntime: any = null;
let scheduleRefreshInFlight = false;
let lastScheduleRefreshAt = 0;
let trayHintClickCount = 0;
let trayHintClickTimer: number | null = null;
let activationStatus = {
  activated: false,
  admin_enabled: false,
};
let securityStatus = {
  password_set: false,
  lock_remaining_seconds: 0,
  remaining_attempts: 3,
  max_attempts: 3,
  security_question: "",
  safe_mode: false,
};

const stateMap: Record<string, string> = {
  Idle: "准备就绪",
  Running: "计时中",
  Paused: "已暂停",
};

// 格式化时间显示
function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

function normalizeActionType(actionType: string | undefined): string {
  return actionType === "shutdown" ? "shutdown" : "lock";
}

function actionTypeDisplay(actionType: string): string {
  return actionType === "shutdown" ? "关机" : "锁屏";
}

function renderTimerLabel(runtime: any) {
  if (!timerStatus) return;

  timerStatus.classList.remove("warning");

  if (runtime.state === "Running") {
    if ((runtime.phase || "Work") === "Rest") {
      timerStatus.textContent = `休息中，${formatLockCountdown(runtime.remaining_seconds)}后开始自动循环事件`;
      return;
    }
    if (!isScheduleEffective) {
      timerStatus.textContent = "当前不在生效周期（到点不执行）";
      timerStatus.classList.add("warning");
      return;
    }
    timerStatus.textContent = `${formatTime(runtime.remaining_seconds)}后执行：${actionTypeDisplay(currentActionType)}`;
    return;
  }

  if (loopPausedBySchedule && !isScheduleEffective) {
    timerStatus.textContent = "当前不在生效周期（自动循环已暂停）";
    timerStatus.classList.add("warning");
    return;
  }

  timerStatus.textContent = stateMap[runtime.state] || runtime.state;
}

// 更新UI
function updateUI(runtime: any) {
  latestRuntime = runtime;
  currentState = runtime.state;
  totalSeconds = runtime.total_seconds;

  // 更新时间显示
  timerDisplay.textContent = formatTime(runtime.remaining_seconds);

  // 更新进度条
  const progress = totalSeconds > 0 ? (runtime.remaining_seconds / totalSeconds) * 100 : 100;
  progressFill.style.width = `${progress}%`;

  // 更新圆环内提示
  renderTimerLabel(runtime);
  renderStatusText(runtime.state);

  // 更新按钮状态
  updateButtonStates(runtime.state);
}

function renderStatusText(state: string) {
  if (statusOverride && Date.now() < statusOverride.expiresAt) {
    statusText.textContent = statusOverride.message;
    return;
  }
  statusOverride = null;
  statusText.textContent = `状态: ${stateMap[state] || state}`;
}

function formatLockCountdown(seconds: number) {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  if (mins > 0) {
    return `${mins}分${secs}秒`;
  }
  return `${secs}秒`;
}

function formatActivationCode(value: string) {
  const cleaned = value.replace(/[^0-9a-zA-Z]/g, "").toUpperCase().slice(0, 16);
  const chunks = [] as string[];
  for (let i = 0; i < cleaned.length; i += 4) {
    chunks.push(cleaned.slice(i, i + 4));
  }
  return chunks.join("-");
}

function showActivationModal() {
  activationError.textContent = "";
  activationCodeInput.value = "";
  activationModal.classList.remove("hidden");
  appContainer.classList.add("blocked");
  activationCodeInput.focus();
}

function hideActivationModal() {
  activationModal.classList.add("hidden");
  appContainer.classList.remove("blocked");
}

function resetTrayHintClicks() {
  trayHintClickCount = 0;
  if (trayHintClickTimer) {
    window.clearTimeout(trayHintClickTimer);
    trayHintClickTimer = null;
  }
}

function showActivationGenerator() {
  activationGenError.textContent = "";
  activationGenPassword.value = "";
  activationGenResult.value = "";
  activationGenModal.classList.remove("hidden");
}

function hideActivationGenerator() {
  activationGenModal.classList.add("hidden");
}

async function refreshActivationStatus() {
  const status = await invoke("get_activation_status") as any;
  activationStatus = {
    activated: !!status.activated,
    admin_enabled: !!status.admin_enabled,
  };
}

async function refreshActionType() {
  try {
    const config = await invoke("get_config") as any;
    currentActionType = normalizeActionType(config.action?.action_type);
    loopEnabled = config.timer?.loop_enabled ?? true;
    loopIntervalMinutes = config.timer?.loop_interval_minutes ?? 5;
    enforceRelockDuringRest = config.timer?.enforce_relock_during_rest ?? true;
    if (latestRuntime) {
      renderTimerLabel(latestRuntime);
    }
  } catch (e) {
    console.error("获取执行动作配置失败:", e);
  }
}

function clearAuthErrors() {
  setupError.textContent = "";
  exitError.textContent = "";
  resetError.textContent = "";
}

function showPasswordSetup(mandatory: boolean) {
  setupMandatory = mandatory;
  clearAuthErrors();
  setupPasswordInput.value = "";
  setupPasswordConfirm.value = "";
  setupQuestionSelect.value = "";
  setupAnswerInput.value = "";
  if (btnCloseSetup) {
    btnCloseSetup.style.display = mandatory ? "none" : "inline-flex";
  }
  passwordSetupModal?.classList.remove("hidden");
}

function hidePasswordSetup() {
  if (!setupMandatory) {
    passwordSetupModal?.classList.add("hidden");
  }
}

function showExitModal() {
  clearAuthErrors();
  exitPasswordInput.value = "";
  exitLockInfo.textContent = "";
  if (securityStatus.lock_remaining_seconds > 0) {
    exitLockInfo.textContent = `密码错误已锁定，请在 ${formatLockCountdown(securityStatus.lock_remaining_seconds)} 后再试`;
    btnExitConfirm.disabled = true;
  } else {
    exitLockInfo.textContent = `剩余尝试次数: ${securityStatus.remaining_attempts}/${securityStatus.max_attempts}`;
    btnExitConfirm.disabled = false;
  }
  exitPasswordModal?.classList.remove("hidden");
  exitPasswordInput.focus();
}

function hideExitModal() {
  exitPasswordModal?.classList.add("hidden");
}

function showResetModal() {
  clearAuthErrors();
  resetAnswerInput.value = "";
  resetPasswordInput.value = "";
  resetPasswordConfirm.value = "";
  resetQuestion.textContent = securityStatus.security_question || "未设置密保问题";
  resetPasswordModal?.classList.remove("hidden");
}

function hideResetModal() {
  resetPasswordModal?.classList.add("hidden");
}

function clearNoticeCountdown() {
  if (noticeCountdownTimer !== null) {
    window.clearInterval(noticeCountdownTimer);
    noticeCountdownTimer = null;
  }
}

function clearStatusOverride() {
  statusOverride = null;
  renderStatusText(currentState);
}

function hideNoticeModal() {
  clearNoticeCountdown();
  noticePending = false;
  clearStatusOverride();
  noticeModal?.classList.add("hidden");
  invoke("set_window_topmost", { enabled: false }).catch((e) => {
    console.error("取消置顶失败:", e);
  });
}

function showNoticeModal(payload: any) {
  if (!noticeModal) return;

  noticePending = true;
  invoke("set_window_topmost", { enabled: true }).catch((e) => {
    console.error("设置置顶失败:", e);
  });
  noticeTitle.textContent = payload.title || "即将执行";
  noticeMessage.textContent = payload.message || "系统即将执行操作";
  noticeCountdown.textContent = String(payload.countdown_seconds ?? 0);

  noticeDelayOptions.innerHTML = "";
  const delayOptions: number[] = payload.delay_options || [];
  const maxDelayTimes = payload.max_delay_times ?? 3;
  const delayCount = payload.delay_count ?? 0;
  const canPostpone = delayCount < maxDelayTimes;
  if (!canPostpone) {
    noticeMessage.textContent = `${payload.message || "系统即将执行操作"}（已达到当日取消上限）`;
  }
  delayOptions.forEach((minutes) => {
    const btn = document.createElement("button");
    btn.className = "btn btn-secondary btn-small";
    btn.textContent = `延后${minutes}分钟`;
    btn.disabled = !canPostpone;
    btn.addEventListener("click", async () => {
      try {
        const ok = await invoke("delay_execution", {
          minutes,
          delayCount,
          maxDelayTimes,
        }) as boolean;
        if (!ok) {
          alert("已用完所有延后机会");
          return;
        }
        hideNoticeModal();
        await invoke("resume_timer");
        try {
          await appWindow.hide();
        } catch (err) {
          console.error("最小化失败:", err);
        }
        const runtime = await invoke("get_timer_runtime") as any;
        updateUI(runtime);
        await refreshScheduleIndicator();
      } catch (e) {
        console.error("延后失败:", e);
        alert("延后失败: " + e);
      }
    });
    noticeDelayOptions.appendChild(btn);
  });

  btnNoticeExecute.onclick = async () => {
    try {
      await invoke("confirm_execution");
      hideNoticeModal();
      await maybeStartLoopRestCycle(currentActionType === "lock" && enforceRelockDuringRest);
      const runtime = await invoke("get_timer_runtime") as any;
      updateUI(runtime);
      await refreshScheduleIndicator();
    } catch (e) {
      console.error("立即执行失败:", e);
    }
  };

  btnNoticeCancel.onclick = async () => {
    try {
      await invoke("cancel_execution");
      hideNoticeModal();
      await maybeStartLoopRestCycle(false);
      const runtime = await invoke("get_timer_runtime") as any;
      updateUI(runtime);
      await refreshScheduleIndicator();
    } catch (e) {
      console.error("取消执行失败:", e);
      alert(`取消失败: ${e}`);
    }
  };
  btnNoticeCancel.disabled = !canPostpone;
  btnNoticeCancel.textContent = canPostpone ? "取消" : "取消（当日已达上限）";

  clearNoticeCountdown();
  let remaining = payload.countdown_seconds ?? 0;
  noticeCountdownTimer = window.setInterval(() => {
    remaining -= 1;
    if (remaining <= 0) {
      noticeCountdown.textContent = "0";
      if (noticePending) {
        invoke("confirm_execution")
          .then(async () => {
            hideNoticeModal();
            await maybeStartLoopRestCycle(currentActionType === "lock" && enforceRelockDuringRest);
          })
          .catch((e) => console.error("自动执行失败:", e));
      }
      return;
    }
    noticeCountdown.textContent = String(remaining);
  }, 1000);

  noticeModal.classList.remove("hidden");
}

function setScheduleIndicator(isEffective: boolean) {
  isScheduleEffective = isEffective;
  if (isEffective) {
    loopPausedBySchedule = false;
  }
  if (!timerDisplayContainer) return;
  if (isEffective) {
    timerDisplayContainer.classList.remove("schedule-inactive");
  } else {
    timerDisplayContainer.classList.add("schedule-inactive");
  }
  if (latestRuntime) {
    renderTimerLabel(latestRuntime);
  }
}

async function refreshScheduleIndicator(useThrottle = false) {
  const now = Date.now();
  if (useThrottle && now - lastScheduleRefreshAt < 2000) {
    return;
  }
  if (scheduleRefreshInFlight) {
    return;
  }
  scheduleRefreshInFlight = true;
  try {
    const isEffective = await invoke("check_schedule_effective") as boolean;
    setScheduleIndicator(isEffective);
    lastScheduleRefreshAt = Date.now();
  } catch (e) {
    console.error("检查生效规则失败:", e);
  } finally {
    scheduleRefreshInFlight = false;
  }
}

function showLoopPausedWarning() {
  loopPausedBySchedule = true;
  statusOverride = {
    message: "当前不在生效周期，自动循环已暂停",
    expiresAt: Date.now() + 6000,
  };
  renderStatusText(currentState);
  if (latestRuntime) {
    renderTimerLabel(latestRuntime);
  }
}

async function maybeStartLoopRestCycle(enforceLock = false) {
  if (!loopEnabled) {
    return;
  }

  try {
    const effective = await invoke("check_schedule_effective") as boolean;
    setScheduleIndicator(effective);
    if (!effective) {
      showLoopPausedWarning();
      return;
    }

    await invoke("start_loop_rest_timer", { minutes: loopIntervalMinutes, enforceLock });
    const runtime = await invoke("get_timer_runtime") as any;
    updateUI(runtime);
  } catch (e) {
    console.error("启动循环休息阶段失败:", e);
  }
}

async function maybeStartNextWorkCycle() {
  if (!loopEnabled) {
    return;
  }

  try {
    const effective = await invoke("check_schedule_effective") as boolean;
    setScheduleIndicator(effective);
    if (!effective) {
      showLoopPausedWarning();
      return;
    }

    await invoke("start_work_cycle_timer");
    const runtime = await invoke("get_timer_runtime") as any;
    updateUI(runtime);
  } catch (e) {
    console.error("启动下一轮工作阶段失败:", e);
  }
}

trayHint?.addEventListener("click", () => {
  if (!activationStatus.admin_enabled) {
    return;
  }

  trayHintClickCount += 1;
  if (trayHintClickTimer) {
    window.clearTimeout(trayHintClickTimer);
  }
  trayHintClickTimer = window.setTimeout(() => {
    resetTrayHintClicks();
  }, 5000);

  if (trayHintClickCount >= 10) {
    resetTrayHintClicks();
    showActivationGenerator();
  }
});

activationCodeInput?.addEventListener("input", () => {
  activationCodeInput.value = formatActivationCode(activationCodeInput.value);
});

activationCodeInput?.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    btnActivate?.click();
  }
});

btnActivate?.addEventListener("click", async () => {
  const code = activationCodeInput.value.trim();
  if (!code) {
    activationError.textContent = "请输入激活码";
    return;
  }
  activationError.textContent = "";
  btnActivate.disabled = true;
  try {
    const result = await invoke("activate_with_code", { code }) as any;
    activationStatus.activated = !!result?.activated;
    hideActivationModal();
    await initAfterActivation();
  } catch (e) {
    activationError.textContent = `激活失败: ${e}`;
  } finally {
    btnActivate.disabled = false;
  }
});

btnGenerateActivation?.addEventListener("click", async () => {
  const password = activationGenPassword.value.trim();
  activationGenError.textContent = "";
  activationGenResult.value = "";
  if (!password) {
    activationGenError.textContent = "请输入口令";
    return;
  }

  btnGenerateActivation.disabled = true;
  try {
    const codes = await invoke("generate_activation_codes", { password }) as string[];
    activationGenResult.value = codes.join("\n");
  } catch (e) {
    activationGenError.textContent = `生成失败: ${e}`;
  } finally {
    btnGenerateActivation.disabled = false;
  }
});

btnCloseActivationGen?.addEventListener("click", () => {
  hideActivationGenerator();
});

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

async function syncIntervalFromConfig() {
  try {
    const config = await invoke("get_config") as any;
    const minutes = config.timer?.interval_minutes ?? 30;
    customMinutes.value = String(minutes);
    loopEnabled = config.timer?.loop_enabled ?? true;
    loopIntervalMinutes = config.timer?.loop_interval_minutes ?? 5;
    enforceRelockDuringRest = config.timer?.enforce_relock_during_rest ?? true;
    currentActionType = normalizeActionType(config.action?.action_type);
  } catch (e) {
    console.error("同步配置间隔失败:", e);
  }
}

async function initAfterActivation() {
  if (appInitialized) {
    return;
  }
  appInitialized = true;
  await syncIntervalFromConfig();
  await refreshActionType();
  const runtime = await invoke("get_timer_runtime") as any;
  updateUI(runtime);
  await refreshScheduleIndicator();
  await refreshSecurityStatus();
  if (!securityStatus.password_set && !securityStatus.safe_mode) {
    showPasswordSetup(true);
  }
}

// 获取初始状态
async function init() {
  try {
    await refreshActivationStatus();
    if (!activationStatus.activated) {
      showActivationModal();
      return;
    }
    await initAfterActivation();
  } catch (e) {
    console.error("初始化失败:", e);
  }
}

// 监听计时器更新事件
listen("timer-update", (event: any) => {
  updateUI(event.payload);
  void refreshScheduleIndicator(true);
});

// 开始/继续
btnStart.addEventListener("click", async () => {
  try {
    loopPausedBySchedule = false;
    await refreshScheduleIndicator();
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
    await refreshScheduleIndicator();
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

// 监听计时完成事件，保存状态
listen("timer-finished", async () => {
  try {
    await invoke("save_timer_finished_state");
    await maybeStartLoopRestCycle(currentActionType === "lock" && enforceRelockDuringRest);
  } catch (e) {
    console.error("保存计时完成状态失败:", e);
  }
});

listen("loop-rest-finished", async () => {
  await maybeStartNextWorkCycle();
});

// 监听提前通知事件
listen("timer-notify", async (event: any) => {
  const payload = event.payload || {};
  currentActionType = normalizeActionType(payload.action_type || currentActionType);
  if (latestRuntime) {
    renderTimerLabel(latestRuntime);
  }
  const message = payload.message || "即将执行操作";
  const countdownSeconds = payload.countdown_seconds ?? 0;
  const durationMs = Math.max(5000, countdownSeconds * 1000);
  statusOverride = {
    message,
    expiresAt: Date.now() + durationMs,
  };
  renderStatusText(currentState);

  if ((payload.notification_type || "").toString().toLowerCase() === "modal") {
    try {
      await invoke("pause_timer");
    } catch (e) {
      console.error("暂停失败:", e);
    }
    showNoticeModal(payload);
  }
});

async function refreshSecurityStatus() {
  try {
    const status = await invoke("get_security_status") as any;
    securityStatus = {
      password_set: !!status.password_set,
      lock_remaining_seconds: status.lock_remaining_seconds ?? 0,
      remaining_attempts: status.remaining_attempts ?? 0,
      max_attempts: status.max_attempts ?? 3,
      security_question: status.security_question || "",
      safe_mode: !!status.safe_mode,
    };
  } catch (e) {
    console.error("获取安全状态失败:", e);
  }
}

function showAboutModal() {
  aboutModal?.classList.remove("hidden");
}

function hideAboutModal() {
  aboutModal?.classList.add("hidden");
}

listen("show-about", () => showAboutModal());

listen("exit-requested", async () => {
  await refreshActivationStatus();
  if (!activationStatus.activated) {
    await appWindow.close();
    return;
  }
  await refreshSecurityStatus();
  if (!securityStatus.safe_mode) {
    showExitModal();
  }
});

// ===== 设置面板功能 =====
const btnSettings = document.getElementById("btn-settings") as HTMLButtonElement;
const btnHelp = document.getElementById("btn-help") as HTMLButtonElement;
const helpModal = document.getElementById("help-modal") as HTMLDivElement;
const btnCloseHelp = document.getElementById("btn-close-help") as HTMLButtonElement;
const aboutModal = document.getElementById("about-modal") as HTMLDivElement;
const btnCloseAbout = document.getElementById("btn-close-about") as HTMLButtonElement;
const btnAboutOk = document.getElementById("btn-about-ok") as HTMLButtonElement;
const settingsPanel = document.getElementById("settings-panel") as HTMLDivElement;
const btnCloseSettings = document.getElementById("btn-close-settings") as HTMLButtonElement;
const btnSaveSettings = document.getElementById("btn-save-settings") as HTMLButtonElement;
const btnOpenLogFile = document.getElementById("btn-open-log-file") as HTMLButtonElement;
const btnTestLock = document.getElementById("btn-test-lock") as HTMLButtonElement;
const advanceNotice = document.getElementById("advance-notice") as HTMLSelectElement;
const executionModeOptions = document.querySelectorAll("input[name=\"execution-mode\"]") as NodeListOf<HTMLInputElement>;
const loopIntervalContainer = document.getElementById("loop-interval-container") as HTMLDivElement;
const loopIntervalPreset = document.getElementById("loop-interval-preset") as HTMLSelectElement;
const loopIntervalCustom = document.getElementById("loop-interval-custom") as HTMLInputElement;
const enforceRelockToggle = document.getElementById("enforce-relock") as HTMLInputElement;

// 设置项元素
const timeLimitEnabled = document.getElementById("time-limit-enabled") as HTMLInputElement;
const timeRangeContainer = document.getElementById("time-range-container") as HTMLDivElement;
const startTime = document.getElementById("start-time") as HTMLInputElement;
const endTime = document.getElementById("end-time") as HTMLInputElement;

const weekdayLimitEnabled = document.getElementById("weekday-limit-enabled") as HTMLInputElement;
const weekdaysContainer = document.getElementById("weekdays-container") as HTMLDivElement;

const autoStart = document.getElementById("auto-start") as HTMLInputElement;
const startMinimized = document.getElementById("start-minimized") as HTMLInputElement;
const startTimerAuto = document.getElementById("start-timer-auto") as HTMLInputElement;

// 打开设置面板
btnSettings?.addEventListener("click", async () => {
  settingsPanel.classList.remove("hidden");
  await loadSettings();
});

btnHelp?.addEventListener("click", () => {
  helpModal?.classList.remove("hidden");
});

btnCloseHelp?.addEventListener("click", () => {
  helpModal?.classList.add("hidden");
});

helpModal?.addEventListener("click", (event) => {
  if (event.target === helpModal) {
    helpModal.classList.add("hidden");
  }
});

btnCloseAbout?.addEventListener("click", hideAboutModal);
btnAboutOk?.addEventListener("click", hideAboutModal);
aboutModal?.addEventListener("click", (event) => {
  if (event.target === aboutModal) {
    hideAboutModal();
  }
});

btnCloseSetup?.addEventListener("click", () => {
  hidePasswordSetup();
});

passwordSetupModal?.addEventListener("click", (event) => {
  if (event.target === passwordSetupModal && !setupMandatory) {
    hidePasswordSetup();
  }
});

btnSavePassword?.addEventListener("click", async () => {
  const password = setupPasswordInput.value.trim();
  const confirm = setupPasswordConfirm.value.trim();
  const question = setupQuestionSelect.value.trim();
  const answer = setupAnswerInput.value.trim();

  if (password.length < 4) {
    setupError.textContent = "密码长度至少4位";
    return;
  }
  if (password !== confirm) {
    setupError.textContent = "两次密码输入不一致";
    return;
  }
  if (!question) {
    setupError.textContent = "请选择密保问题";
    return;
  }
  if (!answer) {
    setupError.textContent = "请输入密保答案";
    return;
  }

  try {
    await invoke("setup_password", {
      password,
      securityQuestion: question,
      securityAnswer: answer,
    });
    setupMandatory = false;
    passwordSetupModal?.classList.add("hidden");
    await refreshSecurityStatus();
  } catch (e) {
    setupError.textContent = `设置失败: ${e}`;
  }
});

btnCloseExit?.addEventListener("click", () => {
  hideExitModal();
});

btnExitCancel?.addEventListener("click", () => {
  hideExitModal();
});

exitPasswordModal?.addEventListener("click", (event) => {
  if (event.target === exitPasswordModal) {
    hideExitModal();
  }
});

btnExitConfirm?.addEventListener("click", async () => {
  const password = exitPasswordInput.value.trim();
  if (!password) {
    exitError.textContent = "请输入退出密码";
    return;
  }
  try {
    const result = await invoke("verify_exit_password", { password }) as any;
    if (!result?.ok) {
      if (result?.locked) {
        exitLockInfo.textContent = `密码错误已锁定，请在 ${formatLockCountdown(result.lock_remaining_seconds || 0)} 后再试`;
        btnExitConfirm.disabled = true;
      } else {
        exitError.textContent = "密码错误，请重试";
        securityStatus.remaining_attempts = result?.remaining_attempts ?? securityStatus.remaining_attempts;
        exitLockInfo.textContent = `剩余尝试次数: ${securityStatus.remaining_attempts}/${securityStatus.max_attempts}`;
      }
    }
  } catch (e) {
    exitError.textContent = `验证失败: ${e}`;
  }
});

btnForgotPassword?.addEventListener("click", async () => {
  await refreshSecurityStatus();
  if (!securityStatus.security_question) {
    exitError.textContent = "尚未设置密保问题";
    return;
  }
  hideExitModal();
  showResetModal();
});

btnCloseReset?.addEventListener("click", () => {
  hideResetModal();
});

btnResetCancel?.addEventListener("click", () => {
  hideResetModal();
});

resetPasswordModal?.addEventListener("click", (event) => {
  if (event.target === resetPasswordModal) {
    hideResetModal();
  }
});

btnResetPassword?.addEventListener("click", async () => {
  const answer = resetAnswerInput.value.trim();
  const password = resetPasswordInput.value.trim();
  const confirm = resetPasswordConfirm.value.trim();
  if (!answer) {
    resetError.textContent = "请输入密保答案";
    return;
  }
  if (password.length < 4) {
    resetError.textContent = "新密码长度至少4位";
    return;
  }
  if (password !== confirm) {
    resetError.textContent = "两次密码输入不一致";
    return;
  }
  try {
    await invoke("reset_password", {
      securityAnswer: answer,
      newPassword: password,
    });
    hideResetModal();
    await refreshSecurityStatus();
  } catch (e) {
    resetError.textContent = `重置失败: ${e}`;
  }
});

// 关闭设置面板
btnCloseSettings?.addEventListener("click", () => {
  settingsPanel.classList.add("hidden");
});

// 时间段限制开关
timeLimitEnabled?.addEventListener("change", () => {
  if (timeLimitEnabled.checked) {
    timeRangeContainer.classList.add("active");
  } else {
    timeRangeContainer.classList.remove("active");
  }
});

// 星期限制开关
weekdayLimitEnabled?.addEventListener("change", () => {
  if (weekdayLimitEnabled.checked) {
    weekdaysContainer.classList.add("active");
  } else {
    weekdaysContainer.classList.remove("active");
  }
});

function updateLoopIntervalControls() {
  const mode = (document.querySelector("input[name=\"execution-mode\"]:checked") as HTMLInputElement)?.value || "loop";
  if (mode === "loop") {
    loopIntervalContainer?.classList.add("active");
  } else {
    loopIntervalContainer?.classList.remove("active");
  }

  if (loopIntervalPreset?.value === "custom") {
    loopIntervalCustom?.classList.remove("hidden-input");
  } else {
    loopIntervalCustom?.classList.add("hidden-input");
    loopIntervalCustom.value = loopIntervalPreset?.value || "5";
  }
}

function applyLoopIntervalValue(minutes: number) {
  const presets = new Set([5, 10, 15]);
  if (presets.has(minutes)) {
    loopIntervalPreset.value = minutes.toString();
    loopIntervalCustom.value = minutes.toString();
    loopIntervalCustom.classList.add("hidden-input");
    return;
  }
  loopIntervalPreset.value = "custom";
  loopIntervalCustom.classList.remove("hidden-input");
  loopIntervalCustom.value = minutes.toString();
}

function resolveLoopIntervalMinutesFromUI(): number {
  if (loopIntervalPreset.value !== "custom") {
    return parseInt(loopIntervalPreset.value, 10);
  }
  return parseInt(loopIntervalCustom.value, 10);
}

executionModeOptions?.forEach((option) => {
  option.addEventListener("change", updateLoopIntervalControls);
});

loopIntervalPreset?.addEventListener("change", updateLoopIntervalControls);

// 加载设置
async function loadSettings() {
  try {
    const config = await invoke("get_config") as any;

    // 执行动作
    const action = normalizeActionType(config.action?.action_type);
    currentActionType = action;
    const actionRadio = document.querySelector(`input[name="action"][value="${action}"]`) as HTMLInputElement;
    if (actionRadio) {
      actionRadio.checked = true;
    }
    if (latestRuntime) {
      renderTimerLabel(latestRuntime);
    }

    // 时间段限制
    timeLimitEnabled.checked = config.schedule?.time_limit_enabled || false;
    if (timeLimitEnabled.checked) {
      timeRangeContainer.classList.add("active");
    }
    startTime.value = config.schedule?.start_time || "09:00";
    endTime.value = config.schedule?.end_time || "18:00";

    // 星期限制
    weekdayLimitEnabled.checked = config.schedule?.weekday_limit_enabled || false;
    if (weekdayLimitEnabled.checked) {
      weekdaysContainer.classList.add("active");
    }
    // 设置星期选择
    const weekdays = config.schedule?.weekdays || [1, 2, 3, 4, 5];
    document.querySelectorAll(".weekdays input[type=\"checkbox\"]").forEach((cb) => {
      const checkbox = cb as HTMLInputElement;
      checkbox.checked = weekdays.includes(parseInt(checkbox.value));
    });

    // 启动设置
    autoStart.checked = config.startup?.auto_start || false;
    startMinimized.checked = config.startup?.start_minimized || false;
    startTimerAuto.checked = config.startup?.start_timer_automatically || false;

    // 提前通知
    if (advanceNotice) {
      advanceNotice.value = (config.timer?.advance_notice_seconds ?? 30).toString();
    }

    // 循环设置
    loopEnabled = config.timer?.loop_enabled ?? true;
    loopIntervalMinutes = config.timer?.loop_interval_minutes ?? 5;
    enforceRelockDuringRest = config.timer?.enforce_relock_during_rest ?? true;
    const modeRadio = document.querySelector(`input[name="execution-mode"][value="${loopEnabled ? "loop" : "single"}"]`) as HTMLInputElement;
    if (modeRadio) {
      modeRadio.checked = true;
    }
    applyLoopIntervalValue(loopIntervalMinutes);
    updateLoopIntervalControls();
    if (enforceRelockToggle) {
      enforceRelockToggle.checked = enforceRelockDuringRest;
    }
  } catch (e) {
    console.error("加载设置失败:", e);
  }
}

// 保存设置
btnSaveSettings?.addEventListener("click", async () => {
  try {
    const config = await invoke("get_config") as any;
    // 获取执行动作
    const actionRadio = document.querySelector("input[name=\"action\"]:checked") as HTMLInputElement;
    const actionType = normalizeActionType(actionRadio?.value || "lock");
    const selectedMode = (document.querySelector("input[name=\"execution-mode\"]:checked") as HTMLInputElement)?.value || "loop";
    const nextLoopEnabled = selectedMode === "loop";
    const resolvedLoopInterval = resolveLoopIntervalMinutesFromUI();
    const nextLoopInterval = Number.isFinite(resolvedLoopInterval) && resolvedLoopInterval > 0
      ? resolvedLoopInterval
      : (config.timer?.loop_interval_minutes ?? 5);
    const nextEnforceRelock = enforceRelockToggle?.checked ?? true;
    if (nextLoopEnabled && (!Number.isFinite(nextLoopInterval) || nextLoopInterval < 1 || nextLoopInterval > 1440)) {
      alert("循环间隔请输入 1-1440 分钟");
      return;
    }

    // 获取星期选择
    const selectedWeekdays: number[] = [];
    document.querySelectorAll(".weekdays input[type=\"checkbox\"]:checked").forEach((cb) => {
      selectedWeekdays.push(parseInt((cb as HTMLInputElement).value));
    });

    // 保存执行动作配置
    await invoke("update_action_config", {
      config: {
        action_type: actionType,
        show_notice: true,
      },
    });
    currentActionType = actionType;

    // 保存生效规则配置
    await invoke("update_schedule_config", {
      config: {
        time_limit_enabled: timeLimitEnabled.checked,
        weekday_limit_enabled: weekdayLimitEnabled.checked,
        start_time: startTime.value,
        end_time: endTime.value,
        weekdays: selectedWeekdays,
        logic: "AND",
      },
    });

    // 保存启动配置
    await invoke("update_startup_config", {
      config: {
        auto_start: autoStart.checked,
        start_minimized: startMinimized.checked,
        start_timer_automatically: startTimerAuto.checked,
      },
    });

    // 保存提示配置
    const noticeSeconds = parseInt(advanceNotice?.value || "30");
    await invoke("update_timer_config", {
      config: {
        interval_minutes: config.timer?.interval_minutes ?? 30,
        advance_notice_seconds: noticeSeconds,
        max_delay_times: config.timer?.max_delay_times ?? 3,
        delay_options: config.timer?.delay_options ?? [5, 10, 30],
        loop_enabled: nextLoopEnabled,
        loop_interval_minutes: nextLoopInterval,
        enforce_relock_during_rest: nextEnforceRelock,
      },
    });
    loopEnabled = nextLoopEnabled;
    loopIntervalMinutes = nextLoopInterval;
    enforceRelockDuringRest = nextEnforceRelock;

    // 保存开机自启
    await invoke("set_auto_start", { enabled: autoStart.checked });

    await refreshScheduleIndicator();
    if (latestRuntime) {
      renderTimerLabel(latestRuntime);
    }
    alert("设置已保存");
    settingsPanel.classList.add("hidden");
  } catch (e) {
    console.error("保存设置失败:", e);
    alert("保存设置失败: " + e);
  }
});

// 打开日志文件（自动创建并写入日志事件）
btnOpenLogFile?.addEventListener("click", async () => {
  try {
    await invoke("open_log_file");
  } catch (e) {
    console.error("打开日志文件失败:", e);
    alert("打开日志文件失败: " + e);
  }
});

// 测试锁屏
btnTestLock?.addEventListener("click", async () => {
  try {
    await invoke("execute_system_action", { action: "lock" });
  } catch (e) {
    console.error("测试锁屏失败:", e);
    alert("测试锁屏失败: " + e);
  }
});

// 初始化
document.addEventListener("DOMContentLoaded", init);
