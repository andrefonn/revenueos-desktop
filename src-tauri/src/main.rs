#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod updates;
use revenue_os_desktop::policy;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    webview::NewWindowResponse,
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_dialog::DialogExt;

static POPUP_ID: AtomicU64 = AtomicU64::new(0);
// Preserve WebView2's actual network user agent for provider compatibility.
// This page-only UI marker grants no authentication or native permissions.
const DESKTOP_MARKER: &str = "Object.defineProperty(navigator, 'userAgent', {value: navigator.userAgent + ' RevenueOSDesktop'});";

fn navigation(url: &url::Url) -> bool {
    if policy::is_app(url) || policy::is_local(url) || policy::is_meta(url) {
        return true;
    }
    if policy::is_safe_external(url) {
        let _ = open::that_detached(url.as_str());
    }
    false
}

fn connect(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build();
        let reachable = match client {
            Ok(client) => client
                .head(format!("{}/login", policy::APP_ORIGIN))
                .send()
                .await
                .map(|r| r.status().is_success() || r.status().is_redirection())
                .unwrap_or(false),
            Err(_) => false,
        };
        if let Some(window) = app.get_webview_window("main") {
            let target = if reachable {
                format!("{}/dashboard", policy::APP_ORIGIN)
            } else {
                "http://tauri.localhost/offline.html".to_owned()
            };
            if let Ok(url) = target.parse() {
                let _ = window.navigate(url);
            }
        }
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize(); let _ = window.show(); let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().with_state_flags(tauri_plugin_window_state::StateFlags::POSITION | tauri_plugin_window_state::StateFlags::SIZE | tauri_plugin_window_state::StateFlags::MAXIMIZED).build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(updates::UpdateState::default())
        .setup(|app| {
            let retry = MenuItem::with_id(app, "retry", "Tentar conexão novamente", true, Some("Ctrl+R"))?;
            let dashboard = MenuItem::with_id(app, "dashboard", "Ir para o dashboard", true, Some("Ctrl+Home"))?;
            let update = MenuItem::with_id(app, "update", "Verificar atualizações", true, None::<&str>)?;
            let about = MenuItem::with_id(app, "about", "Sobre o Revenue OS", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Sair", true, Some("Alt+F4"))?;
            let sep = PredefinedMenuItem::separator(app)?;
            let submenu = Submenu::with_items(app, "Aplicativo", true, &[&dashboard, &retry, &sep, &update, &about, &quit])?;
            app.set_menu(Menu::with_items(app, &[&submenu])?)?;
            let popup_app = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Revenue OS")
                .inner_size(1360.0, 900.0).min_inner_size(900.0, 640.0)
                .initialization_script(DESKTOP_MARKER)
                .on_navigation(navigation)
                .on_new_window(move |url, features| {
                    if policy::is_app(&url) || policy::is_meta(&url) || url.as_str() == "about:blank" {
                        let label = format!("authorization-{}", POPUP_ID.fetch_add(1, Ordering::Relaxed));
                        let popup = WebviewWindowBuilder::new(&popup_app, label, WebviewUrl::External(url))
                            .title("Revenue OS — autorização")
                            .inner_size(720.0, 780.0).initialization_script(DESKTOP_MARKER)
                            .window_features(features)
                            .on_navigation(|url| url.as_str() == "about:blank" || navigation(url))
                            .on_new_window(|_, _| NewWindowResponse::Deny)
                            .build();
                        match popup { Ok(window) => NewWindowResponse::Create { window }, Err(_) => NewWindowResponse::Deny }
                    } else {
                        if policy::is_safe_external(&url) { let _ = open::that_detached(url.as_str()); }
                        NewWindowResponse::Deny
                    }
                })
                .build()?;
            connect(app.handle().clone());
            updates::check(app.handle().clone(), false);
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "retry" => connect(app.clone()),
            "dashboard" => { if let Some(window) = app.get_webview_window("main") { let _ = window.navigate(format!("{}/dashboard", policy::APP_ORIGIN).parse().expect("constant application URL")); } },
            "update" => updates::check(app.clone(), true),
            "about" => { app.dialog().message(format!("Revenue OS para Windows\nVersão {}\n\nUse sua conta existente. Requer conexão com a internet.\nAtualizações: Aplicativo → Verificar atualizações.", app.package_info().version)).title("Sobre o Revenue OS").show(|_| {}); },
            "quit" => app.exit(0),
            _ => {}
        })
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, tauri::WindowEvent::CloseRequested { .. }) { window.app_handle().exit(0); }
        })
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar o Revenue OS");
}
