use crate::{
    cmds,
    config::Config,
    feat,
    utils::{dirs, resolve},
};
use anyhow::Result;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Wry,
};

const TRAY_ID: &str = "main";

#[derive(Clone)]
struct TrayItems {
    rule_mode: CheckMenuItem<Wry>,
    global_mode: CheckMenuItem<Wry>,
    direct_mode: CheckMenuItem<Wry>,
    system_proxy: CheckMenuItem<Wry>,
    tun_mode: CheckMenuItem<Wry>,
}

fn tray_items() -> &'static Mutex<Option<TrayItems>> {
    static TRAY_ITEMS: OnceCell<Mutex<Option<TrayItems>>> = OnceCell::new();
    TRAY_ITEMS.get_or_init(|| Mutex::new(None))
}

pub struct Tray {}

impl Tray {
    pub fn tray_menu(app_handle: &AppHandle) -> Result<Menu<Wry>> {
        let zh = { Config::verge().latest().language == Some("zh".into()) };
        let version = app_handle.package_info().version.to_string();

        macro_rules! t {
            ($en: expr, $zh: expr) => {
                if zh {
                    $zh
                } else {
                    $en
                }
            };
        }

        let open_window = MenuItem::with_id(
            app_handle,
            "open_window",
            t!("Dashboard", "打开面板"),
            true,
            None::<&str>,
        )?;
        let rule_mode = CheckMenuItem::with_id(
            app_handle,
            "rule_mode",
            t!("Rule Mode", "规则模式"),
            true,
            false,
            None::<&str>,
        )?;
        let global_mode = CheckMenuItem::with_id(
            app_handle,
            "global_mode",
            t!("Global Mode", "全局模式"),
            true,
            false,
            None::<&str>,
        )?;
        let direct_mode = CheckMenuItem::with_id(
            app_handle,
            "direct_mode",
            t!("Direct Mode", "直连模式"),
            true,
            false,
            None::<&str>,
        )?;
        let system_proxy = CheckMenuItem::with_id(
            app_handle,
            "system_proxy",
            t!("System Proxy", "系统代理"),
            true,
            false,
            None::<&str>,
        )?;
        let tun_mode = CheckMenuItem::with_id(
            app_handle,
            "tun_mode",
            t!("TUN Mode", "Tun 模式"),
            true,
            false,
            None::<&str>,
        )?;
        let copy_env = MenuItem::with_id(
            app_handle,
            "copy_env",
            t!("Copy Env", "复制环境变量"),
            true,
            None::<&str>,
        )?;
        let open_app_dir = MenuItem::with_id(
            app_handle,
            "open_app_dir",
            t!("App Dir", "应用目录"),
            true,
            None::<&str>,
        )?;
        let open_core_dir = MenuItem::with_id(
            app_handle,
            "open_core_dir",
            t!("Core Dir", "内核目录"),
            true,
            None::<&str>,
        )?;
        let open_logs_dir = MenuItem::with_id(
            app_handle,
            "open_logs_dir",
            t!("Logs Dir", "日志目录"),
            true,
            None::<&str>,
        )?;
        let restart_clash = MenuItem::with_id(
            app_handle,
            "restart_clash",
            t!("Restart Core", "重启内核"),
            true,
            None::<&str>,
        )?;
        let restart_app = MenuItem::with_id(
            app_handle,
            "restart_app",
            t!("Restart App", "重启应用"),
            true,
            None::<&str>,
        )?;
        let app_version = MenuItem::with_id(
            app_handle,
            "app_version",
            format!("Version {version}"),
            false,
            None::<&str>,
        )?;
        let quit = MenuItem::with_id(app_handle, "quit", t!("Quit", "退出"), true, None::<&str>)?;

        let open_dir = Submenu::with_items(
            app_handle,
            t!("Open Dir", "打开目录"),
            true,
            &[&open_app_dir, &open_core_dir, &open_logs_dir],
        )?;
        let more = Submenu::with_items(
            app_handle,
            t!("More", "更多"),
            true,
            &[&restart_clash, &restart_app, &app_version],
        )?;
        let separator_1 = PredefinedMenuItem::separator(app_handle)?;
        let separator_2 = PredefinedMenuItem::separator(app_handle)?;
        let separator_3 = PredefinedMenuItem::separator(app_handle)?;

        *tray_items().lock() = Some(TrayItems {
            rule_mode: rule_mode.clone(),
            global_mode: global_mode.clone(),
            direct_mode: direct_mode.clone(),
            system_proxy: system_proxy.clone(),
            tun_mode: tun_mode.clone(),
        });

        Ok(Menu::with_items(
            app_handle,
            &[
                &open_window,
                &separator_1,
                &rule_mode,
                &global_mode,
                &direct_mode,
                &separator_2,
                &system_proxy,
                &tun_mode,
                &copy_env,
                &open_dir,
                &more,
                &separator_3,
                &quit,
            ],
        )?)
    }

