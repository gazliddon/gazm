

#[derive(Clone, Debug)]
pub enum Events {
    CursorUp,
    CursorDown,
    Space,
    PageUp,
    PageDown,
    ScrollUp,
    ScrollDown,
    SimStep,
    SimRun,
    SimStop,
    ToggleBreakpoint,
    Step,
}
