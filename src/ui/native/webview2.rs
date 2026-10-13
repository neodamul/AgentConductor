use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};
use wry::WebViewBuilder;

/// Gives pages `window.__acRequest(body) -> Promise<string>` over wry IPC.
const BRIDGE: &str = r#"(()=>{let next=0;const pending=new Map();
window.__acReply=(id,ok,value)=>{const p=pending.get(id);if(!p)return;pending.delete(id);ok?p.resolve(value):p.reject(Error(value));};
window.__acRequest=(body)=>new Promise((resolve,reject)=>{const id=++next;pending.set(id,{resolve,reject});window.ipc.postMessage(JSON.stringify({id,body}));});})();"#;

struct Reply {
    id: u64,
    value: Result<String, String>,
}

pub fn open(room: Arc<super::super::Room>) -> Result<(), String> {
    let document = room.document()?;
    let mut event_loop = EventLoopBuilder::<Reply>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title(crate::ui::messages::text(
            "native.agentconductor_agent_room",
        ))
        .with_window_icon(Some(
            tao::window::Icon::from_rgba(include_bytes!("../assets/ac-64.rgba").to_vec(), 64, 64)
                .map_err(|e| {
                crate::ui::messages::render(
                    "native.ac_application_icon_initialization_failed_450d71",
                    &[("e", &format!("{}", e))],
                )
            })?,
        ))
        .with_inner_size(LogicalSize::new(1280.0, 860.0))
        .with_min_inner_size(LogicalSize::new(960.0, 640.0))
        .build(&event_loop)
        .map_err(|e| {
            crate::ui::messages::render(
                "native.native_window_initialization_failed_071776",
                &[("e", &format!("{}", e))],
            )
        })?;
    let proxy = event_loop.create_proxy();
    let closed = Arc::new(AtomicBool::new(false));
    let active = Arc::new(AtomicUsize::new(0));
    let (ipc_closed, ipc_active) = (closed.clone(), active.clone());
    let loaded = AtomicBool::new(false);
    let webview = WebViewBuilder::new()
        .with_incognito(true)
        .with_devtools(false)
        .with_initialization_script(BRIDGE)
        .with_html(document)
        // Only the embedded document may load; every later navigation is refused.
        .with_navigation_handler(move |_| !loaded.swap(true, Ordering::AcqRel))
        .with_ipc_handler(move |request| {
            let Ok(packet) = serde_json::from_str::<serde_json::Value>(request.body()) else {
                return;
            };
            let (Some(id), Some(body)) = (packet["id"].as_u64(), packet["body"].as_str()) else {
                return;
            };
            if ipc_closed.load(Ordering::Acquire) {
                return;
            }
            if ipc_active.fetch_add(1, Ordering::AcqRel) >= 8 {
                ipc_active.fetch_sub(1, Ordering::AcqRel);
                let _ = proxy.send_event(Reply {
                    id,
                    value: Err(
                        crate::ui::messages::text("native.native_controller_is_busy").into(),
                    ),
                });
                return;
            }
            let (room, proxy, active) = (room.clone(), proxy.clone(), ipc_active.clone());
            let body = body.to_owned();
            let spawned = std::thread::Builder::new()
                .name("room-request".into())
                .spawn(move || {
                    let value = std::panic::catch_unwind(|| room.dispatch(body.as_bytes()))
                        .unwrap_or_else(|_| {
                            Err(crate::ui::messages::text("native.native_controller_failed").into())
                        })
                        .map(|value| value.to_string());
                    active.fetch_sub(1, Ordering::AcqRel);
                    let _ = proxy.send_event(Reply { id, value });
                });
            if spawned.is_err() {
                ipc_active.fetch_sub(1, Ordering::AcqRel);
            }
        })
        .build(&window)
        .map_err(|e| {
            crate::ui::messages::render(
                "native.webview2_initialization_failed_install_the_microsoft_edge_webview2",
                &[("e", &format!("{}", e))],
            )
        })?;
    event_loop.run_return(move |event, _, control| {
        *control = ControlFlow::Wait;
        match event {
            Event::UserEvent(reply) => {
                let (ok, value) = match reply.value {
                    Ok(value) => (true, value),
                    Err(error) => (false, error),
                };
                let value = serde_json::to_string(&value).unwrap_or_else(|_| "\"\"".into());
                let _ = webview
                    .evaluate_script(&format!("window.__acReply({},{ok},{value})", reply.id));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                closed.store(true, Ordering::Release);
                *control = ControlFlow::Exit;
            }
            _ => {}
        }
    });
    Ok(())
}
