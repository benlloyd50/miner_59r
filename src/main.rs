use std::{
    collections::HashMap,
    fmt,
    time::{Duration, Instant},
};

use bracket_lib::prelude::*;
use hecs::*;

struct State {
    pub world: World,
    pub input_state: InputState,
}

#[derive(Debug)]
struct Timer {
    ticks: Duration,
    duration: Duration,
    last_tick: Instant,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            ticks: Duration::ZERO,
            duration,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self, now: Instant) {
        let delta = now - self.last_tick;
        self.ticks += delta;
        self.last_tick = Instant::now();
    }

    // checks if the timer has elapsed the time
    pub fn finished(&self) -> bool {
        self.ticks >= self.duration
    }

    pub fn reset(&mut self) {
        self.ticks = Duration::ZERO;
        self.last_tick = Instant::now();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        tick_held_keys(&mut self.input_state);
        ctx.cls();

        handle_input(&mut self.world, &mut self.input_state);

        // if camera follow player is behind move actions then there's a "move effect" a little jarring but kind of cool?
        camera_follow_player(&mut self.world);

        // move actions can trigger other actions
        handle_move_actions(&mut self.world);
        handle_attack_actions(&mut self.world);

        render_state(&self.world, ctx);
    }
}

const GAME_WINDOW_SIZE: (u32, u32) = (85, 24);
const GAME_WINDOW_HALF: (u32, u32) = (GAME_WINDOW_SIZE.0 / 2, GAME_WINDOW_SIZE.1 / 2);

fn world_pos_to_screen(pos: &Position, cam_pos: &Position) -> Option<(u32, u32)> {
    // bounds of camera, todo unwrap this with the position of the game_box we should get camera positions that are then mapped to the console position elsewhere
    // right now this function makes assumptions about all that like where the game box lives
    let x_min = cam_pos.x as i64 - (GAME_WINDOW_HALF.0 as i64) + 1;
    let y_min = cam_pos.y as i64 - (GAME_WINDOW_HALF.1 as i64) + 1;
    let x_max = cam_pos.x as i64 + (GAME_WINDOW_HALF.0 as i64) + 1;
    let y_max = cam_pos.y as i64 + (GAME_WINDOW_HALF.1 as i64) - 1;

    let pos_i64 = (pos.x as i64, pos.y as i64);

    if pos_i64.0 < x_min.max(0)
        || pos_i64.1 < y_min.max(0)
        || pos_i64.0 > x_max
        || pos_i64.1 > y_max
    {
        return None;
    }

    // 1 accounts for the top left offset of the game window in the console screen
    let local_x = pos.x as u32 - x_min as u32 + 2;
    let local_y = pos.y as u32 - y_min as u32 + 2;

    Some((local_x, local_y))
}

#[derive(Debug)]
pub struct Camera {}

#[derive(Debug)]
pub struct Player {}

fn camera_follow_player(world: &mut World) {
    let mut p_q = world.query::<&Position>().with::<&Player>();
    let Some(player) = p_q.iter().next() else {
        return;
    };
    let mut c_q = world.query::<&mut Position>().with::<&Camera>();
    let Some(cam) = c_q.iter().next() else {
        return;
    };

    *cam = Position {
        x: player.x,
        y: player.y,
    };
}

fn render_state(world: &World, ctx: &mut BTerm) {
    let mut c_q = world.query::<&Position>().with::<&Camera>();
    let Some(cam) = c_q.iter().next() else {
        return;
    };

    let mut positions = world.query::<(&Position, &Renderable)>();
    for (pos, renderable) in positions.iter() {
        let Some(local_pos) = world_pos_to_screen(pos, cam) else {
            continue;
        };

        ctx.print_color(
            local_pos.0,
            local_pos.1,
            renderable.fg,
            renderable.bg,
            renderable.glyph,
        );
    }

    // ui box
    let ui_box_offset: (u32, u32) = (1, (CONSOLE_TILES_Y / 2) - 1);
    ctx.draw_hollow_box(
        ui_box_offset.0,
        ui_box_offset.1,
        CONSOLE_TILES_X - 3,
        (CONSOLE_TILES_Y / 2) - 3,
        SANDY_BROWN,
        BLACK,
    );

    print_debug_info(world, ctx, cam, ui_box_offset);

    // game box
    ctx.draw_hollow_box(
        1,
        1,
        CONSOLE_TILES_X - 3,
        (CONSOLE_TILES_Y / 2) - 3,
        SANDY_BROWN,
        BLACK,
    );

    ctx.print_centered(1, "Miner 59r");
    ctx.print_centered((CONSOLE_TILES_Y / 2) - 1, "Delving");
}

