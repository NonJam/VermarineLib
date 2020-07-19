use tetra::{Context, Result, Event};

/// An enum representing the transitions to apply to the pushdown automaton
pub enum Trans<T> {
    /// Continue as normal
    None,

    /// Push the provided state to the Pushdown Automaton
    Push(Box<dyn PDAState<T>>),

    /// Pop the state off the top of the Pushdown Automaton
    Pop,

    /// Switches the state at the top of the Pushdown Automaton with the provided State
    Switch(Box<dyn PDAState<T>>),

    /// Replaces the stack with the provided State
    Replace(Box<dyn PDAState<T>>),

    /// Replaces the stack with the provided stack
    NewStack(Vec<Box<dyn PDAState<T>>>),

    /// Executes a sequence of Trans
    Sequence(Vec<Trans<T>>),

    /// Quit the engine
    Quit,
}

/// Pushdown Automaton struct that stores the list of pushed states along with helper methods for interacting with the automaton
pub struct PushdownAutomaton<T> {
    pub(crate) states: Vec<Box<dyn PDAState<T>>>,
    pub(crate) resource: T,
}

impl<T> PushdownAutomaton<T> {
    pub fn new<S, F1, F2>(ctx: &mut Context, state: F1, resource: F2) -> Result<Self> 
        where
        S: PDAState<T> + 'static,
        F1: FnOnce(&mut Context, &mut T) -> Result<S>,
        F2: FnOnce(&mut Context) -> Result<T>, {
        let mut resource = resource(ctx)?;
        let state = Box::new(state(ctx, &mut resource)?);

        Ok(PushdownAutomaton {
            states: vec![state],
            resource,
        })
    }

    pub(crate) fn push(&mut self, ctx: &mut Context, mut state: Box<dyn PDAState<T>>) {
        state.on_push(ctx, &mut self.resource);
        if let Some(s) = self.states.last_mut() {
            s.on_cover(ctx, &mut self.resource);
        }
        self.states.push(state);
    }

    pub(crate) fn pop(&mut self, ctx: &mut Context) {
        self.states.pop().unwrap().on_pop(ctx, &mut self.resource);
        if let Some(s) = self.states.last_mut() {
            s.on_uncover(ctx, &mut self.resource);
        }
    }

    pub(crate) fn switch(&mut self, ctx: &mut Context, state: Box<dyn PDAState<T>>) {
        self.pop(ctx);
        self.push(ctx, state);
    }

    pub(crate) fn replace(&mut self, ctx: &mut Context, state: Box<dyn PDAState<T>>) {
        self.new_stack(ctx, vec![state]);
    }

    pub(crate) fn new_stack(&mut self, ctx: &mut Context, states: Vec<Box<dyn PDAState<T>>>) {
        while let Some(mut s) = self.states.pop() {
            s.on_pop(ctx, &mut self.resource);
            if let Some(s) = self.states.last_mut() {
                s.on_uncover(ctx, &mut self.resource);
            }
        }

        for s in states.into_iter() {
            self.push(ctx, s);
        }
    }

    pub(crate) fn sequence(&mut self, ctx: &mut Context, sequence: Vec<Trans<T>>) {
        for trans in sequence.into_iter() {
            self.run_trans(ctx, trans);
        }
    }

    pub(crate) fn run_trans(&mut self, ctx: &mut Context, trans: Trans<T>) {
        match trans {
            Trans::None => {},
            Trans::Push(state) => { self.push(ctx, state) },
            Trans::Pop => { self.pop(ctx) },
            Trans::Switch(state) => { self.switch(ctx, state) },
            Trans::Replace(state) => { self.replace(ctx, state) },
            Trans::NewStack(stack) => { self.new_stack(ctx, stack) },
            Trans::Sequence(sequence) => { self.sequence(ctx, sequence) },
            Trans::Quit => {},
        }
    }
}

impl<T> tetra::State for PushdownAutomaton<T> {
    fn update(&mut self, ctx: &mut Context) -> Result {
        let mut trans = None;
        if let Some(s) = self.states.last_mut() {
            trans = Some(s.update(ctx, &mut self.resource)?);
        }

        let len = self.states.len() - 1;
        for idx in (0..len).rev() {
            self.states[idx].shadow_update(ctx, &mut self.resource)?;
        }

        if let Some(trans) = trans {
            self.run_trans(ctx, trans);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result {
        let len = self.states.len() - 1;
        for idx in 0..len {
            self.states[idx].shadow_draw(ctx, &mut self.resource)?;
        }
        
        if let Some(s) = self.states.last_mut() {
            s.draw(ctx, &mut self.resource)?;
        }

        Ok(())
    }

    fn event(&mut self, ctx: &mut Context, event: Event) -> Result {
        if let Some(s) = self.states.last_mut() {
            s.event(ctx, &mut self.resource, event)?;
        }
        
        Ok(())
    }
}

/// A trait representing a type that contains game state and provides logic for updating it
/// and drawing it to the screen. This is where you'll write your game logic!
///
/// The methods on `State` allow you to return a `Result`, either explicitly or via the `?`
/// operator. If an error is returned, the game will close and the error will be returned from
/// the `run` function that was used to start it.
#[allow(unused_variables)]
pub trait PDAState<T> {
    /// Called when a window or input event occurs.
    fn event(&mut self, ctx: &mut Context, resources: &mut T, event: Event) -> Result {
        Ok(())
    }

    /// Called when the state is added to a Pushdown Automaton
    fn on_push(&mut self, ctx: &mut Context, resources: &mut T) {

    }

    /// Called when the state is removed from the Pushdown Automaton
    fn on_pop(&mut self, ctx: &mut Context, resources: &mut T) {

    }

    /// Called when the state has another state pushed ontop of it
    fn on_cover(&mut self, ctx: &mut Context, resources: &mut T) {

    }

    /// Called when the state has the state above it popped
    fn on_uncover(&mut self, ctx: &mut Context, resources: &mut T) {

    }

    /// Called when it is time for the game to update and the state is on the top of the stack.
    fn update(&mut self, ctx: &mut Context, resources: &mut T) -> Result<Trans<T>> {
        Ok(Trans::None)
    }

    /// Called when it is time for the game to be drawn and the state is on the top of the stack.
    fn draw(&mut self, ctx: &mut Context, resources: &mut T) -> Result {
        Ok(())
    }

    /// Called when it is time for the game to update and the state is not on the top of the stack.
    fn shadow_update(&mut self, ctx: &mut Context, resources: &mut T) -> Result {
        Ok(())
    }

    /// Called when it is time for the game to be drawn and the state is not on the top of the stack.
    fn shadow_draw(&mut self, ctx: &mut Context, resources: &mut T) -> Result {
        Ok(())
    }
}