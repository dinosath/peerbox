use std::sync::Arc;

use makepad_widgets::*;

use crate::core::build_peerbox_app;
use crate::commands::identity_ops::IdentityOps;

live_design! {
    link makepad_widgets;

    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    tab_btn = <Button> {
        text: "Tab"
        draw_text: { color: #x8 }
    }

    tab_btn_active = <Button> {
        text: "Tab"
        draw_bg: { color: #x3 }
        draw_text: { color: #xF }
    }

    App = {{App}} {
        ui: <Window> {
            window: {
                inner_size: vec2(960, 640),
                title: "Peerbox"
            }

            body = <View> {
                width: Fill, height: Fill,
                flow: Down,

                header = <View> {
                    width: Fill, height: 44,
                    flow: Right,
                    padding: { left: 16, right: 16 },
                    spacing: 24,
                    align: { y: 0.5 },
                    draw_bg: { color: #x1 }

                    logo = <Label> {
                        text: "Peerbox"
                        draw_text: { color: #xF, text_style: <MAPLE_FONT> { font_size: 14 } }
                    }

                    tabs = <View> {
                        flow: Right, spacing: 4, align: { y: 0.5 }
                        tab_dashboard = tab_btn_active { text: "Dashboard" }
                        tab_files = tab_btn { text: "Files" }
                        tab_peers = tab_btn { text: "Peers" }
                    }

                    <View> { width: Fill, height: Fill }

                    node_info = <Label> {
                        text: "Node: —"
                        draw_text: { color: #x6, text_style: <MAPLE_FONT> { font_size: 10 } }
                    }
                }

                content = <View> {
                    width: Fill, height: Fill, padding: 16, spacing: 12, flow: Down

                    dashboard_view = <View> {
                        width: Fill, height: Fill, flow: Down, spacing: 12

                        stats_row = <View> {
                            width: Fill, flow: Right, spacing: 12

                            = <View> {
                                width: 200, height: 100, flow: Down, padding: 16, spacing: 8
                                draw_bg: { color: #x24, radius: 8 }
                                <Label> {
                                    text: "Storage Usage"
                                    draw_text: { color: #x8, text_style: <MAPLE_FONT> { font_size: 10 } }
                                }
                                <Label> {
                                    text: "0 MB"
                                    draw_text: { color: #xF, text_style: <MAPLE_FONT> { font_size: 20 } }
                                }
                            }

                            = <View> {
                                width: 200, height: 100, flow: Down, padding: 16, spacing: 8
                                draw_bg: { color: #x24, radius: 8 }
                                <Label> {
                                    text: "Connected Peers"
                                    draw_text: { color: #x8, text_style: <MAPLE_FONT> { font_size: 10 } }
                                }
                                <Label> {
                                    text: "0"
                                    draw_text: { color: #xF, text_style: <MAPLE_FONT> { font_size: 20 } }
                                }
                            }

                            = <View> {
                                width: 200, height: 100, flow: Down, padding: 16, spacing: 8
                                draw_bg: { color: #x24, radius: 8 }
                                <Label> {
                                    text: "Integrity"
                                    draw_text: { color: #x8, text_style: <MAPLE_FONT> { font_size: 10 } }
                                }
                                <Label> {
                                    text: "OK"
                                    draw_text: { color: #x0E0, text_style: <MAPLE_FONT> { font_size: 20 } }
                                }
                            }
                        }

                        logs_area = <View> {
                            width: Fill, height: Fill, padding: 8, flow: Down, spacing: 2
                            draw_bg: { color: #x15, radius: 6 }

                            <Label> {
                                text: "Activity Log"
                                draw_text: { color: #x8, text_style: <MAPLE_FONT> { font_size: 10 } }
                            }

                            logs_scroll = <ScrollYView> {
                                width: Fill, height: Fill, flow: Down, spacing: 2
                                = <Label> {
                                    width: Fill
                                    text: "Application started successfully."
                                    draw_text: { color: #xA, text_style: <MAPLE_FONT> { font_size: 9 } }
                                }
                            }
                        }
                    }

                    files_view = <View> {
                        visible: false, width: Fill, height: Fill, flow: Down, spacing: 8
                        <Label> {
                            text: "No files available"
                            draw_text: { color: #x6, text_style: <MAPLE_FONT> { font_size: 11 } }
                        }
                    }

                    peers_view = <View> {
                        visible: false, width: Fill, height: Fill, flow: Down, spacing: 8
                        <Label> {
                            text: "No peers connected"
                            draw_text: { color: #x6, text_style: <MAPLE_FONT> { font_size: 11 } }
                        }
                    }
                }

                status_bar = <View> {
                    width: Fill, height: 28, flow: Right,
                    padding: { left: 12, right: 12, top: 4, bottom: 4 }, spacing: 12
                    draw_bg: { color: #x10 }

                    status_indicator = <View> {
                        width: 8, height: 8
                        draw_bg: { color: #xFF0, radius: 4 }
                    }
                    status_text = <Label> {
                        text: "Starting..."
                        draw_text: { color: #x6, text_style: <MAPLE_FONT> { font_size: 9 } }
                    }

                    <View> { width: Fill, height: Fill }

                    version_label = <Label> {
                        text: "v0.1.0"
                        draw_text: { color: #x4, text_style: <MAPLE_FONT> { font_size: 9 } }
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl AppMain for App {
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

        let config = match config::PeerBoxConfig::load() {
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
