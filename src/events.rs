// # Reg
// event0 ON
// -- ToggleLayer() --
// event0 OFF

// # AND combinator
// event0 ON
// event1 ON
// -- ToggleLayer()  --
// event0 OFF
// event1 OFF

// # OR combinator
// event0 ON
// -- ToggleLayer()  --
// event0 OFF
// event1 ON
// -- ToggleLayer()  --
// event1 OFF

trait AltState {
    fn is_pending(&self) -> Bool;
    /// update with a new state
    fn update(&mut self, state: Bool);

    /// reset the pending state
    fn reset(&mut self);
}

struct EventState {
    /// E.g "ivy", "emacs-focus", "chrome-focus", etc...
    name: String,

    /// Current state of the event
    state: bool,

    /// True if the new state wasn't flused to ktrl yet
    pending: bool,
}

struct EventSoloCombinator {
    
}

struct EventAndCombinator {
    states: Vec<EventState>,
}

fn handle_event(event: String) {
    let (name, state) = parse_event(event);
    let mgr = managers.get(name);
    mgr.update(state);
    mgr.get_pending() -> Some
}
