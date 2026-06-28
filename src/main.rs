use bracket_lib::prelude::*;
use hecs::*;

struct State {
    pub world: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        handle_input(&mut self.world);
        camera_follow_player(&mut self.world);

        render_state(&self.world, ctx);
    }
}

const GAME_WINDOW: ((u32, u32), (u32, u32)) =
    ((2, 2), (CONSOLE_TILES_X - 3 - 1, CONSOLE_TILES_Y / 2));
const GAME_WINDOW_CENTER: (u32, u32) = (
    GAME_WINDOW.1.0 / 2 + GAME_WINDOW.0.0,
    GAME_WINDOW.1.1 / 2 + GAME_WINDOW.0.1,
);

fn world_pos_to_screen(pos: &Position, cam_pos: &Position) -> Option<(u32, u32)> {
    let x_min = cam_pos.x.saturating_sub(GAME_WINDOW_CENTER.0 as usize);
    let y_min = cam_pos.y.saturating_sub(GAME_WINDOW_CENTER.1 as usize);
    let x_max = cam_pos.x.saturating_add(GAME_WINDOW_CENTER.0 as usize);
    let y_max = cam_pos.y.saturating_add(GAME_WINDOW_CENTER.1 as usize);

    if pos.x < x_min || pos.y < y_min || pos.x > x_max || pos.y > y_max {
        return None;
    }

    let local_x = pos.x as u32 - x_min as u32 + GAME_WINDOW_CENTER.0;
    let local_y = pos.y as u32 - y_min as u32 + GAME_WINDOW_CENTER.1;

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

    let mut player = None;
    let mut positions = world.query::<(Entity, &Position)>().without::<&Camera>();
    for pos in positions.iter() {
        let Some(local_pos) = world_pos_to_screen(pos.1, cam) else {
            println!("no position skipping");
            continue;
        };

        if let Ok(_) = world.query_one::<&Player>(pos.0).get()
            && player.is_none()
        {
            player = Some(pos.1);
        }

        ctx.print_color(
            local_pos.0,
            local_pos.1,
            RGB::from_f32(1.0, 1.0, 0.0),
            RGB::from_f32(0., 0., 0.),
            "@",
        );
    }

    // ui box
    let ui_box_offset = (1, (CONSOLE_TILES_Y / 2) - 1);
    ctx.draw_hollow_box(
        ui_box_offset.0,
        ui_box_offset.1,
        CONSOLE_TILES_X - 3,
        (CONSOLE_TILES_Y / 2) - 3,
        SANDY_BROWN,
        BLACK,
    );

    ctx.print(ui_box_offset.0 + 1, ui_box_offset.1 + 1, "DEBUG INFO:");
    if let Some(player_pos) = player {
        ctx.print(
            ui_box_offset.0 + 1,
            ui_box_offset.1 + 2,
            format!("Player Position: {player_pos:?}"),
        )
    }

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

// todo: handle input should return some sort of command
fn handle_input(world: &mut World) {
    let input = INPUT.lock();
    if input.is_key_pressed(VirtualKeyCode::Escape) {
        std::process::exit(0);
    }

    let dt = if input.is_key_pressed(VirtualKeyCode::W) {
        (0, -1)
    } else if input.is_key_pressed(VirtualKeyCode::A) {
        (-1, 0)
    } else if input.is_key_pressed(VirtualKeyCode::D) {
        (1, 0)
    } else if input.is_key_pressed(VirtualKeyCode::S) {
        (0, 1)
    } else {
        (0, 0)
    };

    if dt != (0, 0) {
        let mut pos_q = world.query::<&mut Position>().with::<&Player>();
        for pos in pos_q.iter() {
            pos.x = pos.x.saturating_add_signed(dt.0);
            pos.y = pos.y.saturating_add_signed(dt.1);
            println!("moved: {pos:?}");
        }
    }
}

// const CONSOLE_TILES_X: u32 = 640 / 8;
// const CONSOLE_TILES_Y: u32 = 200 / 8;
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
    let mut gs = State { world };
    init_world(&mut gs);

    main_loop(context, gs)
}

fn init_world(state: &mut State) {
    state.world.spawn((Player {}, Position { x: 0, y: 0 }));
    state.world.spawn((Position { x: 3, y: 3 },));
    state.world.spawn((Position { x: 20, y: 10 },));
    state.world.spawn((Position { x: 20, y: 0 },));
    state.world.spawn((Camera {}, Position { x: 0, y: 0 }));
}

#[derive(Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
