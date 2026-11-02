//! Multiplayer without the Swofty server: run a host server in this launcher, or sign in to a
//! friend's with their host code (see `swofty_core::multiplayer::host`).

use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::mpauth;
use swofty_core::multiplayer::host::{self, HostServer, upnp};
use swofty_core::multiplayer::servercode;

use crate::i18n::{tr, tr_owned};
use crate::pages::{GUTTER, label};
use crate::shell::{Page, Shell};
use crate::theme::{Palette, ScrollArea};

/// The host server this launcher runs, with what the router said.
pub(crate) struct Hosting {
    pub(crate) server: HostServer,
    pub(crate) forwarded: Option<upnp::Forwarded>,
    /// The code friends use, once the public address is known.
    pub(crate) code: Option<String>,
    pub(crate) note: Option<String>,
    /// The server asks for a password.
    pub(crate) password: bool,
}

pub(crate) struct HostPanel {
    pub(crate) code: Entity<InputState>,
    pub(crate) name: Entity<InputState>,
    pub(crate) password: Entity<InputState>,
    pub(crate) address: Entity<InputState>,
    /// A driver name to release on this PC's host server.
    pub(crate) release: Entity<InputState>,
    /// What releasing a name did.
    pub(crate) release_note: Option<String>,
    pub(crate) hosting: Option<Hosting>,
    pub(crate) busy: Option<&'static str>,
    pub(crate) error: Option<String>,
    _inputs: [Subscription; 5],
}

/// A refused sign-in in the launcher's language. The sign-in gives the host server's message;
/// the two refusals about claimed driver names are known by their wording.
pub(crate) fn sign_in_error_text(error: &str) -> String {
    let reason = if error.contains("this name is taken on this host server") {
        "name_claimed"
    } else if error.contains("update F1 Multiplayer to sign in") {
        "secret_required"
    } else {
        return error.to_owned();
    };
    crate::career::reason_text(reason).map_or_else(|| error.to_owned(), |t| tr(t).to_owned())
}

impl HostPanel {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Shell>) -> Self {
        let code =
            cx.new(|cx| InputState::new(window, cx).placeholder(tr("Server code (123 456) or host code")));
        let name = cx.new(|cx| InputState::new(window, cx).placeholder(tr("Your driver name")));
        let password = cx.new(|cx| InputState::new(window, cx).placeholder(tr("Server password (optional)")));
        let address = cx.new(|cx| InputState::new(window, cx).placeholder(tr("Your public IP or DNS name")));
        let release = cx.new(|cx| InputState::new(window, cx).placeholder(tr("Driver name to release")));
        let changed = |cx: &mut Context<Shell>, input: &Entity<InputState>| {
            cx.subscribe(input, |_, _, e: &InputEvent, cx| {
                if matches!(e, InputEvent::Change) {
                    cx.notify();
                }
            })
        };
        let inputs = [
            changed(cx, &code),
            changed(cx, &name),
            changed(cx, &password),
            changed(cx, &address),
            changed(cx, &release),
        ];
        Self {
            code,
            name,
            password,
            address,
            release,
            release_note: None,
            hosting: None,
            busy: None,
            error: None,
            _inputs: inputs,
        }
    }
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().trim().to_owned()
}

impl Shell {
    fn mp_host_name(&self, cx: &App) -> String {
        let typed = value(&self.mp_page.host.name, cx);
        if typed.is_empty() {
            self.mp.as_ref().map_or_else(String::new, |s| s.name.clone())
        } else {
            typed
        }
    }

