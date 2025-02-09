use serde_json::Value;
use std::{
    fs::{self, File},
    io::{self, Write},
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

// 编辑Settings.json文件
fn edit_settings_json() -> io::Result<()> {
    let settings_path = "./Settings.json";

    // 读取文件内容
    let settings_content = fs::read_to_string(settings_path)?;
    let mut settings: Value = serde_json::from_str(&settings_content)?;

    // 修改设置
    settings["IsTransientDisabled"] = serde_json::Value::Bool(true);
    settings["IsSplashEnabled"] = serde_json::Value::Bool(false);
    settings["IsCompatibleWindowTransparentEnabled"] = serde_json::Value::Bool(true);

    // 写回文件
    let new_settings_content = serde_json::to_string_pretty(&settings)?;
    let mut file = File::create(settings_path)?;
    file.write_all(new_settings_content.as_bytes())?;

    Ok(())
}

// 检查ClassIsland.exe是否存在
fn check_classisland_exe_exists() -> bool {
    fs::metadata("./ClassIsland.exe").is_ok()
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

    // 编辑Settings.json文件
    if let Err(e) = edit_settings_json() {
        eprintln!("编辑Settings.json失败: {:?}", e);
        process::exit(1);
    }

    // 启动主程序
    run_wine_command(&["ClassIsland.exe"]);

    // 创建托盘图标（嵌入的图标数据）
    let cursor_red = Cursor::new(include_bytes!("../assets/icon.png"));
    let decoder_red = png::Decoder::new(cursor_red);
    let (info_red, mut reader_red) = decoder_red.read_info().unwrap();
    let mut buf_red = vec![0; info_red.buffer_size()];
    reader_red.next_frame(&mut buf_red).unwrap();

    let icon_red = IconSource::Data {
        data: buf_red,
        height: 32,
        width: 32,
    };

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
