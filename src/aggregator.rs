use std::collections::HashSet;
use tokio::sync::mpsc::{Sender, Receiver};

use crate::events::ExtEventState;
use crate::events::AltEvent;
use crate::events::AltEvent::*;
use crate::ktrl_client::KtrlIpcReq;
use crate::error::DynError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Requirement {
    RqFocus(String),
    RqExtEvent(String),
}

use Requirement::*;

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
    pub fn new(tx: Sender<KtrlIpcReq>, rx: Receiver<AltEvent>) -> Self {
        let ivy = AndAgg::new(
            vec![RqFocus("emacs".to_string()),
                 RqExtEvent("ivy".to_string())].into_iter().collect(),
            "TurnOnLayerAlias(\"ivy\")".to_string(),
            "TurnOffLayerAlias(\"ivy\")".to_string()
        );

        let aggs = vec![ivy];
        Self{tx, rx, aggs}
    }

    async fn send_ipc(&mut self, req: KtrlIpcReq) -> Result<(), DynError> {
        self.tx.send(req).await?;
        Ok(())
    }

    async fn handle_event(&mut self, event: AltEvent) -> Result<(), DynError> {
        dbg!(&event);

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

        for agg in &mut self.aggs {
            let is_on = agg.is_on();

            let is_changed = agg.state == is_on;
            if is_changed && is_on {
                self.send_ipc(agg.on_ipc.clone()).await?;
            } else if is_changed && is_on {
                self.send_ipc(agg.off_ipc.clone()).await?;
            }

            agg.state = is_on;
        }

        Ok(())
    }

    pub async fn event_loop(&mut self) -> Result<(), DynError> {

        loop {
            let event = self.rx.recv().await.unwrap();
            self.handle_event(event).await?;
        }
    }
}