fn print_debug_info(world: &World, ctx: &mut BTerm, cam: &Position, ui_box_offset: (u32, u32)) {
    ctx.print(ui_box_offset.0 + 1, ui_box_offset.1 + 1, "DEBUG INFO:");

    let mut player_q = world.query::<&Position>().with::<&Player>();
    if let Some(player_pos) = player_q.iter().next() {
        ctx.print(
            ui_box_offset.0 + 1,
            ui_box_offset.1 + 2,
            format!("Player Position: {player_pos:?}"),
        )
    }

    let mut positions = world
        .query::<(&Position, &Renderable, Option<&Name>)>()
        .without::<&Camera>()
        .without::<&Player>();

    let mut idx = 0;
    let default_name = &Name::default();
    for (pos, render, name) in positions.iter() {
        // only show what would be visible to the user
        let Some(_) = world_pos_to_screen(pos, cam) else {
            continue;
        };

        let name = name.unwrap_or(default_name);
        ctx.print_color(
            ui_box_offset.0 + 1,
            ui_box_offset.1 + 3 + idx as u32,
            render.fg,
            RGB::from_f32(0., 0., 0.),
            format!("{name:?} {}", render.glyph),
        );
        idx += 1;
        if idx >= 7 {
            break;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveAction {
    dx: isize,
    dy: isize,
}

#[derive(Debug, Clone, Copy)]
pub struct AttackAction {
    target: Entity,
}

// move action

// check position to see if there's an entity there
// if no one then the move suceeds
// can possibly mutate position now or insert component and loop later?
// if there is then the move fails
// both cases the moveaction must be removed

fn handle_move_actions(world: &mut World) {
    let mut buf = CommandBuffer::new();

    {
        let mut pos_q = world.query::<(Entity, &Position, &MoveAction, &Name)>();
        for (mover, pos, dt, name) in pos_q.iter() {
            let x = pos.x.saturating_add_signed(dt.dx);
            let y = pos.y.saturating_add_signed(dt.dy);
            let dest_pos = Position { x, y };
            let mut pos_q_2 = world
                .query::<(Entity, &Position, &Name)>()
                .without::<&Camera>();

            let blockers: Vec<(Entity, Name)> = pos_q_2
                .iter()
                .filter(|(e, target_pos, _)| *e != mover && **target_pos == dest_pos)
                .map(|b| (b.0, b.2.clone()))
                .collect();

            if let Some(blocker) = blockers.first() {
                println!("{name:?} blocked from moving to {pos:?} by {:?}", blocker.1);
                buf.remove_one::<MoveAction>(mover);
                buf.insert_one(mover, AttackAction { target: blocker.0 });
                continue;
            }

            buf.insert_one(mover, dest_pos);
            buf.remove_one::<MoveAction>(mover);
            println!("{name:?} moved: {pos:?}");
        }
    }

    buf.run_on(world);
}

fn handle_attack_actions(world: &mut World) {
    let mut buf = CommandBuffer::new();

    {
        let mut pos_q = world.query::<(Entity, &AttackAction, &Name)>();
        for (attacker, attack, a_name) in pos_q.iter() {
            if let Ok((t_name,)) = world.query_one::<(&Name,)>(attack.target).get() {
                buf.despawn(attack.target);
                println!("{a_name:?} attacked {t_name:?} and killed it instantly");
            }
            buf.remove_one::<AttackAction>(attacker);
        }
    }
    buf.run_on(world);
}

// goal state: player can only move so fast. currently i want 1 tap direction and continuous?
// if you press a key it must be held for some amount of time to trigger continous movement

#[derive(Default)]
pub struct InputState {
    pub keys_held: HashMap<VirtualKeyCode, Duration>,
    pub last_update: Option<Instant>,
}

impl InputState {
    pub fn last_update(&self) -> Instant {
        self.last_update.unwrap_or(Instant::now())
    }

    pub fn just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_held
            .get(&key)
            .is_some_and(|duration| duration.is_zero())
    }

    pub fn key_held_for(&self, key: VirtualKeyCode, time_held_for: Duration) -> bool {
        self.keys_held
            .get(&key)
            .is_some_and(|duration| duration >= &time_held_for)
    }
}

fn tick_held_keys(input_state: &mut InputState) {
    let input = INPUT.lock();
    let pressed_set = input.key_pressed_set();

    let mut was_pressed_this_frame = vec![];
    {
        for pressed_key in pressed_set.iter() {
            // now is calced in here for the micro time diff in between loop iterations which honestly may not matter :shrugs:
            let now = Instant::now();
            let last_update = input_state.last_update();
            if let Some(state) = input_state.keys_held.get_mut(pressed_key) {
                *state += now - last_update;
                println!("update time {state:?}");
            } else {
                input_state.keys_held.insert(*pressed_key, Duration::ZERO);
            }
            was_pressed_this_frame.push(pressed_key.clone());
        }
    }

    input_state
        .keys_held
        .retain(|key, _| was_pressed_this_frame.contains(&key));

    input_state.last_update = Some(Instant::now());

    // if !input_state.keys_held.is_empty() {
    //     println!("held keys: {:#?}", input_state.keys_held);
    // }
}

fn handle_input(world: &mut World, input_state: &InputState) {
    let input = INPUT.lock();
    if input.is_key_pressed(VirtualKeyCode::Escape) {
        std::process::exit(0);
    }

    #[allow(unused)]
    let mut player_e = Entity::DANGLING;
    {
        let mut player_q = world.query::<Entity>().with::<&Player>();
        let Some(found) = player_q.iter().next().clone() else {
            eprintln!("No player entity, cannot handle input");
            return;
        };
        player_e = found;
    }

    const HELD_TIME: Duration = Duration::from_millis(200);

    let dt = if input_state.just_pressed(VirtualKeyCode::W)
        || input_state.key_held_for(VirtualKeyCode::W, HELD_TIME)
    {
        (0, -1)
    } else if input_state.just_pressed(VirtualKeyCode::A)
        || input_state.key_held_for(VirtualKeyCode::A, HELD_TIME)
    {
        (-1, 0)
    } else if input_state.just_pressed(VirtualKeyCode::D)
        || input_state.key_held_for(VirtualKeyCode::D, HELD_TIME)
    {
        (1, 0)
    } else if input_state.just_pressed(VirtualKeyCode::S)
        || input_state.key_held_for(VirtualKeyCode::S, HELD_TIME)
    {
        (0, 1)
    } else {
        (0, 0)
    };

    if dt != (0, 0) {
        let action = MoveAction { dx: dt.0, dy: dt.1 };
        let _ = world
            .insert(player_e, (action,))
            .inspect_err(|e| eprintln!("Failed to insert MoveAction onto player | MoveAction: {action:?} Player: {player_e:?} | Error: {e}"));
    }
}

const CONSOLE_TILES_X: u32 = 88;
const CONSOLE_TILES_Y: u32 = 54;
const SCREEN_DIMENSIONS_X: u32 = CONSOLE_TILES_X * 8 / 4;
const SCREEN_DIMENSIONS_Y: u32 = CONSOLE_TILES_Y * 8 / 4;
const FONT_PATH: &'static str = "Anikki_Square_8x8.png";

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_simple_console(CONSOLE_TILES_X, CONSOLE_TILES_Y, FONT_PATH)
        .with_fullscreen(false)
        .with_dimensions(SCREEN_DIMENSIONS_X, SCREEN_DIMENSIONS_Y)
        .with_font(FONT_PATH, 8u32, 8u32)
        .with_title("miner_59r")
        .build()?;

    let world = World::new();
    let mut gs = State {
        world,
        input_state: InputState::default(),
    };
    init_world(&mut gs);

    main_loop(context, gs)
}

fn init_world(state: &mut State) {
    state.world.spawn((
        Player {},
        Position { x: 50, y: 50 },
        Renderable::new('@', YELLOW, BLACK),
        Name::new("dude"),
    ));
    state.world.spawn((
        Camera {},
        Position { x: 50, y: 50 },
        Name::new("Main Camera"),
    ));

    state.world.spawn((
        Position { x: 0, y: 0 },
        Renderable::new('^', LIGHTBLUE, BLACK),
        Name::new("Small Ore Rock"),
    ));
    state.world.spawn((
        Position { x: 30, y: 30 },
        Renderable::new('D', RED, BLACK),
        Name::new("Fiery Dragon"),
    ));
    state.world.spawn((
        Position { x: 30, y: 60 },
        Renderable::new('g', LIMEGREEN, BLACK),
        Name::new("Goblin"),
    ));
    state.world.spawn((
        Position { x: 70, y: 30 },
        Renderable::new('☺', BROWN1, BLACK),
        Name::new("Mole Person"),
    ));
    state.world.spawn((
        Position { x: 70, y: 60 },
        Renderable::new('▲', AQUA, BLACK),
        Name::new("Small Mineral Deposit"),
    ));
    state.world.spawn((
        Position { x: 80, y: 60 },
        Renderable::new(to_char(244), AQUA, BLACK),
        Name::new("Mushroom Guy"),
    ));
}

#[derive(Debug, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Renderable {
    glyph: char, // cp437 codepoint
    fg: RGBA,
    bg: RGBA,
}

impl Renderable {
    pub fn new<C>(glyph: char, fg: C, bg: C) -> Self
    where
        C: Into<RGBA>,
    {
        Self {
            glyph,
            fg: fg.into(),
            bg: bg.into(),
        }
    }
}

/// REQUIRED for all entities that have an expectation of being interacted with.
/// This would include npcs, player, breakable items
#[derive(Clone)]
pub struct Name {
    inner: String,
}

impl Default for Name {
    fn default() -> Self {
        Self::new("No Name")
    }
}

impl Name {
    pub fn new(name: impl ToString) -> Self {
        Self {
            inner: name.to_string(),
        }
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
