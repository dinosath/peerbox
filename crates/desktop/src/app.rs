use makepad_widgets::*;

use crate::commands::identity_ops::IdentityOps;
use crate::core::build_peerbox_app;

script_mod! {
    use mod.widgets.*

    tab_btn = <Button> {
        text: "Tab"
        draw_text: { color: #x8 }
    }

    tab_btn_active = <Button> {
        text: "Tab"
        draw_bg: { color: #x3 }
        draw_text: { color: #xF }
    }
}

app_main!(App);

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .init();

        log!("Peerbox desktop application started");

        let _config = match config::PeerBoxConfig::load() {
            Ok(c) => {
                log!("Loaded config: node_name={}", c.node_name);
                c
            }
            Err(e) => {
                log!("Failed to load config: {}", e);
                return;
            }
        };

        let app = build_peerbox_app();
        let identity = IdentityOps::generate();
        log!("Node identity: {}", identity.get_node_id());

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                if let Err(e) = app.start().await {
                    log!("Failed to start peerbox: {}", e);
                    return;
                }
                log!("Peerbox core running");
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                }
            });
        });

        self.ui.redraw(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let _ = (cx, actions);
    }
}