    /// Start the host server on this PC, ask the router to forward its port and sign in to it.
    pub(crate) fn mp_host_start(&mut self, cx: &mut Context<Self>) {
        let panel = &self.mp_page.host;
        if panel.busy.is_some() || panel.hosting.is_some() {
            return;
        }
        let name = self.mp_host_name(cx);
        if name.is_empty() {
            self.mp_page.host.error = Some(tr("Enter your driver name first.").into());
            cx.notify();
            return;
        }
        let password = Some(value(&panel.password, cx)).filter(|p| !p.is_empty());
        let address = Some(value(&panel.address, cx)).filter(|a| !a.is_empty());
        let dir = self.store.root.clone();
        self.mp_page.host.busy = Some("host");
        self.mp_page.host.error = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let server = HostServer::start(host::Config {
                        dir: dir.join(host::DATA_DIR),
                        port: host::DEFAULT_PORT,
                        name: format!("{name}'s server"),
                        password: password.clone(),
                        public: true,
                        discovery: Some(servercode::DISCOVERY_PORT),
                        fresh_code: false,
                    })
                    .map_err(|e| {
                        tr_owned("Couldn't start the server on port {port}: {error}")
                            .replace("{port}", &host::DEFAULT_PORT.to_string())
                            .replace("{error}", &e.to_string())
                    })?;
                    let session =
                        mpauth::sign_in_host(&dir, &server.local_endpoint().code(), &name, password.as_deref())
                            .map_err(|e| sign_in_error_text(&e))?;
                    let (forwarded, mut note) = match upnp::forward(server.port()) {
                        Ok(f) if f.behind_another_nat() => (
                            Some(f),
                            Some(
                                tr("Your router forwarded the port, but your internet connection sits behind another \
                                    router or carrier NAT, so friends may not reach you. Ask your provider for a \
                                    public IPv4 address, or let a friend with one host.")
                                .to_owned(),
                            ),
                        ),
                        Ok(f) => (Some(f), None),
                        Err(e) => (
                            None,
                            Some(
                                tr_owned(
                                    "The router didn't forward the port automatically ({error}). Forward TCP and UDP \
                                     port {port} to this PC in your router's settings.",
                                )
                                .replace("{error}", &e.to_string())
                                .replace("{port}", &server.port().to_string()),
                            ),
                        ),
                    };
                    let public = address.or_else(|| {
                        forwarded
                            .as_ref()
                            .and_then(|f| f.external_ip)
                            .map(|ip| ip.to_string())
                    });
                    if public.is_none() && note.is_none() {
                        note = Some(tr("Enter your public IP address to get the host code.").into());
                    }
                    let code = public.map(|a| server.code_for(&a));
                    if let Some(code) = &code
                        && let Err(e) = server.publish_code(code)
                    {
                        note = Some(tr_owned("The code directory: {error}").replace("{error}", &e.to_string()));
                    }
                    Ok::<_, String>((
                        Hosting {
                            server,
                            forwarded,
                            code,
                            note,
                            password: password.is_some(),
                        },
                        session,
                    ))
                })
                .await;
            this.update(cx, |s, cx| {
                s.mp_page.host.busy = None;
                match result {
                    Ok((hosting, session)) => {
                        s.mp_page.host.hosting = Some(hosting);
                        s.mp = Some(session);
                        s.page = Page::Multiplayer;
                    }
                    Err(e) => s.mp_page.host.error = Some(e),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// The host code from a public address the host typed.
    pub(crate) fn mp_host_set_address(&mut self, cx: &mut Context<Self>) {
        let address = value(&self.mp_page.host.address, cx);
        if let Some(h) = self.mp_page.host.hosting.as_mut()
            && !address.is_empty()
        {
            let code = h.server.code_for(&address);
            h.code = Some(code.clone());
            let saved = servercode::Saved::load_or_create(&self.store.root.join(host::DATA_DIR), false);
            cx.background_executor()
                .spawn(async move {
                    if let Ok(saved) = saved {
                        let _ = servercode::publish(&saved, &code);
                    }
                })
                .detach();
            cx.notify();
        }
    }

    /// Free a driver name on this PC's host server, so its driver can claim it again from another
    /// PC (they lost the PC they signed in from, say).
    pub(crate) fn mp_host_release_name(&mut self, cx: &mut Context<Self>) {
        let name = value(&self.mp_page.host.release, cx);
        if name.is_empty() {
            self.mp_page.host.release_note = Some(tr("Type the driver name to release.").into());
            cx.notify();
            return;
        }
        let dir = self.store.root.join(host::DATA_DIR);
        self.mp_page.host.release_note = Some(match host::release_name(&dir, &name) {
            Ok(true) => tr_owned("{name} is free again: the next sign-in with that name claims it.")
                .replace("{name}", &name),
            Ok(false) => tr_owned("Nobody has claimed {name} on your server.").replace("{name}", &name),
            Err(e) => tr_owned("Couldn't release {name}: {error}")
                .replace("{name}", &name)
                .replace("{error}", &e.to_string()),
        });
        cx.notify();
    }

    /// Stop the host server: every room on it closes.
    pub(crate) fn mp_host_stop(&mut self, cx: &mut Context<Self>) {
        let Some(mut hosting) = self.mp_page.host.hosting.take() else {
            return;
        };
        self.mp_leave(cx);
        let dir = self.store.root.clone();
        let session = self.mp.take().filter(|s| s.server.is_some());
        cx.background_executor()
            .spawn(async move {
                std::thread::sleep(Duration::from_millis(300));
                hosting.server.stop();
                if let Some(f) = &hosting.forwarded {
                    upnp::unforward(f);
                }
                if let Some(session) = session {
                    mpauth::sign_out(&dir, &session);
                }
            })
            .detach();
        cx.notify();
    }

    /// Sign in to a friend's host server with their host code.
    pub(crate) fn mp_join_host(&mut self, cx: &mut Context<Self>) {
        let panel = &self.mp_page.host;
        if panel.busy.is_some() {
            return;
        }
        let code = value(&panel.code, cx);
        let name = self.mp_host_name(cx);
        let password = Some(value(&panel.password, cx)).filter(|p| !p.is_empty());
        if code.is_empty() || name.is_empty() {
            self.mp_page.host.error = Some(tr("Enter the server code and your driver name.").into());
            cx.notify();
            return;
        }
        let dir = self.store.root.clone();
        self.mp_page.host.busy = Some("join");
        self.mp_page.host.error = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    mpauth::sign_in_host(&dir, &code, &name, password.as_deref())
                        .map_err(|e| sign_in_error_text(&e))
                })
                .await;
            this.update(cx, |s, cx| {
                s.mp_page.host.busy = None;
                match result {
                    Ok(session) => s.mp = Some(session),
                    Err(e) => s.mp_page.host.error = Some(e),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// The page shown before signing in: host a server, join a friend's, or use the Swofty server.
    pub(crate) fn mp_connect_page(&mut self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let panel = &self.mp_page.host;
        let busy = panel.busy;
        let card = |title: &'static str, body: &'static str| {
            v_flex()
                .flex_1()
                .min_w(px(280.))
                .p(px(16.))
                .gap(px(10.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.border)
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(div().text_size(px(13.)).text_color(p.text_muted).child(body))
        };
        let no_password = value(&panel.password, cx).is_empty();
        let host_card = card(
            tr("Host a server"),
            tr("Run the multiplayer server on this PC. It gets a six-digit server code; your friends join with it \
                (or with the longer host code over the internet). \
                Your router needs to forward one port (TCP and UDP); the launcher asks it to."),
        )
        .child(Input::new(&panel.name).small())
        .child(Input::new(&panel.password).small())
        .when(no_password, |d| {
            d.child(div().text_size(px(12.)).text_color(p.warning).child(tr(
                "Without a password, anyone on the internet who finds your server can sign in to it. Set one \
                 and send it to your friends.",
            )))
        })
        .child(Input::new(&panel.address).small())
        .child(
            Button::new("mp-host-start")
                .primary()
                .small()
                .icon(IconName::Play)
                .label(tr("Start server"))
                .loading(busy == Some("host"))
                .disabled(busy.is_some())
                .on_click(cx.listener(|this, _, _, cx| this.mp_host_start(cx))),
        );
        let join_card = card(
            tr("Join a friend's server"),
            tr("Type the six-digit server code (or paste the host code) your friend sent you, with your driver name \
                and the server's password if it has one."),
        )
        .child(Input::new(&panel.code).small())
        .child(Input::new(&panel.name).small())
        .child(Input::new(&panel.password).small())
        .child(
            Button::new("mp-join-host")
                .primary()
                .small()
                .label(tr("Connect"))
                .loading(busy == Some("join"))
                .disabled(busy.is_some())
                .on_click(cx.listener(|this, _, _, cx| this.mp_join_host(cx))),
        );
        v_flex()
            .size_full()
            .child(self.header(
                Page::Multiplayer.label(),
                tr("Play with friends who use the same addons: host a server on your PC, or join a friend's.").to_owned(),
                div(),
                p,
            ))
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .scroll_y("mp-connect")
                    .px(px(GUTTER))
                    .pt(px(4.))
                    .pb(px(24.))
                    .gap(px(16.))
                    .when_some(panel.error.clone(), |d, e| {
                        d.child(div().text_size(px(13.)).text_color(p.error).child(e))
                    })
                    .child(h_flex().gap(px(16.)).items_start().flex_wrap().child(host_card).child(join_card)),
            )
            .into_any_element()
    }

    /// The host server's status above the room list, while this launcher runs one or is signed in
    /// to one.
    pub(crate) fn mp_host_banner(&self, p: Palette, cx: &mut Context<Self>) -> Option<Div> {
        let session = self.mp.as_ref()?;
        let server = session.server.as_ref()?;
        let local = session
            .endpoint()
            .is_some_and(|e| e.authority().starts_with("127.0.0.1"));
        let panel = &self.mp_page.host;
        let body = match (&panel.hosting, local) {
            (Some(h), _) => {
                let (rooms, relays) = h.server.load();
                let code = h.code.clone();
                let short = servercode::display(h.server.server_code());
                v_flex()
                        .gap(px(8.))
                        .child(
                            h_flex()
                                .gap(px(10.))
                                .items_center()
                                .child(label(
                                    tr_owned("YOUR SERVER IS RUNNING · PORT {port}")
                                        .replace("{port}", &h.server.port().to_string()),
                                    p,
                                ))
                                .child(div().flex_1())
                                .child(
                                    div().text_size(px(12.)).text_color(p.text_muted).child(
                                        tr_owned("{rooms} room(s), {relays} relay(s)")
                                            .replace("{rooms}", &rooms.to_string())
                                            .replace("{relays}", &relays.to_string()),
                                    ),
                                )
                                .child(
                                    Button::new("mp-host-stop")
                                        .outline()
                                        .small()
                                        .label(tr("Stop server"))
                                        .on_click(cx.listener(|this, _, _, cx| this.mp_host_stop(cx))),
                                ),
                        )
                        .child(
                            h_flex()
                                .gap(px(8.))
                                .items_center()
                                .child(label(tr("SERVER CODE"), p))
                                .child(
                                    div()
                                        .flex_1()
                                        .text_size(px(22.))
                                        .font_weight(FontWeight::BOLD)
                                        .font_family("monospace")
                                        .child(short.clone()),
                                )
                                .child(
                                    Button::new("mp-host-copy-short")
                                        .outline()
                                        .small()
                                        .icon(IconName::Copy)
                                        .label(tr("Copy server code"))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.mp_copy_code(short.replace(' ', ""), cx)
                                        })),
                                ),
                        )
                        .when(!h.server.answers_lookups(), |d| {
                            d.child(
                                div().text_size(px(13.)).text_color(p.text_muted).child(
                                    tr_owned(
                                        "UDP port {port} is taken on this PC, so the server code can't be looked \
                                         up; use the host code.",
                                    )
                                    .replace("{port}", &servercode::DISCOVERY_PORT.to_string()),
                                ),
                            )
                        })
                        .child(match code {
                            Some(code) => h_flex()
                                .gap(px(8.))
                                .items_center()
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_size(px(13.))
                                        .font_family("monospace")
                                        .child(code.clone()),
                                )
                                .child(
                                    Button::new("mp-host-copy")
                                        .outline()
                                        .small()
                                        .icon(IconName::Copy)
                                        .label(tr("Copy host code"))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.mp_copy_code(code.clone(), cx)
                                        })),
                                ),
                            None => h_flex()
                                .gap(px(8.))
                                .child(div().w(px(260.)).child(Input::new(&panel.address).small()))
                                .child(
                                    Button::new("mp-host-address")
                                        .outline()
                                        .small()
                                        .label(tr("Make host code"))
                                        .on_click(cx.listener(|this, _, _, cx| this.mp_host_set_address(cx))),
                                ),
                        })
                        .when_some(h.note.clone(), |d, n| {
                            d.child(div().text_size(px(13.)).text_color(p.text_muted).child(n))
                        })
                        .when(h.forwarded.is_some() && !h.password, |d| {
                            d.child(div().text_size(px(13.)).text_color(p.warning).child(tr(
                                "Your router forwards the server's port and the server has no password: anyone \
                                 who finds it can sign in. Stop it and start it again with a password.",
                            )))
                        })
                        .child(
                            h_flex()
                                .gap(px(8.))
                                .items_center()
                                .child(div().w(px(220.)).child(Input::new(&panel.release).small()))
                                .child(
                                    Button::new("mp-host-release")
                                        .outline()
                                        .small()
                                        .icon(IconName::KeyRound)
                                        .label(tr("Release a driver name"))
                                        .on_click(cx.listener(|this, _, _, cx| this.mp_host_release_name(cx))),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_size(px(12.))
                                        .text_color(p.text_muted)
                                        .child(panel.release_note.clone().unwrap_or_else(|| {
                                            tr("A driver who lost the PC they signed in from can claim their name \
                                                again once you release it.")
                                            .to_owned()
                                        })),
                                ),
                        )
            }
            (None, true) => v_flex().gap(px(8.)).child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.))
                            .text_color(p.text_muted)
                            .child(tr(
                                "Your host server isn't running. Start it again to open rooms.",
                            )),
                    )
                    .child(
                        Button::new("mp-host-restart")
                            .primary()
                            .small()
                            .label(tr("Start server"))
                            .loading(panel.busy == Some("host"))
                            // Starting signs in afresh and replaces the stale loopback sign-in;
                            // until then that sign-in still names the driver.
                            .on_click(cx.listener(|this, _, _, cx| this.mp_host_start(cx))),
                    ),
            ),
            (None, false) => v_flex().child(
                div().text_size(px(13.)).text_color(p.text_muted).child(
                    tr_owned("Connected to the host server {server}")
                        .replace("{server}", server.split('#').next().unwrap_or("")),
                ),
            ),
        };
        Some(
            v_flex()
                .p(px(14.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.border)
                .bg(p.surface)
                .child(body),
        )
    }
}