    pub fn update_systray(app_handle: &AppHandle) -> Result<()> {
        let menu = Tray::tray_menu(app_handle)?;

        if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
            tray.set_menu(Some(menu))?;
        } else {
            let icon = app_handle
                .default_window_icon()
                .cloned()
                .ok_or(anyhow::anyhow!("failed to get default tray icon"))?;
            TrayIconBuilder::with_id(TRAY_ID)
                .menu(&menu)
                .icon(icon)
                .show_menu_on_left_click(false)
                .on_menu_event(|app_handle, event| {
                    Tray::on_menu_event(app_handle, event.id().as_ref());
                })
                .on_tray_icon_event(|tray, event| {
                    Tray::on_tray_icon_event(tray.app_handle(), event);
                })
                .build(app_handle)?;
        }

        Tray::update_part(app_handle)?;
        Ok(())
    }

    pub fn update_part(app_handle: &AppHandle) -> Result<()> {
        let zh = { Config::verge().latest().language == Some("zh".into()) };
        let version = app_handle.package_info().version.to_string();

        macro_rules! t {
            ($en: expr, $zh: expr) => {
                if zh {
                    $zh
                } else {
                    $en
                }
            };
        }

        let mode = {
            Config::clash()
                .latest()
                .0
                .get("mode")
                .map(|val| val.as_str().unwrap_or("rule"))
                .unwrap_or("rule")
                .to_owned()
        };

        if let Some(items) = tray_items().lock().as_ref() {
            let _ = items.rule_mode.set_checked(mode == "rule");
            let _ = items.global_mode.set_checked(mode == "global");
            let _ = items.direct_mode.set_checked(mode == "direct");

            #[cfg(target_os = "linux")]
            match mode.as_str() {
                "rule" => {
                    let _ = items.rule_mode.set_text(t!("Rule Mode  ✔", "规则模式  ✔"));
                    let _ = items.global_mode.set_text(t!("Global Mode", "全局模式"));
                    let _ = items.direct_mode.set_text(t!("Direct Mode", "直连模式"));
                }
                "global" => {
                    let _ = items.rule_mode.set_text(t!("Rule Mode", "规则模式"));
                    let _ = items
                        .global_mode
                        .set_text(t!("Global Mode  ✔", "全局模式  ✔"));
                    let _ = items.direct_mode.set_text(t!("Direct Mode", "直连模式"));
                }
                "direct" => {
                    let _ = items.rule_mode.set_text(t!("Rule Mode", "规则模式"));
                    let _ = items.global_mode.set_text(t!("Global Mode", "全局模式"));
                    let _ = items
                        .direct_mode
                        .set_text(t!("Direct Mode  ✔", "直连模式  ✔"));
                }
                _ => {}
            }
        }

        let verge = Config::verge();
        let verge = verge.latest();
        let system_proxy = verge.enable_system_proxy.as_ref().unwrap_or(&false);
        let tun_mode = verge.enable_tun_mode.as_ref().unwrap_or(&false);
        #[cfg(target_os = "macos")]
        let tray_icon = verge.tray_icon.clone().unwrap_or("monochrome".to_string());
        let common_tray_icon = verge.common_tray_icon.as_ref().unwrap_or(&false);
        let sysproxy_tray_icon = verge.sysproxy_tray_icon.as_ref().unwrap_or(&false);
        let tun_tray_icon = verge.tun_tray_icon.as_ref().unwrap_or(&false);

        let mut indication_icon = if *system_proxy {
            #[cfg(target_os = "macos")]
            let mut icon = match tray_icon.as_str() {
                "monochrome" => include_bytes!("../../icons/tray-icon-sys-mono.ico").to_vec(),
                "colorful" => include_bytes!("../../icons/tray-icon-sys.ico").to_vec(),
                _ => include_bytes!("../../icons/tray-icon-sys-mono.ico").to_vec(),
            };
            #[cfg(not(target_os = "macos"))]
            let mut icon = include_bytes!("../../icons/tray-icon-sys.ico").to_vec();

            if *sysproxy_tray_icon {
                let icon_dir_path = dirs::app_home_dir()?.join("icons");
                let png_path = icon_dir_path.join("sysproxy.png");
                let ico_path = icon_dir_path.join("sysproxy.ico");
                if ico_path.exists() {
                    icon = std::fs::read(ico_path).unwrap();
                } else if png_path.exists() {
                    icon = std::fs::read(png_path).unwrap();
                }
            }
            icon
        } else {
            #[cfg(target_os = "macos")]
            let mut icon = match tray_icon.as_str() {
                "monochrome" => include_bytes!("../../icons/tray-icon-mono.ico").to_vec(),
                "colorful" => include_bytes!("../../icons/tray-icon.ico").to_vec(),
                _ => include_bytes!("../../icons/tray-icon-mono.ico").to_vec(),
            };
            #[cfg(not(target_os = "macos"))]
            let mut icon = include_bytes!("../../icons/tray-icon.ico").to_vec();
            if *common_tray_icon {
                let icon_dir_path = dirs::app_home_dir()?.join("icons");
                let png_path = icon_dir_path.join("common.png");
                let ico_path = icon_dir_path.join("common.ico");
                if ico_path.exists() {
                    icon = std::fs::read(ico_path).unwrap();
                } else if png_path.exists() {
                    icon = std::fs::read(png_path).unwrap();
                }
            }
            icon
        };

        if *tun_mode {
            #[cfg(target_os = "macos")]
            let mut icon = match tray_icon.as_str() {
                "monochrome" => include_bytes!("../../icons/tray-icon-tun-mono.ico").to_vec(),
                "colorful" => include_bytes!("../../icons/tray-icon-tun.ico").to_vec(),
                _ => include_bytes!("../../icons/tray-icon-tun-mono.ico").to_vec(),
            };
            #[cfg(not(target_os = "macos"))]
            let mut icon = include_bytes!("../../icons/tray-icon-tun.ico").to_vec();
            if *tun_tray_icon {
                let icon_dir_path = dirs::app_home_dir()?.join("icons");
                let png_path = icon_dir_path.join("tun.png");
                let ico_path = icon_dir_path.join("tun.ico");
                if ico_path.exists() {
                    icon = std::fs::read(ico_path).unwrap();
                } else if png_path.exists() {
                    icon = std::fs::read(png_path).unwrap();
                }
            }
            indication_icon = icon
        }

        let tray = app_handle
            .tray_by_id(TRAY_ID)
            .ok_or(anyhow::anyhow!("failed to get system tray"))?;

        let icon = Image::from_bytes(&indication_icon)?;

        #[cfg(target_os = "macos")]
        {
            let is_template = matches!(tray_icon.as_str(), "monochrome");
            let _ = tray.set_icon_with_as_template(Some(icon), is_template);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = tray.set_icon(Some(icon));
        }

        if let Some(items) = tray_items().lock().as_ref() {
            let _ = items.system_proxy.set_checked(*system_proxy);
            let _ = items.tun_mode.set_checked(*tun_mode);
            #[cfg(target_os = "linux")]
            {
                if *system_proxy {
                    let _ = items
                        .system_proxy
                        .set_text(t!("System Proxy  ✔", "系统代理  ✔"));
                } else {
                    let _ = items.system_proxy.set_text(t!("System Proxy", "系统代理"));
                }
                if *tun_mode {
                    let _ = items.tun_mode.set_text(t!("TUN Mode  ✔", "Tun 模式  ✔"));
                } else {
                    let _ = items.tun_mode.set_text(t!("TUN Mode", "Tun 模式"));
                }
            }
        }

        let switch_map = {
            let mut map = std::collections::HashMap::new();
            map.insert(true, "on");
            map.insert(false, "off");
            map
        };

        let mut current_profile_name = "None".to_string();
        let profiles = Config::profiles();
        let profiles = profiles.latest();
        if let Some(current_profile_uid) = profiles.get_current() {
            let current_profile = profiles.get_item(&current_profile_uid);
            current_profile_name = match &current_profile.unwrap().name {
                Some(profile_name) => profile_name.to_string(),
                None => current_profile_name,
            };
        };
        let _ = tray.set_tooltip(Some(format!(
            "Clash Verge {version}\n{}: {}\n{}: {}\n{}: {}",
            t!("SysProxy", "系统代理"),
            switch_map[system_proxy],
            t!("TUN", "Tun模式"),
            switch_map[tun_mode],
            t!("Profile", "当前订阅"),
            current_profile_name
        )));

        Ok(())
    }

    pub fn on_click(app_handle: &AppHandle) {
        let tray_event = { Config::verge().latest().tray_event.clone() };
        let tray_event = tray_event.unwrap_or("main_window".into());
        match tray_event.as_str() {
            "system_proxy" => feat::toggle_system_proxy(),
            "tun_mode" => feat::toggle_tun_mode(),
            "main_window" => resolve::create_window(app_handle),
            _ => {}
        }
    }

    pub fn on_tray_icon_event(app_handle: &AppHandle, event: TrayIconEvent) {
        match event {
            #[cfg(not(target_os = "macos"))]
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => Tray::on_click(app_handle),
            #[cfg(target_os = "macos")]
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                ..
            } => Tray::on_click(app_handle),
            _ => {}
        }
    }

    pub fn on_menu_event(app_handle: &AppHandle, id: &str) {
        match id {
            mode @ ("rule_mode" | "global_mode" | "direct_mode") => {
                let mode = &mode[0..mode.len() - 5];
                feat::change_clash_mode(mode.into());
            }
            "open_window" => resolve::create_window(app_handle),
            "system_proxy" => feat::toggle_system_proxy(),
            "tun_mode" => feat::toggle_tun_mode(),
            "copy_env" => feat::copy_clash_env(app_handle),
            "open_app_dir" => crate::log_err!(cmds::open_app_dir()),
            "open_core_dir" => crate::log_err!(cmds::open_core_dir()),
            "open_logs_dir" => crate::log_err!(cmds::open_logs_dir()),
            "restart_clash" => feat::restart_clash_core(),
            "restart_app" => app_handle.request_restart(),
            "quit" => cmds::exit_app(app_handle.clone()),
            _ => {}
        }
    }
}
