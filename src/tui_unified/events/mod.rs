// Event system - placeholder implementations

use crossterm::event::{KeyEvent, MouseEvent};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Quit,
    Refresh,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    KeyPress,
    Mouse,
    System,
    Git,
    Ui,
}

impl Event {
    pub fn event_type(&self) -> EventType {
        match self {
            Event::Key(_) => EventType::KeyPress,
            Event::Mouse(_) => EventType::Mouse,
            Event::Resize(_, _) => EventType::System,
            Event::Quit => EventType::System,
            Event::Refresh => EventType::Ui,
            Event::Custom(_) => EventType::Git,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventPriority {
    High,
    Normal,
    Low,
}

#[derive(Debug)]
pub enum EventResult {
    Handled,
    NotHandled,
    Quit,
}

pub struct EventFilter {
    allowed_types: Vec<EventType>,
}

impl EventFilter {
    pub fn new(allowed_types: Vec<EventType>) -> Self {
        Self { allowed_types }
    }

    pub fn should_process(&self, event: &Event) -> bool {
        self.allowed_types.contains(&event.event_type())
    }
}

pub struct EventHandler {
    events: VecDeque<Event>,
    filters: Vec<EventFilter>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            filters: Vec::new(),
        }
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn add_filter(&mut self, filter: EventFilter) {
        self.filters.push(filter);
    }
}

pub struct EventRouter {
    handlers: Vec<EventHandler>,
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn route_event(&mut self, event: Event) {
        for handler in &mut self.handlers {
            for filter in &handler.filters {
                if filter.should_process(&event) {
                    handler.push_event(event.clone());
                    break;
                }
            }
        }
    }
}
