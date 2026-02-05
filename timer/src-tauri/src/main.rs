// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use timer_lib::single_instance::{activate_existing_window, SingleInstance};

fn main() {
    // 单实例检查
    let single_instance = SingleInstance::new();

    if single_instance.is_none() {
        // 已有实例在运行，激活它并退出
        eprintln!("TimerApp 已经在运行中，正在激活已有窗口...");
        activate_existing_window();
        std::process::exit(0);
    }

    // 第一个实例，正常启动
    timer_lib::run()
}
