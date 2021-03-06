use std::collections::HashSet;
use tokio::sync::mpsc::{Sender, Receiver};

use crate::events::ExtEventState;
use crate::events::AltEvent;
use crate::events::AltEvent::*;
use crate::ktrl_client::KtrlIpcReq;
use anyhow::Result;
use crate::config::{AltCfg, Requirement, Requirement::*};

#[derive(Debug)]
struct AndAgg {
    /// readonly: outlines the requirement for this aggregator
    cfg: HashSet<Requirement>,

    /// set of the requirements that are currently met.
    /// E.g The required focused window is currently in focus
    active: HashSet<Requirement>,

    /// request that'll be sent when all the requirements are met (state == true)
    /// NOTE: ipc requests are only sent when `state` changes
    on_ipc: KtrlIpcReq,

    /// request that'll be sent when one of the requirements isn't met (state == false)
    /// NOTE: ipc requests are only sent when `state` changes
    off_ipc: KtrlIpcReq,

    /// Whether all the requirements are currently met or not
    state: bool,
}

impl AndAgg {
    fn new(cfg: HashSet<Requirement>, on_ipc: KtrlIpcReq, off_ipc: KtrlIpcReq) -> Self {
        AndAgg {
            cfg,
            active: HashSet::new(),
            on_ipc,
            off_ipc,
            state: false,
        }
    }

    fn is_on(&self) -> bool {
        self.cfg == self.active
    }
}

pub struct EvAggregator {
    tx: Sender<KtrlIpcReq>,
    rx: Receiver<AltEvent>,

    aggs: Vec<AndAgg>, // TODO: Generalize
}

impl EvAggregator {
    pub fn new(cfg: AltCfg, tx: Sender<KtrlIpcReq>, rx: Receiver<AltEvent>) -> Self {
        let aggs: Vec<AndAgg> = cfg.aggs.iter()
            .map(|agg| {
                AndAgg::new(agg.requirements.clone(),
                            agg.on_ipc.clone(),
                            agg.off_ipc.clone())
            }).collect();

        Self{tx, rx, aggs}
    }

    async fn send_effect(&mut self, req: KtrlIpcReq) -> Result<()> {
        let effect = format!("IpcDoEffect((fx: {}, val: Press))", req);
        self.tx.send(effect).await?;
        Ok(())
    }

    async fn handle_event(&mut self, event: AltEvent) -> Result<()> {
        for agg in &mut self.aggs {
            for requirement in &agg.cfg {
                match (requirement, &event) {
                    (RqFocus(pattern), AltFocusEvent(focus_ev)) => {
                        let window = focus_ev.window.to_lowercase();
                        if let Some(_) = window.find(pattern) {
                            agg.active.insert(requirement.clone());
                        } else {
                            agg.active.remove(&requirement);
                        }
                    },

                    (RqExtEvent(name), AltExtEvent(ext_ev)) => {
                        if name != &ext_ev.name {
                            continue;
                        }

                        if ext_ev.state == ExtEventState::On {
                            agg.active.insert(requirement.clone());
                        } else {
                            agg.active.remove(&requirement);
                        }
                    },

                    _ => continue,
                }
            }
        }

        let mut ipc_reqs: Vec<KtrlIpcReq> = vec![];
        for agg in &mut self.aggs {
            let is_on = agg.is_on();

            let is_changed = agg.state != is_on;
            if is_changed && is_on {
                ipc_reqs.push(agg.on_ipc.clone());
            } else if is_changed && !is_on {
                ipc_reqs.push(agg.off_ipc.clone());
            }

            agg.state = is_on;
        }

        for req in ipc_reqs {
            self.send_effect(req).await?;
        }

        Ok(())
    }

    pub async fn event_loop(&mut self) -> Result<()> {

        loop {
            let event = self.rx.recv().await.unwrap();
            self.handle_event(event).await?;
        }
    }
}
