# Frontend Guidelines

**Framework:** Rust (Iced)
**Architecture:** Elm Architecture (Model-View-Update)

## 1. Architectural Rules

### No God Objects
The current `MaldApp` struct is too large. New features must be implemented as independent components with their own `State`, `Message`, `view()`, and `update()` methods.
- **Bad:** Adding 15 fields to `MaldApp` for a new calculator feature.
- **Good:** Creating `src/gui/components/calculator.rs` and adding one `CalculatorState` field to `MaldApp`.

### View Composition
Views should be pure functions of state.
- Break large `view()` functions into smaller helper functions (`view_header()`, `view_content()`, `view_sidebar()`).
- Do not compute logic inside `view()`. Compute in `update()` and store in state.

### Design System Usage
- **Never** use raw pixel values (e.g., `padding(15)`).
- **Always** import `theme::spacing` and use constants (e.g., `padding(spacing::LG)`).
- **Colors:** Use `theme::colors` constants. Never `Color::from_rgb(...)` in widgets.

## 2. Component Structure

```rust
// Standard Iced Component Pattern
pub struct MyComponent {
    state: ComponentState,
}

#[derive(Debug, Clone)]
pub enum ComponentMessage {
    ActionA,
    ActionB(String),
}

impl MyComponent {
    pub fn new() -> Self { ... }
    
    pub fn update(&mut self, message: ComponentMessage) {
        match message {
            // Update logic here
        }
    }
    
    pub fn view(&self) -> Element<ComponentMessage> {
        // Return widget tree
    }
}
```

## 3. State Management
- **Single Source of Truth:** Data should live in one place. If `MaldApp` owns the document, sub-components should iterate or borrow it, not duplicate it.
- **Message Passing:** Child components emit messages that bubble up to `MaldApp` if global state change is needed.

## 4. File Structure
- `src/gui/widgets/`: Reusable, generic UI elements (Buttons, Cards, Inputs).
- `src/gui/components/`: Functional modules (Sidebar, ActivityBar, FileTree).
- `src/gui/views/`: Full-screen layouts (Home, Editor, Graph).
