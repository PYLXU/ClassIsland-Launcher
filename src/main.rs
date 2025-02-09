use std::{
    process::{self, Command, Stdio},
    thread,
    time::Duration,
};
use sysinfo::{ProcessExt, System, SystemExt};
use {std::io::Cursor, std::sync::mpsc, tray_item::IconSource, tray_item::TrayItem};

// 单实例检查
fn check_single_instance() {
    let instance = single_instance::SingleInstance::new("classisland-launcher").unwrap();
    if !instance.is_single() {
        process::exit(0);
    }
}

// 运行wine命令
fn run_wine_command(args: &[&str]) {
    Command::new("wine")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("执行命令失败");
}

// 检查进程是否存活
fn check_process_alive() -> bool {
    System::new_all()
        .processes_by_exact_name("ClassIsland.exe")
        .next()
        .is_some()
}

fn main() {
    check_single_instance();

    // 启动主程序
    run_wine_command(&["ClassIsland.exe"]);

    // 创建托盘图标（嵌入的图标数据）
    let cursor_red = Cursor::new(include_bytes!("../assets/icon.png"));
    let decoder_red = png::Decoder::new(cursor_red);
    let (info_red, mut reader_red) = decoder_red.read_info().unwrap();
    let mut buf_red = vec![0;info_red.buffer_size()];
    reader_red.next_frame(&mut buf_red).unwrap();

    let icon_red = IconSource::Data{data: buf_red, height: 32, width: 32};

    let mut tray = TrayItem::new("ClassIsland", icon_red).unwrap();

    // 添加菜单项
    tray.add_menu_item("编辑设置", || {
        run_wine_command(&["ClassIsland.exe", "--uri", "classisland://app/settings"]);
    })
    .unwrap();

    tray.add_menu_item("编辑档案", || {
        run_wine_command(&["ClassIsland.exe", "--uri", "classisland://app/profile"]);
    })
    .unwrap();

    tray.add_menu_item("退出", || {
        System::new_all()
            .processes_by_exact_name("ClassIsland.exe")
            .for_each(|p| {
                let _ = p.kill();
            });
        process::exit(0);
    })
    .unwrap();

    // 启动状态检查线程
    thread::spawn(|| loop {
        thread::sleep(Duration::from_secs(2));
        if !check_process_alive() {
            process::exit(0);
        }
    });

    // 进入消息循环
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}
