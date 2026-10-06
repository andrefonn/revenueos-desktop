use revenue_os_desktop::policy;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::AppHandleExt;

#[derive(Default)]
pub struct UpdateState {
    busy: Arc<AtomicBool>,
}
struct Operation(Arc<AtomicBool>);
impl Drop for Operation {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

fn message(app: &tauri::AppHandle, text: &str, error: bool) {
    app.dialog()
        .message(text)
        .title("Atualizações do Revenue OS")
        .kind(if error {
            MessageDialogKind::Error
        } else {
            MessageDialogKind::Info
        })
        .show(|_| {});
}

pub fn check(app: tauri::AppHandle, manual: bool) {
    let busy = app.state::<UpdateState>().busy.clone();
    if busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        if manual {
            message(
                &app,
                "Uma verificação ou atualização já está em andamento.",
                false,
            );
        }
        return;
    }
    tauri::async_runtime::spawn(async move {
        let _operation = Operation(busy);
        let report_errors = AtomicBool::new(manual);
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_title("Revenue OS — verificando atualizações…");
        }
        let result = run(&app, manual, &report_errors).await;
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_title("Revenue OS");
        }
        if let Some(window) = app.get_webview_window("update-progress") {
            let _ = window.close();
        }
        if result.is_err() && report_errors.load(Ordering::Relaxed) {
            message(&app, "Não foi possível concluir a atualização. Confira sua conexão e tente novamente. Sua versão instalada foi mantida; nenhum arquivo não verificado será instalado.", true);
        }
    });
}

async fn run(
    app: &tauri::AppHandle,
    manual: bool,
    report_errors: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state_app = app.clone();
    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(25))
        .on_before_exit(move || {
            let _ = state_app.save_window_state(
                tauri_plugin_window_state::StateFlags::POSITION
                    | tauri_plugin_window_state::StateFlags::SIZE
                    | tauri_plugin_window_state::StateFlags::MAXIMIZED,
            );
        })
        .build()?;
    let Some(mut update) = updater.check().await? else {
        if manual {
            message(
                app,
                &format!(
                    "Você está usando a versão mais recente: {}.",
                    app.package_info().version
                ),
                false,
            );
        }
        return Ok(());
    };
    if !policy::is_update(&update.download_url) {
        return Err("Untrusted update asset URL".into());
    }
    // Checking should fail quickly; downloading a signed installer on a slower
    // connection needs a separate, larger deadline.
    update.timeout = Some(Duration::from_secs(600));
    let accepted = app.dialog()
        .message(format!("A versão {} está disponível.\n\nA atualização será baixada e sua assinatura será verificada. Depois, o Revenue OS será fechado para instalar e reabrir. Salve seu trabalho antes de continuar.", update.version))
        .title("Atualização disponível")
        .buttons(MessageDialogButtons::OkCancelCustom("Atualizar agora".into(), "Depois".into()))
        .blocking_show();
    if !accepted {
        return Ok(());
    }
    report_errors.store(true, Ordering::Relaxed);
    let window = WebviewWindowBuilder::new(
        app,
        "update-progress",
        WebviewUrl::App("update.html".into()),
    )
    .title("Atualizando Revenue OS")
    .inner_size(560.0, 460.0)
    .resizable(false)
    .maximizable(false)
    .closable(false)
    .on_navigation(|url| policy::is_local(url))
    .build()?;
    let mut downloaded = 0u64;
    let mut last_percent = 101u64;
    update.download_and_install(|chunk, total| {
        downloaded += chunk as u64;
        let percent = total.filter(|value| *value > 0).map(|total| (downloaded.saturating_mul(100) / total).min(100));
        if let Some(percent) = percent {
            if percent != last_percent {
                last_percent = percent;
                let _ = window.eval(&format!("document.getElementById('progress').value={percent};document.getElementById('message').textContent='Baixando atualização: {percent}%';"));
            }
        }
    }, || {
        let _ = window.eval("document.getElementById('title').textContent='Verificando e instalando';document.getElementById('message').textContent='Aguarde. O aplicativo será reaberto após a instalação.';");
    }).await?;
    Ok(())
}
